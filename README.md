# DATATYPE.wireless.IAS

> 📑 **IAS report importer** — reads an IAS (Intermodulation Analysis System) HTML frequency-coordination report, turns its assignment tables into channel records. It publishes nothing itself: on the bus, the records travel as the `wireless_ias` worker's reply to a job.
>
> 📚 **Wiki:** [Documentation/README.md](Documentation/README.md) — principles, architecture, delivery.

---

## 🧩 Architectural Role
* **Family**: DATATYPE — an importer for somebody else's file format
* **Category**: Wireless / RF coordination reports — source `1_IAS` of the four DataMigration importers (IAS, Soundbase, Wireless Workbench, CSV)
* **Pipeline Stage**: Data Migration — the records come back on `APK.audio/System/Protocols/wireless_ias/incoming/result`, unretained, as the answer to a job

---

## ⚙️ Repository Overview

| Path | What it is |
|---|---|
| `ias_importer.rs` | The importer. Compiled as module `ias_importer` of the `apkaudio-data-migration` crate. |
| `test_ias_importer.rs` | A `[[bin]]` runner of that crate: parses one fixture, prints the first ten channels, exits 1 on failure. |
| `test_data/report.html` | Fixture: a real IAS report, seven `Zone:` headings, 168 channels. |
| `IMPORT:file:HTML/Entry.py` | The Python edge — see below. |
| `SRC/` | The worker crate `apk-worker-wireless-ias`: `ias_importer.rs` (by `#[path]`) behind a job loop on `apk-plugin-runner`, with its `config.ini` (section `[wireless_ias]`). Tested by `cargo test --manifest-path SRC/Cargo.toml`; built as the image `worker-wireless-ias:local`. |
| `Docker/` | The worker stack: container `Worker-WIRELESS_IAS`, project `workers`, no port. Included by `PODS/POD.workers/Docker/docker-compose.yml`. |

### `ias_importer.rs`

`classify_ias_str(html, source_name)` parses and has no side effect;
`parse_ias_report(path)` reads a file and `parse_ias_report_str(html, source_name)` wraps it — neither publishes.

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

**No parse publishes** (since LOOSE-datatype-datatype-wireless). The functions return an
`IasParseResult` (status, `source_format: "IAS_HTML_Report"`, file path, `mqtt_topic`, count,
skipped rows, channels) and open no broker connection, so a test run never overwrites live state.
`mqtt_topic` still names `APK.audio/System/DataMigration/IAS/ImportedData` (from
`apkaudio_contracts::topics::data_migration`), but nothing publishes there any more.

Who calls it: the worker below (`SRC/src/lib.rs` `handle_job` → `classify_ias_str`); the Data
Migration *IAS* frame posts the file to it as a job. The orchestrator's `DataMigration/+/FilePath`
inbox that used to call `parse_ias_report_str` and publish the result retained was retired
(NODE.BareMetal ed053f0, LOOSE-datamigration-frames-to-workers).

### Build and test

The crate is `Softapps/MANAGEMENT_AND_UTILITIES/MANAGEMENT.DataMigration` — `mod.rs` pulls this
file in by `#[path]`, and `Cargo.toml` declares the runner as a `[[bin]]`:

```bash
cd PODS/POD.THIN/Softapps/MANAGEMENT_AND_UTILITIES/MANAGEMENT.DataMigration
cargo test --lib ias                     # the parser on the fixture; publishes nothing
cargo run  --bin test_ias_importer       # prints a sample; publishes nothing
```

Checked 2026-10-30: `cargo test --lib ias` 4 passed. The runner reads the fixture from
`../../DATATYPE/DATATYPE.wireless.IAS/test_data/report.html` (it used to look under the
DataMigration crate, where it no longer is).

### `IMPORT:file:HTML/Entry.py` (Python)

`convert_ias_html_to_markers(html)` returns the header row
`ZONE, GROUP, DEVICE, NAME, FREQ_MHZ, PEAK` and the rows from `oaRustCore.html_scraper.scrape_tables`
(Rust: `PODS/POD.THICK/NODE.BareMetal/SRC/Core/CORE:engine:RUST/PARSER/html_scraper.rs`), and raises
if that module is not importable. `STATUS` is `"live"` or `"unavailable"` accordingly, and
`PODS/POD.THICK/NODE.BareMetal/SRC/Core/TEST:runner:PYTHON/test_handler_status.py` loads this file
by path to assert it. `run_tests()` scrapes a one-table report through that scraper and returns
`False` when it is missing or answers wrongly — never `True` for having run nothing. `start`/`stop`
are documented no-ops: the edge holds nothing between calls. Logging comes from `importer_logging`
(in `NODE.BareMetal/SRC/Core/`).

---

### As a worker (`Worker-WIRELESS_IAS`, POD.workers)

The same parser answers jobs on the bus (LOOSE-workers-convert-rest). A client subscribes to
`APK.audio/System/Protocols/wireless_ias/incoming/result`, then publishes
`{"job_id": "<yours>", "filename": "report.html", ...}` to `APK.audio/System/Protocols/wireless_ias/outgoing/job`
with the file as "text". The answer carries the same `job_id`, `ok`, and the parse result of
`classify_ias_str` under `result`; a malformed job or one over `max_job_bytes` (8 MiB, `SRC/config.ini`) comes
back `ok: false` with an `error`. Nothing is retained and the worker opens no port. Measured
2026-10-30: the fixture `test_data/report.html` sent as a job came back `ok: true` with 168 channels.

```bash
docker compose -f PODS/POD.workers/Docker/docker-compose.yml up -d --build wireless-ias
cargo test --manifest-path SRC/Cargo.toml        # handle_job on the fixture, the self test
```

## 🚀 Usage & Integration
* **Web Portal Entry**: none of its own; the records are for the Data Migration frames under
  `PODS/POD.THIN/WEB.Frontend-Assets/SRC/FrontEnd/Gui_Frames/3_Data_Migration/`. `display.json`
  declares the folder to the launcher; its `entry`, `main.py`, is not a file here, so the launcher
  opens the folder itself.
* **Remote Repository**: `https://github.com/LikeDotAudio/APK.softapp.DATATYPE.wireless.IAS.git`

---

## 🏛 Philosophy

- **The bus is the API, through the worker.** The parser publishes nothing; the worker answers a
  job on `wireless_ias/incoming/result`, unretained (`SRC/src/lib.rs`).
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
