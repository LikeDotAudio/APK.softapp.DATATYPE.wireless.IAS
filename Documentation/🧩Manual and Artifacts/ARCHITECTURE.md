# Architecture — DATATYPE.wireless.IAS

← [Wiki home](../README.md)

## Using it

In the Data Migration tab, the *IAS* frame (`WEB.Frontend-Assets/SRC/FrontEnd/Gui_Frames/3_Data_Migration/1_IAS/IAS_Importer.json`) reads the chosen report in the browser and posts its text as a job to `APK.audio/System/Protocols/wireless_ias/outgoing/job`, then draws its table from the reply on `…/wireless_ias/incoming/result` (neither retained).

## How it works

```text
…/wireless_ias/outgoing/job ─▶ Worker-WIRELESS_IAS (SRC/src/lib.rs handle_job)
                    └─▶ ias_importer::classify_ias_str(html)  no side effect
                                walk <p> and table.Assignment in document order
                                  <p>Zone: X</p>       → zone = X
                                  table: first <th>    → group
                                         each <tr>, blocks of 4 <td>: device, name, —, <b>freq</b>
                                         keep 100–3000 MHz → IasChannel
                    └─▶ reply {job_id, ok, result} → …/wireless_ias/incoming/result (not retained)
```

`IasChannel { zone, group, device, name, freq_mhz, band_info }`; `IasParseResult { status, source_format: "IAS_HTML_Report", file_path, total_channels, skipped_rows, channels }`. The importer publishes nothing (LOOSE-datatype-datatype-wireless); it names no topic either (its `mqtt_topic` for the retired `…/IAS/ImportedData` was removed: LOOSE-importeddata-dead-topics).

## Files

| Path | What |
|---|---|
| `SRC/importer/` | the parser crate `apkaudio-wireless-ias` and its unit tests (no publisher), `tests/fixtures.rs` over `test_data/report.html`; MANAGEMENT.DataMigration depends on it by git rev |
| `SRC/check.sh` | the one local check: `cargo test --workspace --locked` (+ pytest for the Python edge) |
| `test_data/report.html` | a real IAS report: seven zone headings, 168 channels |
| `IMPORT.file.HTML/Entry.py` | Python edge: `convert_ias_html_to_markers` over `oaRustCore.html_scraper.scrape_tables` (a different Rust scraper) |

## Testing

```bash
bash PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.IAS/SRC/check.sh
```

## As a worker

`SRC/src/lib.rs` `handle_job` decodes a job (`text` or `content_b64`), refuses one over
`max_job_bytes` (8 MiB), runs `classify_ias_str` and returns `{schema, job_id, tool, ok, result, error, elapsed_ms}` (the wireless kernel's `job` envelope)
— always, so a caller waiting on a `job_id` is never left waiting. `run_job_agent` (the runner's `JobWorker`: session, ConnAck resubscribe, lane thread, busy answer) feeds it from
`APK.audio/System/Protocols/wireless_ias/outgoing/job` and answers on `…/incoming/result`, neither
retained. `apk-plugin-runner` adds the retained `incoming/api` description, the config, the will and
the self test (`SRC/src/self_test.rs`: a probe file compiled into the binary runs through
`handle_job`). The pod is `PODS/POD.workers`.
