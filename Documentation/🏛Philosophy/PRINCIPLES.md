# Principles — DATATYPE.wireless.IAS

The rules the IAS report importer keeps, each with the code that keeps it (in `SRC/importer/src/lib.rs` unless named). Ecosystem rules are cited, not copied: [Refusal over fabrication](../../../../../../../Documentation/%F0%9F%8F%9BPhilosophy/Doctrine/readme.Refusal%20over%20fabrication.md).

← [Wiki home](../README.md)

| Principle | Kept by | Where it breaks |
|---|---|---|
| **Read the report in the order it is written.** A zone heading applies to the tables under it. | one document-order walk over `p, table.Assignment`; test `each_channel_takes_the_zone_heading_above_its_table` | — (until 2026-10-30 every channel got the last zone) |
| **Do not invent a value.** An empty cell stays empty; `band_info` is empty because the report has no band column. | `band_info: String::new()`; test `nothing_is_invented` | — (was `"UHF"`, `Generic RF`, `CH_<n>`) |
| **Parsing has no side effect; publishing is someone else's step.** Neither entry point opens a broker connection. | `classify_ias_str`, `parse_ias_report_str`; the worker's `handle_job` (`SRC/src/lib.rs`) sends the reply | — (until LOOSE-datatype-datatype-wireless every parse published retained) |
| **The bus is the API.** A frame posts a job and reads the reply; nothing is retained, so a later tab does not see the last import. | `SRC/src/lib.rs` `run_job_agent` / `handle_job` | — |
| **Keep going on a bad cell.** A block whose frequency is not 100–3000 MHz is skipped. | the guard at line 112; `chunks_exact(4)` drops a partial block | — (skipped blocks are counted in `skipped_rows`; test `no_channels_is_empty_not_success_and_rejects_are_counted`) |
| **A stub says so in a constant.** The Python edge reports `STATUS = "unavailable"` without the Rust scraper and raises rather than parse HTML itself. `run_tests()` scrapes a one-table report through the scraper and is `False` when it is missing or answers wrongly; `start`/`stop` are documented no-ops (the edge is stateless). | `IMPORT.file.HTML/Entry.py` (`STATUS`, `convert_ias_html_to_markers`, `run_tests`); asserted by `IMPORT.file.HTML/Tests/test_entry.py` here and NODE.BareMetal's consumer check `test_handler_status.py` | — (until 2026-10-30 `run_tests()` returned `True` unconditionally) |
| **One parser, one owner.** The parser is the crate `apkaudio-wireless-ias`; the worker links it by path, MANAGEMENT.DataMigration by git rev. | `SRC/importer/Cargo.toml`; `SRC/Cargo.toml` | — (until 2026-10-30 DataMigration compiled this file by `#[path]` and built its runner from here) |
| **What the four wireless importers share is written once.** The band rule, the status word and the job envelope are the wireless kernel's; the job loop is the runner's. | `apkaudio_wireless_kernel::{usable_mhz…, import_status, job}`; `apk_plugin_runner::jobs::JobWorker` (`SRC/src/lib.rs`) | — (until 2026-10-30 the loop and envelope were copied line for line from the CSV worker) |
| **The answer is versioned and its shape pinned.** | `schema` (`job::SCHEMA`); test `the_result_shape_is_pinned` | the result is the parser's struct serialised, held by the shape test rather than a second DTO type |
| **Builds from its own repository.** Runner and contracts by `tag = "sdk-v0.9"`, the wireless kernel by its pushed rev; Cargo.lock pins those git sources; the image builds from `SRC/` alone. | `SRC/Cargo.toml`, `SRC/Cargo.lock`; `Docker/Dockerfile` (context `../SRC`) | building against a local runner checkout needs a hand-added, git-excluded `[patch]` |
