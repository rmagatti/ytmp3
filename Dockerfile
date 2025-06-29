# Multi-stage Dockerfile using cargo-chef for optimized dependency caching
# This approach significantly speeds up Docker builds by caching Rust dependencies
# separately from application code changes.

FROM --platform=linux/amd64 rustlang/rust:nightly-bookworm as chef

RUN apt update && apt install -y bash curl npm libc-dev binaryen \
    protobuf-compiler libssl-dev libprotobuf-dev gcc git g++ libc-dev \
    make binaryen perl

RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-chef
RUN cargo install --locked cargo-leptos --version 0.2.35
RUN npm install -g sass

WORKDIR /work

# Prepare the recipe
FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build dependencies using cargo-chef
FROM chef as builder
COPY --from=planner /work/recipe.json recipe.json

# Build dependencies - this layer will be cached unless dependencies change
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/work/target \
    cargo chef cook --release --recipe-path recipe.json

# Copy package files and install npm dependencies for Tailwind/DaisyUI
COPY package.json package-lock.json* ./
RUN npm install

# Copy source code and build the application
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/work/target \
    cargo leptos build --release && \
    cp target/release/ytmp3 /tmp/ytmp3 && \
    cp -r target/site /tmp/site

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
COPY --from=builder /tmp/ytmp3 /app/ytmp3
COPY --from=builder /tmp/site /app/site
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