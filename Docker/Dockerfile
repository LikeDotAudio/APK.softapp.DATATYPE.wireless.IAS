# syntax=docker/dockerfile:1
# Worker-WIRELESS_IAS — the IAS report importer as a worker on the bus (LOOSE-workers-convert-rest).
#   docker compose -f 'PODS/POD.workers/Docker/docker-compose.yml' up -d --build
#   APK_DEPS_TOKEN=$(gh auth token) docker compose -f Docker/docker-compose.yml build   # BuildKit (secret)
#
# THE BUILD CONTEXT IS THIS REPOSITORY'S `SRC/` (`context: ../SRC` in the
#    compose file beside this one) and nothing else. Runner and contracts come
#    from the public APK.plugin.SDK by tag, the wireless kernel from
#    DATATYPE.wireless.CSV by rev — every one a git source pinned in Cargo.lock,
#    so `--locked` builds exactly what `cargo test` passed in a standalone clone.
#
# PRIVATE SIBLING: DATATYPE.wireless.CSV (the kernel) is a private repository.
#    The token arrives as a BuildKit SECRET, never a build arg, so it is in no
#    layer and no image history — the APK_DEPS_TOKEN pattern PLUGIN.SQL uses.
#    Compose: `build: { secrets: [apk_deps_token] }` + top-level
#    `secrets: { apk_deps_token: { environment: APK_DEPS_TOKEN } }`.
#    Without it the kernel fetch fails with "failed to authenticate".

# ---------- build ----------
FROM rust:1.94-bookworm AS build
WORKDIR /src
ENV CARGO_TARGET_DIR=/src/target
COPY . ./SRC

ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN --mount=type=secret,id=apk_deps_token,required=false \
    if [ -s /run/secrets/apk_deps_token ]; then \
      git config --global url."https://x-access-token:$(cat /run/secrets/apk_deps_token)@github.com/".insteadOf "https://github.com/"; \
    fi \
 && cargo build --release --locked --manifest-path SRC/Cargo.toml --bins \
 && rm -f /root/.gitconfig \
 && mkdir -p /out \
 && cp target/release/apk-worker-wireless-ias target/release/apk-plugin-health /out/

# ---------- runtime ----------
FROM debian:bookworm-slim
# Not root: the worker opens no port and touches no device.
RUN useradd --uid 10001 --no-create-home --shell /usr/sbin/nologin worker
COPY --from=build /out/ /usr/local/bin/
USER worker
ENTRYPOINT ["apk-worker-wireless-ias"]
