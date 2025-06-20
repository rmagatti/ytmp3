# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a YouTube to MP3 converter web application built with the Leptos framework using Axum as the backend server. The project uses `cargo-leptos` for build tooling and follows a full-stack Rust architecture with server-side rendering (SSR) and client-side hydration.

The application allows users to enter a YouTube URL, converts the video to MP3 format using yt-dlp and ffmpeg, and provides the MP3 file for download.

## Development Commands

### Development Server
```bash
cargo leptos watch
```
Starts the development server with hot reload on http://127.0.0.1:3000

### Building
```bash
cargo leptos build --release
```
Builds the production version. Output goes to `target/server/release` (server binary) and `target/site` (static assets).

### Testing
```bash
cargo leptos end-to-end
cargo leptos end-to-end --release
```
Runs Playwright end-to-end tests located in `end2end/tests/`. Make sure to run `npm install` in the `end2end/` directory first.

### Prerequisites
- Rust nightly toolchain (configured via rust-toolchain.toml)
- `cargo install cargo-leptos --locked`
- `rustup target add wasm32-unknown-unknown`
- `npm install -g sass` (for SCSS compilation)
- `yt-dlp` (for YouTube video downloading)
- `ffmpeg` (for audio conversion)

## Architecture

### Core Structure
- **src/app.rs**: Main application component with routing and UI logic
- **src/main.rs**: Server entry point (Axum server setup)
- **src/lib.rs**: Library entry point with hydration function for WASM

### Build Features
- **SSR feature**: Server-side rendering with Axum backend
- **Hydrate feature**: Client-side WASM bundle for hydration
- Dual compilation targets: server binary + WASM client bundle

### Key Configuration
- Uses Leptos 0.8.0 with nightly features
- SCSS styling compiled from `style/main.scss`
- Static assets served from `public/` directory
- Development server on port 3000, reload port 3001

### Deployment Structure
Production deployment requires:
1. Server binary from `target/server/release`
2. Site directory from `target/site`
3. Environment variables: `LEPTOS_OUTPUT_NAME`, `LEPTOS_SITE_ROOT`, `LEPTOS_SITE_PKG_DIR`, `LEPTOS_SITE_ADDR`, `LEPTOS_RELOAD_PORT`

## Docker Deployment

### Building Docker Image
```bash
docker build -t ytmp3 .
```

### Running Locally with Docker
```bash
docker run -p 3000:3000 ytmp3
```

### Railway Deployment
The project is configured for Railway deployment with:
- `Dockerfile` with multi-stage build
- `railway.toml` configuration
- `.dockerignore` for optimized builds

Required system dependencies in production:
- `yt-dlp` (for YouTube video downloading and audio conversion)
- `python3` and `pip3` (for yt-dlp installation)

The Docker image includes all necessary dependencies and is ready for deployment to Railway or any Docker-compatible platform.