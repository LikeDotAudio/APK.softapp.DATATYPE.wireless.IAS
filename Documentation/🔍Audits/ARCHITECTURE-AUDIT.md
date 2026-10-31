# 🔍 Architecture audit — DATATYPE.wireless.IAS (2026-10-30)

Audited against the APK.audio multi-repo architecture standard: coupling, boundaries, duplication, contracts.
← [Audits](INDEX.md) · Improvements: [🛣Roadmap/IMPROVEMENTS.md](../%F0%9F%9B%A3Roadmap/IMPROVEMENTS.md)

## Bounded context
Importer for the IAS (Intermodulation Analysis System) HTML frequency-coordination report → channel records; shipped as the bus worker `Worker-WIRELESS_IAS` (job in, result out, no port) and compiled into DataMigration. Rust (parser `ias_importer.rs` + worker crate `SRC/`), a thin Python edge `IMPORT:file:HTML/Entry.py`; ~620 lines. Sampled: all of it — `Docker/Dockerfile`, `Docker/docker-compose.yml`, `SRC/Cargo.toml`, `SRC/config.ini`, `SRC/src/lib.rs`, `self_test.rs`, the bin, `ias_importer.rs`, `Entry.py`, `display.json`; compared the job loop with sibling DATATYPE.wireless.CSV.

## Context map
| Talks to | How (topic / endpoint / file / import) | Relationship |
|---|---|---|
| Any job client (the Data Migration frame) | sub `APK.audio/System/Protocols/wireless_ias/outgoing/job`, pub `…/incoming/result` (`schema: 1`) | Published Language (job envelope, versioned) |
| APK.plugin.SDK (`apk-plugin-runner`, `apkaudio-contracts`) | Cargo `tag = "sdk-v0.5"`, patched to the checkout locally and in the Dockerfile; job loop `apk_plugin_runner::jobs::JobWorker` | Conformist, by version |
| DATATYPE.wireless.CSV `SRC/kernel` (`apkaudio-wireless-kernel`) | Cargo git rev (band rule, status word, job envelope) | Shared kernel, one-way |
| MANAGEMENT.DataMigration | depends on `SRC/importer` (`apkaudio-wireless-ias`) by git rev | Supplier (crate) |
| `apk_wireless_import` (DATATYPE.wireless.CSV/SRC/python) | `IMPORT:file:HTML/Entry.py` imports it; pinned by rev in `IMPORT:file:HTML/requirements.txt` | Supplier (package) |
| oaRustCore (optional native wheel) | the Python edge's engine; absent → `STATUS = "unavailable"` | Customer of a native module |
| POD.workers aggregator | `include:`s `Docker/docker-compose.yml`; project `workers` | Pod aggregator (house pattern) |

## Scorecard
| Category | Score (0–3) | Finding |
|---|---|---|
| Coupling & Cohesion | 2 | SDK by tag, parser a crate with one owner, no `#[path]` from another repo; the image context is the repo root until an SDK tag carries `jobs` |
| Boundary Integrity | 3 | Pure parser; MQTT only through the runner's `JobWorker`; versioned answer with a pinned shape |
| Duplication Control | 3 | Job loop is the runner's, envelope and band rule the kernel's; MARKER_HEADERS/lifecycle shared via apk_wireless_import |
| Testing & Contracts | 3 | `SRC/check.sh`: worker (incl. self test, topics, shape), parser + fixture tests, the Python edge's Tests/; DataMigration's frame-header contract test |
| Change management | 2 | Worker 0.2.0, `schema: 1`; nothing tagged yet (tags are the owner's) |
Score key: 3 meets the standard · 2 minor gaps · 1 significant gaps · 0 absent / violated.

## Findings
1. **Image builds from the monorepo root.** `Docker/Dockerfile` COPYs contracts, the runner and the CSV kernel and patches them in with `--config`, because `sdk-v0.5` predates `apk_plugin_runner::jobs` and the kernel rev is not pushed. Needs owner: publish the next SDK tag and push DATATYPE.wireless.CSV.
2. **Good — no port, hardened container.** `cap_drop: [ALL]`, `read_only`, `no-new-privileges`, non-root uid 10001, runner health probe.
3. **Good — topics from the contract.** `job_and_result_topics_are_this_modules` asserts the job/result topic strings.
4. **Good — pure core, thin shell.** `handle_job` is three lines over the kernel; the parser publishes nothing.
