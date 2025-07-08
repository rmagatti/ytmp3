FROM ghcr.io/ekshore/cargo-leptos-runner-nightly:latest AS builder

WORKDIR /build

# Install Bun for handling daisyui and other dependencies
RUN curl -fsSL https://bun.sh/install | bash
ENV PATH="/root/.bun/bin:$PATH"

COPY . .

# Install dependencies using Bun (including daisyui)
RUN bun install

RUN cargo leptos build --release

FROM ubuntu:plucky AS runner

WORKDIR /app

# Install yt-dlp and dependencies
RUN apt-get update && apt-get install -y \
    python3 \
    python3-pip \
    ffmpeg \
    curl \
    ca-certificates \
    git \
    && pip3 install --break-system-packages git+https://github.com/yt-dlp/yt-dlp.git \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/server /app/server
COPY --from=builder /build/target/site /app/site

RUN mkdir -p /home && \
    useradd -ms /bin/bash app && \
    chown app:app /home/app && \
    chmod 755 /home/app
USER app

ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_SITE_ROOT="site"

EXPOSE 3000

CMD [ "/app/server" ]
