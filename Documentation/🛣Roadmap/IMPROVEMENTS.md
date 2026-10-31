# 🛣 Improvements — DATATYPE.wireless.IAS (from the 2026-10-30 architecture audit)

What this repo should change to meet the standard, most valuable first. Source: [🔍Audits/ARCHITECTURE-AUDIT.md](../%F0%9F%94%8DAudits/ARCHITECTURE-AUDIT.md).
← [Roadmap](INDEX.md)

| # | Improvement | Category | Effort (S/M/L) | Evidence |
|---|---|---|---|---|
| 1 | Take `apk-plugin-health` from the runner (one-line bin over `apk_plugin_runner::health_check()`) — Waiting on PODS/POD.THICK/DISCOVERY.Engine (handoff filed) | Coupling & Cohesion | S | `SRC/Cargo.toml` `[[bin]] apk-plugin-health` |
| 2 | Build the image from this repository alone — Needs owner: publish an SDK tag carrying `apk_plugin_runner::jobs` and push DATATYPE.wireless.CSV (the kernel rev); then drop the COPYs and `--config` patches | Coupling & Cohesion | S | `Docker/Dockerfile` |

Delivered items: see [🌍Publishing/DELIVERY.md](../%F0%9F%8C%8DPublishing/DELIVERY.md).
