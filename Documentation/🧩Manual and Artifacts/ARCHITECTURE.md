# Architecture — DATATYPE.wireless.IAS

← [Wiki home](../README.md)

## Using it

In the Data Migration tab, the *IAS* frame (`WEB.Frontend-Assets/SRC/FrontEnd/Gui_Frames/3_Data_Migration/1_IAS/IAS_Importer.json`) publishes the chosen report on `APK.audio/System/DataMigration/IAS/FilePath` — as a disk path, or JSON with `raw_content` / base64 `raw_blob` — and draws its table from `…/IAS/ImportedData`.

## How it works

```text
…/IAS/FilePath ─▶ NODE.BareMetal data-migration daemon (ORCHESTRATOR:hunter:VISA/src/services/data_migration.rs)
                    └─▶ oaRustCore::data_migration::ias_importer::parse_ias_report[_str]
                          └─▶ classify_ias_str(html)          no side effect
                                walk <p> and table.Assignment in document order
                                  <p>Zone: X</p>       → zone = X
                                  table: first <th>    → group
                                         each <tr>, blocks of 4 <td>: device, name, —, <b>freq</b>
                                         keep 100–3000 MHz → IasChannel
                          └─▶ publish retained, QoS 1 → APK.audio/System/DataMigration/IAS/ImportedData
```

`IasChannel { zone, group, device, name, freq_mhz, band_info }`; `IasParseResult { status, source_format: "IAS_HTML_Report", file_path, mqtt_topic, total_channels, channels }`. Publishing waits up to 5 s for CONNACK/PUBACK on the loopback broker; no broker is not an error.

## Files

| Path | What |
|---|---|
| `ias_importer.rs` | parser, publisher and unit tests; module `ias_importer` of `MANAGEMENT.DataMigration` (`mod.rs`, `#[path]`) |
| `test_ias_importer.rs` | `[[bin]] test_ias_importer`: parses the fixture, prints ten channels — **and publishes retained** |
| `test_data/report.html` | a real IAS report: seven zone headings, 168 channels |
| `IMPORT:file:HTML/Entry.py` | Python edge: `convert_ias_html_to_markers` over `oaRustCore.html_scraper.scrape_tables` (a different Rust scraper) |

## Testing

```bash
cd PODS/POD.THIN/Softapps/MANAGEMENT_AND_UTILITIES/MANAGEMENT.DataMigration
cargo test --lib ias        # 3 tests: zones in order, nothing invented, empty cell stays empty
```

## As a worker

`SRC/src/lib.rs` `handle_job` decodes a job (`text` or `content_b64`), refuses one over
`max_job_bytes` (8 MiB), runs `classify_ias_str` and returns `{job_id, tool, ok, result, error, elapsed_ms}`
— always, so a caller waiting on a `job_id` is never left waiting. `run_job_agent` feeds it from
`APK.audio/System/Protocols/wireless_ias/outgoing/job` and answers on `…/incoming/result`, neither
retained. `apk-plugin-runner` adds the retained `incoming/api` description, the config, the will and
the self test (`SRC/src/self_test.rs`: a probe file compiled into the binary runs through
`handle_job`). The pod is `PODS/POD.workers`.
