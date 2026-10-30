# Worker-WIRELESS_IAS — the IAS report importer as a worker on the bus (LOOSE-workers-convert-rest).
#   docker compose -f 'PODS/POD.workers/Docker/docker-compose.yml' up -d --build
#   docker build -f 'PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.IAS/Docker/Dockerfile' -t worker-wireless-ias:local .
#
# ⚠️ THE BUILD CONTEXT IS THE REPO ROOT (`context: ../../../../../..` in the
#    compose file beside this one), because the worker links two crates that
#    live outside this repository: apkaudio-contracts and apk-plugin-runner.
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
# The tool: the parser at the repository root (the DataMigration crate
# compiles the same file by #[path]) and the worker crate in SRC/.
ARG TOOL=PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.IAS
COPY ["PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.IAS/ias_importer.rs", "./PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.IAS/ias_importer.rs"]
COPY ["PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.IAS/SRC", "./PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.IAS/SRC"]

RUN cargo build --release --locked --manifest-path "$TOOL/SRC/Cargo.toml" --bins \
 && mkdir -p /out \
 && cp target/release/apk-worker-wireless-ias target/release/apk-plugin-health /out/

# ---------- runtime ----------
FROM debian:bookworm-slim
# Not root: the worker opens no port and touches no device.
RUN useradd --uid 10001 --no-create-home --shell /usr/sbin/nologin worker
COPY --from=build /out/ /usr/local/bin/
USER worker
ENTRYPOINT ["apk-worker-wireless-ias"]
