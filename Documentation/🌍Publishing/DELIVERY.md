# Delivery — DATATYPE.wireless.IAS

← [Wiki home](../README.md)

The importer ships compiled into the orchestrator, and as a worker image of its own:

| What | Built by | Lands |
|---|---|---|
| `ias_importer.rs` | `MANAGEMENT.DataMigration` (`apkaudio-data-migration`) compiles it by `#[path]`; `oaRustCore` re-exports that crate as `data_migration`; `apkaudio-orchestrator` links it. `NODE.BareMetal/Docker/Dockerfile` copies `PODS/POD.THIN/Softapps` so the path resolves in the image. | the **Node-BareMetal** container (`node-baremetal:local`); after an edit, rebuild that stack (`MANAGER.Docktor/K8/up-stack.sh NODE.BareMetal`) |
| `IMPORT:file:HTML/Entry.py` | nothing builds it; it is loaded by file path | read by `test_handler_status.py` |
| the files | `PODS/POD.THIN/Softapps` copied into **WEB.Frontend-Assets**; the monorepo's FTPS job uploads the checkout to apk.audio | nothing opens them there |
| `SRC/` + `ias_importer.rs` | `Docker/Dockerfile`, context the repo root (it links `contracts/` and `apk-plugin-runner`); `cargo build --release --locked` | the **Worker-WIRELESS_IAS** container (`worker-wireless-ias:local`, project `workers`, no port); after an edit, `docker compose -f PODS/POD.workers/Docker/docker-compose.yml up -d --build wireless-ias` |

Remote: `https://github.com/LikeDotAudio/APK.softapp.DATATYPE.wireless.IAS.git` (no workflow of its own).

**Tested:** by hand — `cargo test --lib ias` in MANAGEMENT.DataMigration (3 passed 2026-10-30); `test_handler_status.py` for the Python edge's `STATUS`. `.apk.scripts/smoke.py` checks only that HEAD is pushed ([SMOKE](../%F0%9F%A7%A9Manual%20and%20Artifacts/SMOKE.md)).

**Not automated:** no CI runs the tests; the Node-BareMetal rebuild is by hand; `cargo run --bin test_ias_importer` publishes retained to the live broker.
