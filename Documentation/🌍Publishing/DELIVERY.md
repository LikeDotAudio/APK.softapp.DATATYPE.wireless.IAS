# Delivery — DATATYPE.wireless.IAS

← [Wiki home](../README.md)

The importer ships compiled into the orchestrator, and as a worker image of its own:

| What | Built by | Lands |
|---|---|---|
| the crate `apkaudio-wireless-ias` (`SRC/importer/`) | `MANAGEMENT.DataMigration` (`apkaudio-data-migration`) depends on it by git rev; `oaRustCore` re-exports that crate as `data_migration`; `apkaudio-orchestrator` links it. `NODE.BareMetal/Docker/Dockerfile` copies this repository so Core's `[patch]` resolves it locally. | the **Node-BareMetal** container (`node-baremetal:local`); after an edit, rebuild that stack (`MANAGER.Docktor/K8/up-stack.sh NODE.BareMetal`) |
| `IMPORT.file.HTML/Entry.py` | nothing builds it; it is loaded by file path | read by `test_handler_status.py` |
| the files | `PODS/POD.THIN/Softapps` copied into **WEB.Frontend-Assets**; the monorepo's FTPS job uploads the checkout to apk.audio | nothing opens them there |
| `SRC/` (worker + `importer/`) | `Docker/Dockerfile`, context this repository's `SRC/` (SDK tag and pinned git sources in Cargo.lock; the private kernel's token as a BuildKit secret; nothing from the monorepo); `cargo build --release --locked` | the **Worker-WIRELESS_IAS** container (`worker-wireless-ias:local`, project `workers`, no port); after an edit, `docker compose -f PODS/POD.workers/Docker/docker-compose.yml up -d --build wireless-ias` |

Remote: `https://github.com/LikeDotAudio/APK.softapp.DATATYPE.wireless.IAS.git` (no workflow of its own).

**Tested:** by hand, one command — `bash SRC/check.sh`: `cargo test --workspace --locked` in `SRC/` (worker 8, parser 4, fixture 1 — 2026-10-31, `git archive HEAD` alone against `sdk-v0.9`), `pytest IMPORT.file.HTML/Tests` (5 passed; outside the monorepo it needs `apk_wireless_import` installed from `IMPORT.file.HTML/requirements.txt` or `APK_WIRELESS_IMPORT_DIR` set); MANAGEMENT.DataMigration's `tests/frame_headers.rs` holds the records to the Data Migration frame. `.apk.scripts/smoke.py` checks only that HEAD is pushed ([SMOKE](../%F0%9F%A7%A9Manual%20and%20Artifacts/SMOKE.md)).

**Not automated:** no CI runs the tests (the root CI verifies nothing on purpose; `SRC/check.sh` is the pre-push check); the Node-BareMetal rebuild is by hand.

## Delivered 2026-10-31: portable file names (no `:` `|` or case twins)

- The monorepo now checks out on Windows and macOS: no tracked name holds `<>:"|?*`, a trailing dot or space, a device name or a case-only twin. Colon folders became dot folders (the POD.x convention).
- Renamed: `IMPORT:file:HTML` → `IMPORT.file.HTML` (loaded by file path, so nothing imports the folder name). References to NODE.BareMetal `CORE.engine.RUST` / `TEST.runner.PYTHON` follow.

## Delivered from the 2026-10-30 audit

- **Loose files into SRC/ (CC-03, from the SDK/contracts audit)** — `ias_importer.rs` and its `test_*` runner left the stack root (a move begun by the peer SDK fixer, uncommitted, adopted here): the parser is now `SRC/importer/src/lib.rs` and the runner is `SRC/importer/tests/fixtures.rs`. Verified by `cargo test --workspace`.
- **display.json requires (WK-07, from the SDK/contracts audit)** — `"requires": {"plugins": ["wireless_ias"], "services": ["broker"]}`, `entry: SRC/importer/src/lib.rs` (requires added by the peer SDK fixer, adopted here).
- **Job loop from the runner** — `run_job_agent` is `apk_plugin_runner::jobs::JobWorker`; `handle_job` is three lines over the wireless kernel's `read_file_job` / `FileJob::answer`. The hand-built rumqttc session, `failure`, base64 decoding and size limit copied from the CSV worker are gone. Verified by the worker tests and the self test.
- **SDK by tag** — runner and contracts from APK.plugin.SDK `tag = "sdk-v0.5"` then, the kernel from DATATYPE.wireless.CSV by rev; git-excluded `SRC/.cargo/config.toml` and the Dockerfile's `--config` patch all three to the checkout; Cargo.lock refreshed (contracts 0.2.0, W01 handoff). Verified by `docker build` of the IAS image (same recipe) and `cargo test --locked`. Standalone needed the next SDK tag and the pushed kernel rev at the time — since done: `sdk-v0.7`, kernel `646e960`, no local patch (the bullets below).
- **Parser is a crate DataMigration depends on** — `SRC/importer/` (`apkaudio-wireless-ias`), taken by MANAGEMENT.DataMigration by git rev instead of a `#[path]` include; the fixture path is this crate's own.
- **Shared 100–3000 MHz rule** — the importer asks `apkaudio_wireless_kernel::usable_mhz…` and `import_status`; the three re-typed copies (IAS, SOUNDbase, WWB) and CSV's two are gone.
- **Versioned result + shape test** — answers carry `schema: 1`; `the_result_shape_is_pinned` holds the result and channel keys.
- **One local check** — `SRC/check.sh` (`cargo test --workspace --locked` + pytest); the root CI verifies nothing on purpose.
- **Python edge: shared header row + lifecycle, no BareMetal import** — `IMPORT.file.HTML/Entry.py` takes `MARKER_HEADERS`, `Lifecycle` and `log` from `apk_wireless_import` (DATATYPE.wireless.CSV/SRC/python, pinned by rev in `requirements.txt`) instead of its own copy and NODE.BareMetal's `importer_logging`; `run_tests()` is False unless live and the scrape answers. `Tests/` added (5 passed).
- **Health probe from the runner, not its source file** — `SRC/src/bin/apk-plugin-health.rs` is one line over `apk_plugin_runner::health_check()` (W02 foundation); the `[[bin]]` that compiled DISCOVERY.Engine's `RUNNER/src/bin/apk-plugin-health.rs` by path is gone. Verified by `bash SRC/check.sh` and `docker build`.
- **The image builds from this repository alone** — `Docker/Dockerfile` copies only `SRC/` (compose `context: ../SRC`); the COPYs of `contracts/` and DISCOVERY.Engine's runner, the CSV kernel and every `--config patch` are gone. `SRC/Cargo.toml` takes runner and contracts by SDK tag (`sdk-v0.7` after the coordinator's repin; jobs arrived in `sdk-v0.6`) and the kernel at the pushed rev `628590d`; Cargo.lock now records those GitHub sources (no local patch). The kernel's repository is private, so the build takes the read token as a BuildKit secret (`apk_deps_token`, compose `secrets:`; the PLUGIN.SQL pattern), never a build arg. Verified by a compose build of `Docker/Dockerfile` with context `SRC/` and `cargo test --workspace` against GitHub.
- **One SDK copy in the lock** — the wireless kernel is pinned at DATATYPE.wireless.CSV `646e960` (kernel on `sdk-v0.7`) in `SRC/Cargo.toml` and `SRC/importer/Cargo.toml`, so `SRC/Cargo.lock` carries a single `apk-plugin-runner` / `apkaudio-contracts` (sdk-v0.7) instead of v0.6 beside v0.7. Verified by `cargo test --workspace --locked`.
- **SDK `sdk-v0.9`, kernel `6af4baa`; worker 0.3.0, importer 0.2.0** (PLAN-3432.01) — runner and contracts at `tag = "sdk-v0.9"`, the wireless kernel at DATATYPE.wireless.CSV `6af4baa` (kernel 0.2.0, on `sdk-v0.9`) in `SRC/Cargo.toml` and `SRC/importer/Cargo.toml`; `SRC/Cargo.lock` re-resolved minimally against GitHub, one `apkaudio-contracts`. MANAGEMENT.DataMigration still pins the importer at `6933cc4` (no SDK in that crate; it moves when DataMigration next re-pins). Verified 2026-10-31: the tree alone (`git archive`, no `.cargo` patch), `cargo test --workspace --locked` — worker 8, parser 4, fixture 1 pass.
