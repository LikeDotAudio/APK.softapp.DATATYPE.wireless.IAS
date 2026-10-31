# 🛣 Improvements — DATATYPE.wireless.IAS (from the 2026-10-30 architecture audit)

What this repo should change to meet the standard, most valuable first. Source: [🔍Audits/ARCHITECTURE-AUDIT.md](../%F0%9F%94%8DAudits/ARCHITECTURE-AUDIT.md).
← [Roadmap](INDEX.md)

| # | Improvement | Category | Effort (S/M/L) | Evidence |
|---|---|---|---|---|
| 1 | Repin the wireless kernel to the DATATYPE.wireless.CSV commit that moves the kernel to `sdk-v0.7` (committed locally, P08), so the lock stops carrying `sdk-v0.6` alongside `sdk-v0.7` — Waiting on push of DATATYPE.wireless.CSV | Duplication Control | S | `SRC/Cargo.toml` / `SRC/importer/Cargo.toml` `rev = "628590d…"`; two SDK sources in `SRC/Cargo.lock` |

Delivered items: see [🌍Publishing/DELIVERY.md](../%F0%9F%8C%8DPublishing/DELIVERY.md).
