# Smoke test — DATATYPE.wireless.IAS

Is `DATATYPE.wireless.IAS` deployed? Written by `.apk.scripts/smoke.py` from the module's own shape; what each check means is in that script's header.

← [Wiki home](../README.md)

**❌ FAIL** — 2/3 checks, since 2026-11-12 00:16 EST

| Check | Target | Result | Detail |
|---|---|---|---|
| pushed | [APK.softapp.DATATYPE.wireless.IAS](https://github.com/LikeDotAudio/APK.softapp.DATATYPE.wireless.IAS.git) | ❌ | 1 commit(s) not pushed |
| container | `Worker-WIRELESS_IAS` | ✅ | running, healthy |
| web | [SRC/importer/src/lib.rs](https://apk.audio/APK%3ASoftapps/DATATYPE/DATATYPE.wireless.IAS/SRC/importer/src/lib.rs) | ✅ | HTTP 200 |

Re-run: `python3 .apk.scripts/smoke.py PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.IAS`

<!-- smoke:sig f55fe711ce14 since 2026-11-12 00:16 EST -->
