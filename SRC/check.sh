#!/usr/bin/env bash
# Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
# MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
# Every check this repository owns, in one command (2026-10-30 audit: "add CI"
# — the root CI verifies nothing on purpose, so this is what runs before a push).
#   bash SRC/check.sh
# From SRC/ itself: cargo finds .cargo/config.toml (the local SDK/kernel patch)
# from the working directory, not from --manifest-path.
set -euo pipefail
cd "$(dirname "$0")"
cargo test --workspace --locked
cd ..
python3 -m pytest -q "IMPORT:file:HTML/Tests" -p no:cacheprovider
