# DATATYPE.wireless.IAS

> 📑 **IAS report importer** — reads an IAS (Intermodulation Analysis System) HTML frequency-coordination report, turns its assignment tables into channel records and publishes them on the bus.

---

## 🧩 Architectural Role
* **Family**: DATATYPE — an importer for somebody else's file format
* **Category**: Wireless / RF coordination reports — source `1_IAS` of the four DataMigration importers (IAS, Soundbase, Wireless Workbench, CSV)
* **Pipeline Stage**: Data Migration — the record lands on `APK.audio/System/DataMigration/IAS/ImportedData`

---

## ⚙️ Repository Overview

| Path | What it is |
|---|---|
| `ias_importer.rs` | The importer. Compiled as module `ias_importer` of the `apkaudio-data-migration` crate. |
| `test_ias_importer.rs` | A `[[bin]]` runner of that crate: parses one fixture, prints the first ten channels, exits 1 on failure. |
| `test_data/report.html` | Fixture: a real IAS report, seven `Zone:` headings, 168 channels. |
| `IMPORT:file:HTML/Entry.py` | The Python edge — see below. |

### `ias_importer.rs`

`classify_ias_str(html, source_name)` parses and has no side effect;
`parse_ias_report(path)` / `parse_ias_report_str(html, source_name)` call it and then publish.

1. Walks `<p>` and `table.Assignment` together, in document order. A `<p>` containing `Zone:`
   sets the zone for the tables below it.
2. For each `table.Assignment`, takes its first `<th>` as the group, then reads each `<tr>` in
   blocks of four `<td>` (IAS prints up to two channel blocks per row): device, name, —,
   frequency (inside `<b>` when present).
3. Keeps a block whose frequency parses to 100–3000 MHz as an
   `IasChannel { zone, group, device, name, freq_mhz, band_info }`. An empty cell stays empty,
   and `band_info` is empty — an IAS report has no band column.

On `test_data/report.html` that gives 168 channels across the parking-lot zone and Zones 1–6, in
order (unit test `each_channel_takes_the_zone_heading_above_its_table`). Until 2026-10-30 the
zone headings were read in a separate pass before the tables, so every channel got the last zone,
`Zone 6`; `band_info` was the constant `"UHF"` and empty cells became `Generic RF` / `CH_<n>`.

**Every parse publishes.** The `IasParseResult` (status, `source_format: "IAS_HTML_Report"`, file
path, topic, count, channels) goes out as pretty JSON, QoS 1, **retained**, to
`APK.audio/System/DataMigration/IAS/ImportedData` (built by
`apkaudio_contracts::topics::data_migration("IAS", "ImportedData")`) on the loopback broker, with
credentials from `apkaudio_contracts::broker_auth` when set and a 5 MB packet limit. It waits up
to 5 s for CONNACK and PUBACK; no broker is not an error.

Who calls it: `PODS/POD.THICK/NODE.BareMetal/SRC/Core/ORCHESTRATOR:hunter:VISA/src/services/data_migration.rs`
subscribes to `APK.audio/System/DataMigration/+/FilePath` and hands an IAS payload to
`oaRustCore::data_migration::ias_importer`.

### Build and test

The crate is `Softapps/MANAGEMENT_AND_UTILITIES/MANAGEMENT.DataMigration` — `mod.rs` pulls this
file in by `#[path]`, and `Cargo.toml` declares the runner as a `[[bin]]`:

```bash
cd PODS/POD.THIN/Softapps/MANAGEMENT_AND_UTILITIES/MANAGEMENT.DataMigration
cargo test --lib ias                     # the parser on the fixture; publishes nothing
cargo run  --bin test_ias_importer       # prints a sample — and publishes to the local broker, retained
```

Checked 2026-10-30: `cargo test --lib ias` 3 passed. The runner reads the fixture from
`../../DATATYPE/DATATYPE.wireless.IAS/test_data/report.html` (it used to look under the
DataMigration crate, where it no longer is).

### `IMPORT:file:HTML/Entry.py` (Python)

`convert_ias_html_to_markers(html)` returns the header row
`ZONE, GROUP, DEVICE, NAME, FREQ_MHZ, PEAK` and the rows from `oaRustCore.html_scraper.scrape_tables`
(Rust: `PODS/POD.THICK/NODE.BareMetal/SRC/Core/CORE:engine:RUST/PARSER/html_scraper.rs`), and raises
if that module is not importable. `STATUS` is `"live"` or `"unavailable"` accordingly, and
`PODS/POD.THICK/NODE.BareMetal/SRC/Core/TEST:runner:PYTHON/test_handler_status.py` loads this file
by path to assert it. Logging comes from `importer_logging` (in `NODE.BareMetal/SRC/Core/`).

---

## 🚀 Usage & Integration
* **Web Portal Entry**: none of its own; the records are for the Data Migration frames under
  `PODS/POD.THIN/WEB.Frontend-Assets/SRC/FrontEnd/Gui_Frames/3_Data_Migration/`. `display.json`
  declares the folder to the launcher; its `entry`, `main.py`, is not a file here, so the launcher
  opens the folder itself.
* **Remote Repository**: `https://github.com/LikeDotAudio/APK.softapp.DATATYPE.wireless.IAS.git`

---

## 🏛 Philosophy

- **The bus is the API.** The importer's output is a retained message on a topic built from the
  contract crate (`ias_importer.rs`, `ias_mqtt_topic()` and `publish_to_mqtt()`).
- **Topics come from the contract, never typed out** (comment above `ias_mqtt_topic()`).
- **One parser, no silent fallback.** The Python edge refuses to parse HTML itself; a missing Rust
  scraper shows as `STATUS = "unavailable"`, which a test checks, rather than as a report with no
  rows (`IMPORT:file:HTML/Entry.py` header).
- **Keep going on a bad row.** Cells that are not a frequency in 100–3000 MHz are skipped, not
  fatal (`classify_ias_str`).
- **Do not invent a value.** An empty cell stays empty and `band_info` is empty, since the
  report carries no band (`classify_ias_str`, test `nothing_is_invented`).

<!-- wiki:smoke -->
**Smoke test** — ✅ PASS 1/1 since 2026-10-30 12:51 EDT — [results](Documentation/%F0%9F%A7%A9Manual%20and%20Artifacts/SMOKE.md)
<!-- /wiki:smoke -->
