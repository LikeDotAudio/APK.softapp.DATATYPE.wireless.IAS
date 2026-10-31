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
| `SRC/importer/` | The parser crate `apkaudio-wireless-ias` and `tests/fixtures.rs` (the fixture, 168 channels). Linked by the worker by path, by MANAGEMENT.DataMigration by git rev. |
| `SRC/check.sh` | Every check here in one command: `cargo test --workspace --locked` in `SRC/` (and pytest for the Python edge). |
| `test_data/report.html` | Fixture: a real IAS report, seven `Zone:` headings, 168 channels. |
| `IMPORT.file.HTML/Entry.py` | The Python edge — see below. |
| `SRC/` | The worker crate `apk-worker-wireless-ias`: the parser crate `importer/` behind the runner's job loop (`apk_plugin_runner::jobs::JobWorker`) and the wireless kernel's job envelope, with its `config.ini` (section `[wireless_ias]`). Tested by `bash SRC/check.sh`; built as the image `worker-wireless-ias:local`. |
| `Docker/` | The worker stack: container `Worker-WIRELESS_IAS`, project `workers`, no port. Included by `PODS/POD.workers/Docker/docker-compose.yml`. |

### `SRC/importer/` (`apkaudio-wireless-ias`)

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
`IasParseResult` (status, `source_format: "IAS_HTML_Report"`, file path, count,
skipped rows, channels) and open no broker connection, so a test run never overwrites live state.

Who calls it: the worker below (`SRC/src/lib.rs` `handle_job` → `classify_ias_str`); the Data
Migration *IAS* frame posts the file to it as a job. The orchestrator's `DataMigration/+/FilePath`
inbox that used to call `parse_ias_report_str` and publish the result retained was retired
(NODE.BareMetal ed053f0, LOOSE-datamigration-frames-to-workers).

### Build and test

```bash
bash SRC/check.sh                       # everything below, --locked
cd SRC && cargo test --workspace        # worker (8), parser crate (4 + fixture 1)
```

The parser is the crate `apkaudio-wireless-ias` (`SRC/importer/`). The worker links it by path;
MANAGEMENT.DataMigration (and through it oaRustCore) links it by git rev — nobody compiles the
file by `#[path]` any more. `SRC/importer/tests/fixtures.rs` parses `test_data/report.html` from
its path (168 channels); it was the `test_ias_importer` `[[bin]]` DataMigration built from this
repository's file until the 2026-10-30 audit. The 100–3000 MHz guard and the status word are the
wireless kernel's (`apkaudio-wireless-kernel`, DATATYPE.wireless.CSV/SRC/kernel, by git rev).

### `IMPORT.file.HTML/Entry.py` (Python)

`convert_ias_html_to_markers(html)` returns the header row
`ZONE, GROUP, DEVICE, NAME, FREQ_MHZ, PEAK` and the rows from `oaRustCore.html_scraper.scrape_tables`
(Rust: `PODS/POD.THICK/NODE.BareMetal/SRC/Core/CORE.engine.RUST/PARSER/html_scraper.rs`), and raises
if that module is not importable. `STATUS` is `"live"` or `"unavailable"` accordingly, and
`PODS/POD.THICK/NODE.BareMetal/SRC/Core/TEST.runner.PYTHON/test_handler_status.py` loads this file
by path to assert it. `run_tests()` scrapes a one-table report through that scraper and returns
`False` when it is missing or answers wrongly — never `True` for having run nothing. `start`/`stop`
are documented no-ops: the edge holds nothing between calls. `MARKER_HEADERS`, the lifecycle and logging come from
`apk_wireless_import` (DATATYPE.wireless.CSV/SRC/python, pinned in `requirements.txt`), not from
NODE.BareMetal's `importer_logging`; `Tests/` here pins the behaviour (`python3 -m pytest -q Tests`).

---

### As a worker (`Worker-WIRELESS_IAS`, POD.workers)

The same parser answers jobs on the bus (LOOSE-workers-convert-rest). A client subscribes to
`APK.audio/System/Protocols/wireless_ias/incoming/result`, then publishes
`{"job_id": "<yours>", "filename": "report.html", ...}` to `APK.audio/System/Protocols/wireless_ias/outgoing/job`
with the file as "text". The answer carries `schema: 1`, the same `job_id`, `ok`, and the parse result of
`classify_ias_str` under `result`; a malformed job or one over `max_job_bytes` (8 MiB, `SRC/config.ini`) comes
back `ok: false` with an `error`. Nothing is retained and the worker opens no port. Measured
2026-10-30: the fixture `test_data/report.html` sent as a job came back `ok: true` with 168 channels.

```bash
docker compose -f PODS/POD.workers/Docker/docker-compose.yml up -d --build wireless-ias
bash SRC/check.sh                                # handle_job on the fixture, the self test, the shape test
```

## 🚀 Usage & Integration
* **Web Portal Entry**: none of its own; the records are for the Data Migration frames under
  `PODS/POD.THIN/WEB.Frontend-Assets/SRC/FrontEnd/Gui_Frames/3_Data_Migration/`. `display.json`
  declares the folder to the launcher with `entry: SRC/importer/src/lib.rs`, `tech: rust`, and
  `requires: {plugins: [wireless_ias], services: [broker]}`, so the Launch Pad marks it when
  Worker-WIRELESS_IAS is down.
* **Remote Repository**: `https://github.com/LikeDotAudio/APK.softapp.DATATYPE.wireless.IAS.git`

---

## 🏛 Philosophy

- **The bus is the API, through the worker.** The parser publishes nothing; the worker answers a
  job on `wireless_ias/incoming/result`, unretained (`SRC/src/lib.rs`).
- **One parser, no silent fallback.** The Python edge refuses to parse HTML itself; a missing Rust
  scraper shows as `STATUS = "unavailable"`, which a test checks, rather than as a report with no
  rows (`IMPORT.file.HTML/Entry.py` header).
- **Keep going on a bad row.** Cells that are not a frequency in 100–3000 MHz are skipped, not
  fatal (`classify_ias_str`).
- **Do not invent a value.** An empty cell stays empty and `band_info` is empty, since the
  report carries no band (`classify_ias_str`, test `nothing_is_invented`).

<!-- wiki:smoke -->
**Smoke test** — ✅ PASS 1/1 since 2026-10-30 12:51 EDT — [results](Documentation/%F0%9F%A7%A9Manual%20and%20Artifacts/SMOKE.md)
<!-- /wiki:smoke -->
