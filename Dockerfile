FROM ghcr.io/ekshore/cargo-leptos-runner-nightly:latest AS builder

WORKDIR /build

COPY . .

RUN cargo leptos build --release

FROM ubuntu:plucky AS runner

WORKDIR /app

COPY --from=builder /build/target/release/server /app/server
COPY --from=builder /build/target/site /app/site

RUN useradd -ms /bin/bash app
USER app

ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_SITE_ROOT="site"

EXPOSE 3000

CMD [ "/app/server" ]
