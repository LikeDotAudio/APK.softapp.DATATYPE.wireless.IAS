// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! The IAS report importer as a worker (LOOSE-workers-convert-rest).
//!
//! ```text
//!   browser ── APK.audio/System/Protocols/wireless_ias/outgoing/job ──▶ worker
//!   worker  ── APK.audio/System/Protocols/wireless_ias/incoming/result ─▶ browser
//! ```
//!
//! ONE JOB IN, ONE RESULT OUT, MATCHED BY `job_id`. Neither leaf retains (see
//! `contracts/rust/src/baremetal_io.rs`): a retained job would run again on
//! every reconnect, and a retained result would answer a question the next
//! reader never asked.
//!
//! [`handle_job`] is the whole tool and has no bus in it, so the tests below
//! run it on the fixture in `../test_data/` with no broker anywhere near.
//! [`run_job_agent`] hands it to the runner's job loop
//! (`apk_plugin_runner::jobs::JobWorker`); the envelope — `job_id`, `text` |
//! `content_b64`, the size limit, the versioned answer — is the wireless
//! kernel's (`apkaudio_wireless_kernel::job`), shared with the other three
//! wireless workers.

pub use apkaudio_wireless_ias as ias_importer;
pub mod carriers;
pub mod self_test;

use apkaudio_wireless_kernel::job;
use serde_json::Value;

/// The token the bus knows this worker by. The same word as `[wireless_ias]`.
pub const MODULE: &str = "wireless_ias";

/// The tool the result came from, for a reader that listens to several.
pub const TOOL: &str = "DATATYPE.wireless.IAS";

/// The compiled-in config, the same text the runner publishes at `/config`.
pub const CONFIG_INI: &str = include_str!("../config.ini");

/// Used when `config.ini` does not say.
const DEFAULT_MAX_JOB_BYTES: usize = 8 * 1024 * 1024;

/// `max_job_bytes` from `config.ini`, or the default.
pub fn max_job_bytes(config_ini: &str) -> usize {
    job::max_job_bytes(config_ini, MODULE, DEFAULT_MAX_JOB_BYTES)
}

/// Answer one job. Always returns a result document, never panics on input.
pub fn handle_job(payload: &[u8], max_bytes: usize) -> Value {
    match job::read_file_job(payload, TOOL, "report.html", "the report", max_bytes) {
        Ok(file) => file.answer(TOOL, ias_importer::classify_ias_str(&file.text(), &file.filename)),
        Err(failure) => failure,
    }
}

/// The carriers an IAS report names (ROAD-3519). One assignment is one
/// carrier. An IAS report carries no serial, so the id is formed
/// (src/carriers.rs) from `source file | zone | group | device | name`,
/// numbered `#n` in report order where those repeat. `channel` is the
/// report's channel name; `band_info` is empty (the report has no band
/// column), so `band` is the ITU band of the frequency. The report has no
/// channel number or maker, and its device column is the model.
pub fn carriers_of(answer: &Value) -> Option<(String, Vec<carriers::Carrier>)> {
    if answer["ok"] != true {
        return None;
    }
    let result = &answer["result"];
    let file = text(result, "file_path").to_string();
    let list = result["channels"].as_array()?.iter().filter_map(|r| {
        Some(carriers::Carrier {
            unit: carriers::Carrier::unit_of(&[&file, text(r, "zone"), text(r, "group"), text(r, "device"), text(r, "name")]),
            name: text(r, "name").into(),
            frequency_mhz: r["freq_mhz"].as_f64()?,
            channel: text(r, "name").into(),
            band: text(r, "band_info").into(),
            model: text(r, "device").into(),
            manufacturer: String::new(),
            device: text(r, "device").into(),
            zone: text(r, "zone").into(),
            group: text(r, "group").into(),
            source_file: file.clone(),
        })
    });
    Some((file.clone(), carriers::number_units(list.collect())))
}

/// A text field of a result row; empty when absent.
fn text<'a>(row: &'a Value, key: &str) -> &'a str {
    row[key].as_str().unwrap_or("")
}

/// The carriers of every imported file, restated as a Device/ tree and
/// sighted for presence (src/carriers.rs). Blocking, beside the job loop.
pub fn run_device_agent(host: &str, port: u16) {
    carriers::run(MODULE, host, port, carriers::restate_s(CONFIG_INI, MODULE, 30));
}

/// The job loop: the runner's. Blocking; the runner starts it on its own
/// thread and exits the process if it ever returns, so Docker restarts it.
pub fn run_job_agent(host: &str, port: u16) {
    let max = max_job_bytes(CONFIG_INI);
    apk_plugin_runner::jobs::JobWorker::new(MODULE, TOOL, move |payload: &[u8]| {
        // One job taken off outgoing/job and run: this worker's packet (ROAD-3472).
        apk_plugin_runner::loopback::packet_received();
        let answer = handle_job(payload, max);
        // Its carriers on the bus as a Device/ tree (ROAD-3519).
        if let Some((file, list)) = carriers_of(&answer) {
            carriers::imported(&file, list);
        }
        answer
    })
        // A job is a file, and base64 grows it by a third.
        .max_packet(max / 3 * 4 + 64 * 1024)
        .describe(|a| format!("ok={} channels={}", a["ok"], a["result"]["total_channels"]))
        .run(host, port);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The manifest the binary compiles in parses, declares the loopback port
    /// and lists every key of config.ini's [wireless_ias] that code reads (ROAD-3472).
    #[test]
    fn the_compiled_in_manifest_parses_and_reads_every_config_key() {
        let manifest = apkaudio_contracts::embedded_manifest!().expect("agent.toml parses");
        assert_eq!(manifest.name, MODULE);
        assert_eq!(manifest.kind, "worker");
        assert_eq!(manifest.telemetry_port, 8225);
        let config = apk_plugin_runner::ini_to_json(CONFIG_INI);
        let keys: Vec<&str> = config[MODULE].as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(manifest.unread_config_keys(keys), Some(0), "a [wireless_ias] key nothing reads");
    }
    use apkaudio_contracts::baremetal_io;
    use base64::Engine as _;
    use serde_json::json;

    const REPORT: &str = include_str!("../../test_data/report.html");

    fn job(body: Value) -> Vec<u8> {
        body.to_string().into_bytes()
    }

    #[test]
    fn a_text_job_is_parsed_and_answered_under_its_id() {
        let out = handle_job(&job(json!({"job_id": "j1", "filename": "report.html", "text": REPORT})), 1 << 20);
        assert_eq!(out["job_id"], "j1");
        assert_eq!(out["ok"], true, "{out}");
        assert_eq!(out["result"]["total_channels"], 168);
        assert_eq!(out["result"]["status"], "success");
        assert_eq!(out["result"]["file_path"], "report.html");
    }

    #[test]
    fn a_base64_job_is_the_same_as_its_text() {
        let b64 = base64::engine::general_purpose::STANDARD.encode(REPORT);
        let a = handle_job(&job(json!({"job_id": "j2", "filename": "report.html", "content_b64": b64})), 1 << 20);
        let b = handle_job(&job(json!({"job_id": "j2", "filename": "report.html", "text": REPORT})), 1 << 20);
        assert_eq!(a["ok"], true, "{a}");
        assert_eq!(a["result"], b["result"]);
    }

    #[test]
    fn a_bad_job_is_answered_not_dropped() {
        let out = handle_job(b"not json", 1 << 20);
        assert_eq!(out["ok"], false);
        assert!(out["job_id"].is_null());

        let out = handle_job(&job(json!({"text": REPORT})), 1 << 20);
        assert_eq!(out["ok"], false, "no job_id");

        let out = handle_job(&job(json!({"job_id": "j3"})), 1 << 20);
        assert_eq!(out["job_id"], "j3");
        assert_eq!(out["ok"], false, "no content");

        let out = handle_job(&job(json!({"job_id": "j4", "content_b64": "%%%"})), 1 << 20);
        assert_eq!(out["ok"], false, "bad base64");
    }

    #[test]
    fn an_oversized_job_is_refused_by_size() {
        let out = handle_job(&job(json!({"job_id": "j5", "text": REPORT})), 100);
        assert_eq!(out["ok"], false);
        assert!(out["error"].as_str().unwrap().contains("at most 100"));
    }

    #[test]
    fn the_limit_comes_from_config_ini() {
        assert_eq!(max_job_bytes(CONFIG_INI), 8_388_608);
        assert_eq!(max_job_bytes("[wireless_ias]\n"), DEFAULT_MAX_JOB_BYTES);
    }

    #[test]
    fn job_and_result_topics_are_this_modules() {
        let (job, retain) = baremetal_io::module_job(MODULE).unwrap();
        assert_eq!(job, "APK.audio/System/Protocols/wireless_ias/outgoing/job");
        assert!(!retain);
        let (result, retain) = baremetal_io::module_result(MODULE).unwrap();
        assert_eq!(result, "APK.audio/System/Protocols/wireless_ias/incoming/result");
        assert!(!retain);
    }

    /// The wire shape a browser reads. A parser field renamed is a failure
    /// here first; change it on purpose, together with `job::SCHEMA`.
    #[test]
    fn the_result_shape_is_pinned() {
        let out = handle_job(&job(json!({"job_id": "s", "filename": "report.html", "content_b64": base64::engine::general_purpose::STANDARD.encode(REPORT)})), 1 << 24);
        assert_eq!(out["ok"], true, "{out}");
        assert_eq!(out["schema"], job::SCHEMA);
        let keys = |v: &Value| {
            let mut k: Vec<String> = v.as_object().unwrap().keys().cloned().collect();
            k.sort();
            k
        };
        assert_eq!(keys(&out["result"]), ["channels", "file_path", "skipped_rows", "source_format", "status", "total_channels"]);
        assert_eq!(keys(&out["result"]["channels"][0]), ["band_info", "device", "freq_mhz", "group", "name", "zone"]);
        assert_eq!(job::SCHEMA, 1);
    }
    /// ROAD-3519: every carrier of the fixture becomes a Device/ row with
    /// PLUGIN.WIFI's leaf names, under an id formed from the file, not an address.
    #[test]
    fn the_fixture_puts_every_carrier_under_device() {
        let out = handle_job(&job(json!({"job_id": "d", "filename": "report.html", "text": REPORT})), 1 << 24);
        assert_eq!(out["ok"], true, "{out}");
        let (file, list) = carriers_of(&out).expect("an ok answer has carriers");
        assert_eq!(file, "report.html");
        assert_eq!(list.len() as u64, out["result"]["total_channels"].as_u64().unwrap());
        assert!(!list.is_empty());
        let ids: std::collections::BTreeSet<String> = list.iter().map(|c| c.device_id(MODULE)).collect();
        assert_eq!(ids.len(), list.len(), "every carrier its own id");
        let mut bands = 0;
        for c in &list {
            let f = c.fields(MODULE);
            let get = |leaf: &str| f.iter().find(|(t, _)| t.ends_with(&format!("/{leaf}"))).map(|(_, v)| v.clone()).unwrap();
            assert!(f[0].0.starts_with("APK.audio/System/Protocols/wireless_ias/Device/wireless_ias_plan_"), "{}", f[0].0);
            assert_eq!(get("device_class"), "Wireless Transmitter");
            let mhz: f64 = get("frequency_mhz").parse().unwrap();
            assert!(mhz > 100.0 && mhz < 3000.0, "{mhz}");
            assert!(!get("channel").is_empty() && !get("name").is_empty());
            if get("band") != "-" {
                bands += 1;
            }
            assert!(c.device_id(MODULE).starts_with("wireless_ias:plan:"));
        }
        assert!(bands > 0, "the band leaf is said (the file's, else the ITU band)");
        // The same file again lands on the same ids.
        let (_, again) = carriers_of(&handle_job(&job(json!({"job_id": "d", "filename": "report.html", "text": REPORT})), 1 << 24)).unwrap();
        assert_eq!(again.iter().map(|c| c.device_id(MODULE)).collect::<Vec<_>>(), list.iter().map(|c| c.device_id(MODULE)).collect::<Vec<_>>());
        assert!(carriers_of(&handle_job(b"not json", 1 << 24)).is_none(), "a failed job names no carrier");
    }
}
