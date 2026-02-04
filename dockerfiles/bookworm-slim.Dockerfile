# Stage 1: Download and verify the merx binary from GitHub Releases.
FROM debian:bookworm-slim AS download

# VERSION must be provided via --build-arg (e.g., "0.1.1").
# TARGETARCH is automatically set by BuildKit (amd64 or arm64).
ARG VERSION
ARG TARGETARCH

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

# Map Docker's TARGETARCH to the Rust target triple used in release archives,
# download the archive and its SHA256 checksum, verify integrity, then extract.
RUN test -n "${VERSION}" || { echo "ERROR: VERSION build argument is required"; exit 1; } \
    && case "${TARGETARCH}" in \
         amd64) TARGET="x86_64-unknown-linux-gnu" ;; \
         arm64) TARGET="aarch64-unknown-linux-gnu" ;; \
         *) echo "Unsupported architecture: ${TARGETARCH}" && exit 1 ;; \
       esac \
    && ARCHIVE="merx-v${VERSION}-${TARGET}.tar.gz" \
    && curl -fsSL -o "/tmp/${ARCHIVE}" \
       "https://github.com/koki-develop/merx/releases/download/v${VERSION}/${ARCHIVE}" \
    && curl -fsSL -o "/tmp/${ARCHIVE}.sha256" \
       "https://github.com/koki-develop/merx/releases/download/v${VERSION}/${ARCHIVE}.sha256" \
    && cd /tmp && sha256sum -c "${ARCHIVE}.sha256" \
    && tar xzf "/tmp/${ARCHIVE}" -C /tmp \
    && cp /tmp/merx /usr/local/bin/merx

# Stage 2: Copy only the binary into a clean image.
FROM debian:bookworm-slim

COPY --from=download /usr/local/bin/merx /usr/local/bin/merx

ENTRYPOINT ["merx"]
