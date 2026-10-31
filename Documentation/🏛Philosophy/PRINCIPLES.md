# Principles — DATATYPE.wireless.IAS

The rules the IAS report importer keeps, each with the code that keeps it (in `ias_importer.rs` unless named). Ecosystem rules are cited, not copied: [Refusal over fabrication](../../../../../../../Documentation/%F0%9F%8F%9BPhilosophy/Doctrine/readme.Refusal%20over%20fabrication.md).

← [Wiki home](../README.md)

| Principle | Kept by | Where it breaks |
|---|---|---|
| **Read the report in the order it is written.** A zone heading applies to the tables under it. | one document-order walk over `p, table.Assignment` (line 79); test `each_channel_takes_the_zone_heading_above_its_table` (line 156) | — (until 2026-10-30 every channel got the last zone) |
| **Do not invent a value.** An empty cell stays empty; `band_info` is empty because the report has no band column. | `band_info: String::new()` (line 124); test `nothing_is_invented` (line 172) | — (was `"UHF"`, `Generic RF`, `CH_<n>`) |
| **Parsing has no side effect; publishing is someone else's step.** Neither entry point opens a broker connection. | `classify_ias_str` (line 74), `parse_ias_report_str` (line 55); the worker's `handle_job` (`SRC/src/lib.rs`) sends the reply | — (until LOOSE-datatype-datatype-wireless every parse published retained) |
| **Topics come from the contract.** | `ias_mqtt_topic()` (line 40) → `apkaudio_contracts::topics::data_migration` | — |
| **The bus is the API.** A frame posts a job and reads the reply; nothing is retained, so a later tab does not see the last import. | `SRC/src/lib.rs` `run_job_agent` / `handle_job` | `IasParseResult.mqtt_topic` still names `…/IAS/ImportedData`, which nothing publishes to |
| **Keep going on a bad cell.** A block whose frequency is not 100–3000 MHz is skipped. | the guard at line 112; `chunks_exact(4)` (line 104) drops a partial block | — (skipped blocks are counted in `skipped_rows`; test `no_channels_is_empty_not_success_and_rejects_are_counted`) |
| **A stub says so in a constant.** The Python edge reports `STATUS = "unavailable"` without the Rust scraper and raises rather than parse HTML itself. `run_tests()` scrapes a one-table report through the scraper and is `False` when it is missing or answers wrongly; `start`/`stop` are documented no-ops (the edge is stateless). | `IMPORT:file:HTML/Entry.py` (`STATUS`, `convert_ias_html_to_markers`, `run_tests`); asserted by `NODE.BareMetal/SRC/Core/TEST:runner:PYTHON/test_handler_status.py` | — (until 2026-10-30 `run_tests()` returned `True` unconditionally) |
