# Build stage
FROM rustlang/rust:nightly AS builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    cmake \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /usr/src/app

# Copy Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create src directory with dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this will fail but cache dependencies)
RUN cargo build --release --bin web-server || true

# Remove dummy file
RUN rm -rf src

# Copy actual source code
COPY src ./src
COPY init.sql ./

# Build the actual application
RUN cargo build --release --bin web-server

# Build frontend
FROM node:20-alpine AS frontend-builder

WORKDIR /app/frontend

# Copy frontend files
COPY frontend/package*.json ./
RUN npm ci

COPY frontend ./
RUN npm run build

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/web-server ./
COPY --from=builder /usr/src/app/init.sql ./

# Copy frontend build
COPY --from=frontend-builder /app/frontend/dist ./frontend/dist

# Create non-root user
RUN useradd -m -u 1000 appuser && chown -R appuser:appuser /app
USER appuser

# Expose port (default 3000, but can be overridden)
EXPOSE 3000

# Run the web server
ENTRYPOINT ["./web-server"]