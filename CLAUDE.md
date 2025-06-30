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

### Code Quality
```bash
cargo clippy
cargo fmt
```
The project uses strict clippy lints that deny panic-prone patterns like `unwrap_used`, `expect_used`, and `panic`. Always use proper error handling.

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
- **src/components/**: UI components (home_page, login_page, logout_page, auth)
- **src/server/**: Server-side functionality including video conversion logic
- **src/converter.rs**: Core conversion utilities

### Build Features
- **SSR feature**: Server-side rendering with Axum backend
- **Hydrate feature**: Client-side WASM bundle for hydration
- Dual compilation targets: server binary + WASM client bundle

### Key Configuration
- Uses Leptos 0.8.0 with nightly features
- Tailwind CSS for styling (configured in `style/tailwind.css`)
- Static assets served from `public/` directory
- Development server on port 3000, reload port 3001
- Authentication integration with Supabase
- Strict clippy lints configured (denies unwrap, panic, expect, etc.)

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

### GitHub Container Registry (GHCR)

#### Automatic Building with GitHub Actions
The project includes a GitHub Actions workflow (`.github/workflows/docker.yml`) that automatically:
- Builds the Docker image on every push to main
- Pushes to GitHub Container Registry
- Tags with branch name, commit SHA, and `latest`

#### Manual Building and Pushing
```bash
# First, authenticate with GHCR
echo $GITHUB_TOKEN | docker login ghcr.io -u rmagatti --password-stdin

# Use the provided script
./build-and-push.sh rmagatti

# Or manually
docker build -t ghcr.io/rmagatti/ytmp3:latest .
docker push ghcr.io/rmagatti/ytmp3:latest
```

#### Using the Image from GHCR
```bash
docker pull ghcr.io/rmagatti/ytmp3:latest
docker run -p 3000:3000 ghcr.io/rmagatti/ytmp3:latest
```

### Railway Deployment
The project is configured for Railway deployment with:
- `Dockerfile` with multi-stage build
- `railway.toml` configuration
- `.dockerignore` for optimized builds

Required system dependencies in production:
- `yt-dlp` (for YouTube video downloading and audio extraction)
- `ffmpeg` (required by yt-dlp for audio format conversion)
- `python3` and `pip3` (for yt-dlp installation)

The Docker image includes all necessary dependencies and is ready for deployment to Railway or any Docker-compatible platform.

## Important Notes

### YouTube Access Limitations
Due to YouTube's bot detection and anti-automation measures:
- Some videos may fail to download with "Sign in to confirm you're not a bot" errors
- Age-restricted videos require authentication and cannot be downloaded
- Private or region-restricted videos will fail
- The service includes user-friendly error messages for common issues
- Anti-bot measures in place:
  - Custom user agents
  - Android player client extraction
  - Random sleep intervals between requests
  - Latest yt-dlp version

This is a limitation of YouTube's platform, not the application itself.