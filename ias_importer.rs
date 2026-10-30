// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! Standalone IAS Importer & Test Runner written in Rust.
//! Parses IAS HTML reports (e.g., report.html), serializes to structured data,
//! and publishes the parsed channel matrix to the MQTT bus.

use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use apkaudio_contracts::topics::{self, MQTT_DEFAULT_PORT, MQTT_LOOPBACK_HOST};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IasChannel {
    pub zone: String,
    pub group: String,
    pub device: String,
    pub name: String,
    pub freq_mhz: f64,
    pub band_info: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IasParseResult {
    pub status: String,
    pub source_format: String,
    pub file_path: String,
    pub mqtt_topic: String,
    pub total_channels: usize,
    pub channels: Vec<IasChannel>,
}

/// `APK.audio/System/DataMigration/IAS/ImportedData`, built from the contract
/// rather than typed out — see `contracts/rust/src/topics.rs`, whose own header
/// names the DataMigration importers as the literals it exists to replace.
pub fn ias_mqtt_topic() -> String {
    topics::data_migration("IAS", "ImportedData")
        .expect("IAS/ImportedData are literal segments the contract accepts")
}

/// Standalone parser for IAS HTML Reports.
/// Extract zones, headers, device names, channel titles, and frequencies.
pub fn parse_ias_report(file_path: impl AsRef<Path>) -> Result<IasParseResult, String> {
    let path = file_path.as_ref();
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file {:?}: {}", path, e))?;
    parse_ias_report_str(&content, &path.to_string_lossy())
}

/// Parse raw IAS HTML report content and publish the result, retained.
pub fn parse_ias_report_str(content: &str, source_name: &str) -> Result<IasParseResult, String> {
    let result = classify_ias_str(content, source_name)?;
    if let Ok(json_payload) = serde_json::to_string_pretty(&result) {
        publish_to_mqtt(&ias_mqtt_topic(), &json_payload);
    }
    Ok(result)
}

fn text_of(el: scraper::ElementRef<'_>) -> String {
    el.text().collect::<Vec<_>>().join("").trim().to_string()
}

/// Parse raw IAS HTML report content. No side effect — `parse_ias_report_str` publishes.
///
/// The report is a sequence of `<p>Zone: …</p>` headings, each followed by the
/// `table.Assignment` blocks of that zone. They are walked in ONE pass, in document
/// order, so a table takes the zone heading above it. (Reading every heading first and
/// every table after left each channel with the last zone: all 168 channels of
/// `test_data/report.html` came out "Zone 6".)
///
/// A cell the report leaves empty stays empty, and `band_info` is empty: an IAS report
/// carries no band column, and a placeholder ("Generic RF", "CH_<n>", "UHF") reads
/// downstream as something the report said.
pub fn classify_ias_str(content: &str, source_name: &str) -> Result<IasParseResult, String> {
    let mut channels = Vec::new();
    let document = Html::parse_document(content);

    let sel = |s: &str| Selector::parse(s).map_err(|e| format!("Selector error: {:?}", e));
    let walk_selector = sel("p, table.Assignment")?;
    let tr_selector = sel("tr")?;
    let td_selector = sel("td")?;
    let th_selector = sel("th")?;
    let b_selector = sel("b")?;

    let mut current_zone = String::new();

    for el in document.select(&walk_selector) {
        if el.value().name() == "p" {
            // <p style="... font-size: large ...">Zone: ...</p>
            let p_text = text_of(el);
            if let Some(zone_str) = p_text.split("Zone:").nth(1) {
                current_zone = zone_str.trim().to_string();
            }
            continue;
        }

        // A table's group is its own <th>; a table without one has none.
        let group = el.select(&th_selector).next().map(text_of).unwrap_or_default();

        for row in el.select(&tr_selector) {
            let cells: Vec<_> = row.select(&td_selector).collect();
            // IAS lays out up to two channel blocks of 4 cells per row.
            for chunk in cells.chunks_exact(4) {
                let device = text_of(chunk[0]);
                let name = text_of(chunk[1]);
                // Frequency is wrapped inside <b>
                let freq_text = match chunk[3].select(&b_selector).next() {
                    Some(b_elem) => text_of(b_elem),
                    None => text_of(chunk[3]),
                };
                if let Ok(freq_mhz) = freq_text.parse::<f64>() {
                    if freq_mhz > 100.0 && freq_mhz < 3000.0 {
                        channels.push(IasChannel {
                            zone: current_zone.clone(),
                            group: group.clone(),
                            device,
                            name,
                            freq_mhz,
                            band_info: String::new(),
                        });
                    }
                }
            }
        }
    }

    Ok(IasParseResult {
        status: "success".to_string(),
        source_format: "IAS_HTML_Report".to_string(),
        file_path: source_name.to_string(),
        mqtt_topic: ias_mqtt_topic(),
        total_channels: channels.len(),
        channels,
    })
}

pub fn publish_to_mqtt(topic: &str, payload: &str) {
    use rumqttc::{Client, MqttOptions, Event, Packet, QoS};
    use std::time::Duration;

    let mut mqttoptions = MqttOptions::new("apkaudio-ias-importer", MQTT_LOOPBACK_HOST, MQTT_DEFAULT_PORT);
    mqttoptions.set_keep_alive(Duration::from_secs(10));
    if let Some((username, password)) = apkaudio_contracts::broker_auth::credentials() {
        mqttoptions.set_credentials(username, password);
    }
    // Max payload is ~35KB, set max_packet_size to 5MB
    mqttoptions.set_max_packet_size(5_000_000, 5_000_000);

    let (client, mut connection) = Client::new(mqttoptions, 10);
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let mut connected = false;
    let mut acked = false;

    for notification in connection.iter() {
        if let Ok(event) = notification {
            match event {
                Event::Incoming(Packet::ConnAck(_)) => {
                    connected = true;
                    let _ = client.publish(topic, QoS::AtLeastOnce, true, payload.as_bytes());
                    println!("[IAS MQTT PUBLISHER] Connected & Published payload to '{}'", topic);
                }
                Event::Incoming(Packet::PubAck(_)) => {
                    println!("[IAS MQTT PUBLISHER] Broker Acked payload on '{}'", topic);
                    acked = true;
                    break;
                }
                _ => {}
            }
        }
        if std::time::Instant::now() > deadline {
            break;
        }
    }
    if acked || connected {
        std::thread::sleep(Duration::from_millis(300));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPORT: &str = include_str!("test_data/report.html");

    #[test]
    fn each_channel_takes_the_zone_heading_above_its_table() {
        let r = classify_ias_str(REPORT, "report.html").unwrap();
        assert_eq!(r.total_channels, 168);
        let mut zones: Vec<&str> = r.channels.iter().map(|c| c.zone.as_str()).collect();
        zones.dedup();
        // in document order, each zone once: the parking lot, then Zone 1 … Zone 6
        assert_eq!(zones.len(), 7, "zones in order: {zones:?}");
        assert!(zones[0].starts_with("Parking Lot"));
        assert_eq!(&zones[1..], ["Zone 1", "Zone 2", "Zone 3", "Zone 4", "Zone 5", "Zone 6"]);
        let first_zone1 = r.channels.iter().find(|c| c.zone == "Zone 1").unwrap();
        assert_eq!(first_zone1.group, "E-Talk / \"Space\" Studio");
        assert_eq!(first_zone1.name, "ACR2 RF 19");
        assert_eq!(first_zone1.freq_mhz, 471.385);
    }

    #[test]
    fn nothing_is_invented() {
        let r = classify_ias_str(REPORT, "report.html").unwrap();
        assert!(r.channels.iter().all(|c| c.band_info.is_empty()));
        assert!(r.channels.iter().all(|c| c.device != "Generic RF" && !c.name.starts_with("CH_")));
    }

    #[test]
    fn an_empty_cell_stays_empty() {
        let html = r#"<p>Zone: A</p><table class="Assignment"><tr><td></td><td></td><td></td><td><b>500.0</b></td></tr></table>"#;
        let r = classify_ias_str(html, "x").unwrap();
        assert_eq!(r.total_channels, 1);
        let c = &r.channels[0];
        assert_eq!((c.zone.as_str(), c.group.as_str(), c.device.as_str(), c.name.as_str()), ("A", "", "", ""));
    }
}
