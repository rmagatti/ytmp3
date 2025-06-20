FROM --platform=linux/amd64 rustlang/rust:nightly-bookworm as builder

RUN apt update && apt install -y bash curl npm libc-dev binaryen \
    protobuf-compiler libssl-dev libprotobuf-dev gcc git g++ libc-dev \
    make binaryen perl

RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-generate
RUN cargo install --locked cargo-leptos --version 0.2.35
RUN npm install -g sass

WORKDIR /work
COPY . .

RUN cargo leptos build --release

FROM --platform=linux/amd64 debian:bookworm-slim as runtime
WORKDIR /app

# Install runtime dependencies including yt-dlp and ffmpeg
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends \
        openssl \
        ca-certificates \
        python3 \
        python3-pip \
        curl \
        file \
        ffmpeg \
    && pip3 install --no-cache-dir --break-system-packages --upgrade yt-dlp \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary and static assets
COPY --from=builder /work/target/release/ytmp3 /app/
COPY --from=builder /work/target/site /app/site
COPY --from=builder /work/Cargo.toml /app/

# Verify the binary architecture and make it executable
RUN file /app/ytmp3 && chmod +x /app/ytmp3

# Set environment variables for production
ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_SITE_ROOT="./site"
ENV LEPTOS_OUTPUT_NAME="ytmp3"
ENV LEPTOS_SITE_PKG_DIR="pkg"
ENV LEPTOS_RELOAD_PORT="3001"

EXPOSE 3000

# Run the server
CMD ["/app/ytmp3"]