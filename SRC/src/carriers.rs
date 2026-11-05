// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! The carriers an imported file names, said as a `Device/` tree and as
//! presence (ROAD-3519).
//!
//! ```text
//! APK.audio/System/Protocols/{module}/Device/{key}/{leaf}   one per LEAVES entry, unkept, restated every restate_s
//! APK.audio/Discovery/{module}/{deviceId}                   DiscoveryPresence, the runner's (presence::seen)
//! ```
//!
//! THE SAME PATH PLUGIN.WIFI TAKES. Each carrier is a [`Sighting`] handed to
//! `apk_plugin_runner::presence::seen` with its `Device/` prefix, so the
//! runner keeps the retained link and identity beside the leaves, and the
//! leaves themselves are said unkept (`unkept::say`, QoS 0, not retained) on
//! the `restate_s` clock. The leaf names are PLUGIN.WIFI's —
//! `device_class`, `frequency_mhz`, `channel`, `band`, `name`, `model`,
//! `manufacturer` — so a reader that knows one radio knows these.
//!
//! IDENTITY. No wireless coordination file this worker reads carries a
//! serial, and an address (IP, MAC) is never an identity here. So the id is
//! FORMED, from what the file itself names the carrier by — [`Carrier::unit`],
//! which each worker builds and documents (see `carriers_of` in `lib.rs`) —
//! hashed: `deviceId = {module}:plan:{fnv1a64(unit)}`, and the `Device/`
//! key is that id as one topic segment (`{module}_plan_{hex}`). The same file
//! imported again names the same carriers, so it lands on the same ids.
//! `status` is `discovered`, never `identified`: a plan says a carrier
//! should be on the air, not that a unit answered.
//!
//! LIFETIME. A file's carriers are said until a file of the same name is
//! imported again (which replaces them) or the worker restarts (it keeps no
//! state: presence's clock, `presence_inactive_after_s`, then says them
//! offline). Carriers a re-import dropped are said gone and unlinked at once.

use std::collections::BTreeMap;
use std::sync::{Condvar, Mutex, OnceLock};
use std::time::Duration;

use apkaudio_contracts::device_publish::Sighting;
use apkaudio_contracts::device_record::DeviceRecordStatus;
use apkaudio_contracts::identity::fnv1a64;
use apkaudio_contracts::topics::{self, sanitize_segment};

/// What every carrier is: the transmitter on that frequency.
pub const DEVICE_CLASS: &str = "Wireless Transmitter";

/// Every leaf of one carrier, in the panel's order. A value the file does not
/// carry is said as `-`, as PLUGIN.WIFI does — except `band`: a file with no
/// band column (the IAS report, the ZONE/GROUP/DEVICE/NAME/FREQ CSVs) gets
/// the ITU band its frequency lies in ([`itu_band`]), as PLUGIN.WIFI derives
/// `2.4 GHz` / `5 GHz` from a frequency.
pub const LEAVES: [&str; 11] = [
    "name",
    "device_class",
    "frequency_mhz",
    "channel",
    "band",
    "model",
    "manufacturer",
    "device",
    "zone",
    "group",
    "source_file",
];

/// One carrier as the file states it. Nothing here is guessed: an empty
/// string is a field the file left out.
#[derive(Debug, Clone, PartialEq)]
pub struct Carrier {
    /// What the file names this carrier by, joined: the input to its id.
    pub unit: String,
    pub name: String,
    pub frequency_mhz: f64,
    pub channel: String,
    pub band: String,
    pub model: String,
    pub manufacturer: String,
    pub device: String,
    pub zone: String,
    pub group: String,
    pub source_file: String,
}

impl Carrier {
    /// The identity input from the file's own names, in order; `#n` makes the
    /// nth row with the same names its own carrier (see [`number_units`]).
    pub fn unit_of(parts: &[&str]) -> String {
        parts.iter().map(|p| p.trim()).collect::<Vec<_>>().join("\u{1f}")
    }

    /// The sighting presence keeps: identity is the formed id, never an address.
    pub fn sighting(&self, module: &str) -> Sighting {
        Sighting::new(module, &format!("plan:{}", fnv1a64(&self.unit)))
            .class(DEVICE_CLASS)
            .made_by(&self.manufacturer, &self.model)
            .notes(&format!("planned in {}", self.source_file))
            .status(DeviceRecordStatus::Discovered)
    }

    pub fn device_id(&self, module: &str) -> String {
        self.sighting(module).device_id()
    }

    /// `APK.audio/System/Protocols/{module}/Device/{key}`.
    pub fn prefix(&self, module: &str) -> String {
        let key = sanitize_segment(&self.device_id(module));
        topics::protocols(module, &["Device", &key]).expect("a module token, `Device` and a sanitized key are segments")
    }

    /// The name a person reads: the file's own, else what it is and where.
    pub fn display_name(&self) -> String {
        if !self.name.trim().is_empty() {
            return self.name.trim().to_string();
        }
        let what = [&self.device, &self.model, &self.group, &self.zone].into_iter().map(|s| s.trim()).find(|s| !s.is_empty()).unwrap_or("carrier");
        format!("{what} {:.3} MHz", self.frequency_mhz)
    }

    /// Every leaf, in [`LEAVES`] order, as (topic, value).
    pub fn fields(&self, module: &str) -> Vec<(String, String)> {
        let or_dash = |s: &str| if s.trim().is_empty() { "-".to_string() } else { s.trim().to_string() };
        let values = [
            self.display_name(),
            DEVICE_CLASS.to_string(),
            format!("{:.3}", self.frequency_mhz),
            or_dash(&self.channel),
            if self.band.trim().is_empty() { itu_band(self.frequency_mhz).to_string() } else { self.band.trim().to_string() },
            or_dash(&self.model),
            or_dash(&self.manufacturer),
            or_dash(&self.device),
            or_dash(&self.zone),
            or_dash(&self.group),
            or_dash(&self.source_file),
        ];
        let prefix = self.prefix(module);
        LEAVES.iter().zip(values).map(|(k, v)| (format!("{prefix}/{k}"), v)).collect()
    }
}

/// The ITU band a frequency lies in: VHF is 30–300 MHz, UHF 300–3000 MHz.
/// The kernel's usable range (100–3000 MHz) keeps every carrier in one of them.
pub fn itu_band(mhz: f64) -> &'static str {
    match mhz {
        f if (30.0..300.0).contains(&f) => "VHF",
        f if (300.0..3000.0).contains(&f) => "UHF",
        _ => "-",
    }
}

/// Make every `unit` in a list distinct: the nth carrier whose names repeat
/// an earlier one's (two unnamed rows in one group) gets `#n`. File order is
/// the file's, so the same file numbers the same way every time.
pub fn number_units(mut carriers: Vec<Carrier>) -> Vec<Carrier> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for c in &mut carriers {
        let n = seen.entry(c.unit.clone()).or_insert(0);
        *n += 1;
        c.unit = format!("{}#{}", c.unit, n);
    }
    carriers
}

/// What is said now, per source file, and what a re-import dropped.
#[derive(Default)]
struct Book {
    shows: BTreeMap<String, Vec<Carrier>>,
    departed: Vec<Carrier>,
    fresh: bool,
}

fn book() -> &'static (Mutex<Book>, Condvar) {
    static BOOK: OnceLock<(Mutex<Book>, Condvar)> = OnceLock::new();
    BOOK.get_or_init(Default::default)
}

/// A file was imported: its carriers replace any earlier import of the same
/// file name, and the restate thread says them at once.
pub fn imported(source_file: &str, carriers: Vec<Carrier>) {
    let (lock, wake) = book();
    let mut b = lock.lock().unwrap_or_else(|e| e.into_inner());
    let old = b.shows.insert(source_file.to_string(), carriers).unwrap_or_default();
    let now: Vec<String> = b.shows[source_file].iter().map(|c| c.unit.clone()).collect();
    b.departed.extend(old.into_iter().filter(|c| !now.contains(&c.unit)));
    b.fresh = true;
    wake.notify_all();
}

/// Every carrier said now, every file.
pub fn current() -> Vec<Carrier> {
    let (lock, _) = book();
    lock.lock().unwrap_or_else(|e| e.into_inner()).shows.values().flatten().cloned().collect()
}

/// The restate loop: every `restate` (or at once after an import), each
/// carrier is sighted and its leaves said. Blocking; the runner starts it on
/// its own thread beside the job loop.
pub fn run(module: &'static str, host: &str, port: u16, restate: Duration) {
    let client = apk_plugin_runner::session::Session::new(module, "devices").to(host, port).capacity(4096).spawn_drained();
    println!("📡 [{module}] carriers of imported files under {}", topics::protocols(module, &["Device"]).unwrap_or_default());
    loop {
        let (all, departed) = {
            let (lock, wake) = book();
            let mut b = lock.lock().unwrap_or_else(|e| e.into_inner());
            if !b.fresh {
                b = wake.wait_timeout(b, restate).map(|(g, _)| g).unwrap_or_else(|e| e.into_inner().0);
            }
            b.fresh = false;
            let departed = std::mem::take(&mut b.departed);
            (b.shows.values().flatten().cloned().collect::<Vec<_>>(), departed)
        };
        for c in &departed {
            apk_plugin_runner::presence::gone(&c.device_id(module));
            apk_plugin_runner::presence::unlink(&c.prefix(module));
        }
        for c in &all {
            apk_plugin_runner::presence::seen(&c.sighting(module), Some(&c.prefix(module)));
            for (topic, value) in c.fields(module) {
                apk_plugin_runner::unkept::say(&client, topic, value);
            }
        }
    }
}

/// `restate_s` from `config.ini`'s `[module]`, or `default_s`.
pub fn restate_s(config_ini: &str, module: &str, default_s: u64) -> Duration {
    let config = apk_plugin_runner::ini_to_json(config_ini);
    let s = config[module]["restate_s"].as_str().and_then(|v| v.trim().parse::<u64>().ok()).filter(|s| *s > 0).unwrap_or(default_s);
    Duration::from_secs(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn carrier(name: &str, unit: &str) -> Carrier {
        Carrier {
            unit: unit.into(),
            name: name.into(),
            frequency_mhz: 566.6,
            channel: "".into(),
            band: "G57".into(),
            model: "AD4Q".into(),
            manufacturer: "".into(),
            device: "".into(),
            zone: "".into(),
            group: "IEM".into(),
            source_file: "plan.csv".into(),
        }
    }

    #[test]
    fn a_carrier_says_every_leaf_under_its_formed_id() {
        let c = carrier("Lead", "IEM\u{1f}Lead#1");
        let f = c.fields("wireless_test");
        assert_eq!(f.len(), LEAVES.len());
        let id = c.device_id("wireless_test");
        assert_eq!(id, format!("wireless_test:plan:{}", fnv1a64("IEM\u{1f}Lead#1")));
        assert!(f[0].0.starts_with(&format!("APK.audio/System/Protocols/wireless_test/Device/wireless_test_plan_{}/", fnv1a64("IEM\u{1f}Lead#1"))), "{}", f[0].0);
        let get = |leaf: &str| f.iter().find(|(t, _)| t.ends_with(&format!("/{leaf}"))).map(|(_, v)| v.as_str()).unwrap();
        assert_eq!(get("device_class"), "Wireless Transmitter");
        assert_eq!(get("frequency_mhz"), "566.600");
        assert_eq!(get("channel"), "-", "a channel the file left out is not invented");
        assert_eq!(get("band"), "G57");
        assert_eq!(get("name"), "Lead");
    }

    #[test]
    fn an_unnamed_carrier_is_named_by_what_it_is_and_where() {
        assert_eq!(carrier("", "x").display_name(), "AD4Q 566.600 MHz");
    }

    #[test]
    fn a_file_with_no_band_says_the_itu_band_of_the_frequency() {
        let mut c = carrier("Lead", "x");
        c.band = String::new();
        let band = c.fields("m").into_iter().find(|(t, _)| t.ends_with("/band")).unwrap().1;
        assert_eq!(band, "UHF");
        assert_eq!(itu_band(174.5), "VHF");
    }

    #[test]
    fn repeated_names_are_numbered_in_file_order() {
        let list = number_units(vec![carrier("", "g"), carrier("", "g"), carrier("", "h")]);
        let units: Vec<&str> = list.iter().map(|c| c.unit.as_str()).collect();
        assert_eq!(units, ["g#1", "g#2", "h#1"]);
    }

    #[test]
    fn a_reimport_replaces_the_file_and_departs_what_it_dropped() {
        imported("t-reimport.csv", number_units(vec![carrier("a", "a"), carrier("b", "b")]));
        imported("t-reimport.csv", number_units(vec![carrier("a", "a")]));
        let (lock, _) = book();
        let b = lock.lock().unwrap();
        assert_eq!(b.shows["t-reimport.csv"].len(), 1);
        assert!(b.departed.iter().any(|c| c.unit == "b#1"));
    }

    #[test]
    fn restate_comes_from_config_ini() {
        assert_eq!(restate_s("[m]\nrestate_s = 7\n", "m", 30), Duration::from_secs(7));
        assert_eq!(restate_s("[m]\n", "m", 30), Duration::from_secs(30));
    }
}
