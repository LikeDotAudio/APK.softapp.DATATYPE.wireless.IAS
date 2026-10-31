# Worker-WIRELESS_IAS — the IAS report importer as a worker on the bus (LOOSE-workers-convert-rest).
#   docker compose -f 'PODS/POD.workers/Docker/docker-compose.yml' up -d --build
#   docker build -f 'PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.IAS/Docker/Dockerfile' -t worker-wireless-ias:local .
#
# ⚠️ THE BUILD CONTEXT IS THE REPO ROOT (`context: ../../../../../..` in the
#    compose file beside this one). The crate names apkaudio-contracts and
#    apk-plugin-runner from APK.plugin.SDK by tag (sdk-v0.5) and the wireless
#    kernel from DATATYPE.wireless.CSV by rev; the `--config patch…` entries
#    below point all three at the checkout's copies, so `--locked` holds and
#    nothing is fetched for them. The context can shrink to this repository once
#    a published SDK tag carries `apk_plugin_runner::jobs` and the kernel's rev
#    is pushed (both the owner's).
#    Every COPY source below is a REAL repo-root path (PODS/README.md rule 1).
#    Same recipe as DATATYPE.wireless.CSV/Docker/Dockerfile.

# ---------- build ----------
FROM rust:1.94-bookworm AS build
WORKDIR /src
ENV CARGO_TARGET_DIR=/src/target
COPY rust-toolchain.toml ./

# THE DESTINATION MIRRORS THE CHECKOUT, so relative path dependencies resolve
# here as they do on disk. NODE.BareMetal/SRC/contracts is a SYMLINK to
# <repo>/contracts in a checkout; the two real directories land where it names.
COPY ["contracts/rust",   "./PODS/POD.THICK/NODE.BareMetal/SRC/contracts/rust"]
COPY ["contracts/tokens", "./PODS/POD.THICK/NODE.BareMetal/SRC/contracts/tokens"]
# The runner, and the workspace manifest it inherits `edition` from.
COPY ["PODS/POD.THICK/DISCOVERY.Engine/SRC/Cargo.toml", "./PODS/POD.THICK/DISCOVERY.Engine/SRC/Cargo.toml"]
COPY ["PODS/POD.THICK/DISCOVERY.Engine/SRC/RUNNER",     "./PODS/POD.THICK/DISCOVERY.Engine/SRC/RUNNER"]
# The wireless kernel (DATATYPE.wireless.CSV/SRC/kernel), patched in below.
COPY ["PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.CSV/SRC/kernel", "./PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.CSV/SRC/kernel"]
# The tool: the worker crate in SRC/, with its parser crate SRC/importer/.
ARG TOOL=PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.IAS
COPY ["PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.IAS/SRC", "./PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.IAS/SRC"]

RUN cargo build --release --locked \
      --config 'patch."https://github.com/LikeDotAudio/APK.plugin.SDK".apk-plugin-runner.path="PODS/POD.THICK/DISCOVERY.Engine/SRC/RUNNER"' \
      --config 'patch."https://github.com/LikeDotAudio/APK.plugin.SDK".apkaudio-contracts.path="PODS/POD.THICK/NODE.BareMetal/SRC/contracts/rust"' \
      --config 'patch."https://github.com/LikeDotAudio/APK.softapp.DATATYPE.wireless.CSV".apkaudio-wireless-kernel.path="PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.CSV/SRC/kernel"' \
      --manifest-path "$TOOL/SRC/Cargo.toml" --bins \
 && mkdir -p /out \
 && cp target/release/apk-worker-wireless-ias target/release/apk-plugin-health /out/

# ---------- runtime ----------
FROM debian:bookworm-slim
# Not root: the worker opens no port and touches no device.
RUN useradd --uid 10001 --no-create-home --shell /usr/sbin/nologin worker
COPY --from=build /out/ /usr/local/bin/
USER worker
ENTRYPOINT ["apk-worker-wireless-ias"]
