FROM rust:1.91-bookworm AS builder

ARG API_BASE_URL
ENV API_BASE_URL=${API_BASE_URL}

WORKDIR /app

# System deps for Rust + wasm builds
RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
	  ca-certificates curl git pkg-config libssl-dev clang \
	&& rm -rf /var/lib/apt/lists/*

# Node.js 20 (for frontend build tooling)
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
	&& apt-get update \
	&& apt-get install -y --no-install-recommends nodejs \
	&& rm -rf /var/lib/apt/lists/*

# Some Trunk hooks call `npm.cmd` (Windows shim). Provide a shim for Linux builds.
RUN printf '%s\n' '#!/usr/bin/env bash' 'exec npm "$@"' > /usr/local/bin/npm.cmd \
	&& chmod +x /usr/local/bin/npm.cmd

# Trunk + wasm target
RUN rustup target add wasm32-unknown-unknown \
	&& cargo install trunk --version 0.21.14

# Install npm deps first for better docker layer caching
COPY package.json package-lock.json ./
RUN npm ci

# Build the app
COPY . .
RUN trunk build --release

FROM nginx:alpine
COPY --from=builder /app/dist/ /usr/share/nginx/html/
EXPOSE 80
