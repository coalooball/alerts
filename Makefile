# Kafka Alerts Web Server Makefile
# Quick build and development targets

.PHONY: all build frontend backend clean dev start stop install-deps test fmt lint help

# Default target
all: build

# Build everything (frontend + backend)
build: frontend backend

# Frontend targets
frontend: install-frontend-deps build-frontend

install-frontend-deps:
	@echo "📦 Installing frontend dependencies..."
	cd frontend && npm install

build-frontend:
	@echo "🔨 Building frontend..."
	cd frontend && npm run build

dev-frontend:
	@echo "🚀 Starting frontend development server..."
	cd frontend && npm run dev

# Backend targets
backend: build-backend

build-backend:
	@echo "🦀 Building Rust backend..."
	cargo build --release

dev-backend:
	cargo build

# Combined development
dev: dev-backend build-frontend

# Server management
start: build
	@echo "🌐 Starting web server..."
	cargo run --bin web-server

start-dev: dev
	@echo "🌐 Starting web server (development build)..."
	cargo run --bin web-server

stop:
	@echo "🛑 Stopping web server..."
	pkill -f web-server || true

restart: stop start

# Kafka tools
kafka-producer:
	@echo "📤 Starting Kafka producer..."
	cargo run --bin kafka-sender

kafka-consumer:
	@echo "📥 Starting Kafka consumer..."
	cargo run --bin kafka-consumer

# Development tools
test:
	@echo "🧪 Running tests..."
	cargo test

fmt:
	@echo "📝 Formatting code..."
	cargo fmt

lint:
	@echo "🔍 Running linter..."
	cargo clippy

check:
	@echo "✅ Quick syntax check..."
	cargo check

# Cleanup
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	rm -rf frontend/dist
	rm -rf frontend/node_modules

clean-frontend:
	@echo "🧹 Cleaning frontend build..."
	rm -rf frontend/dist
	rm -rf frontend/node_modules

# Database setup
setup-db:
	@echo "🐘 Setting up PostgreSQL database..."
	@echo "1. Make sure PostgreSQL is running at 10.26.64.224:5432"
	@echo "2. Create 'alert_server' database if it doesn't exist:"
	@echo "   psql -U postgres -h 10.26.64.224 -c \"CREATE DATABASE alert_server;\""
	@echo "3. The web server will automatically initialize tables on first startup"

create-db:
	@echo "🗄️ Creating alert_server database..."
	psql -U postgres -h 10.26.64.224 -c "CREATE DATABASE alert_server;" || echo "Database may already exist"

# Quick development workflow
quick: dev-backend build-frontend start-dev

# Help
help:
	@echo "Available targets:"
	@echo "  all          - Build everything (default)"
	@echo "  build        - Build frontend and backend"
	@echo "  frontend     - Install deps and build frontend"
	@echo "  backend      - Build backend (release mode)"
	@echo "  dev          - Build for development"
	@echo "  start        - Start web server (release build)"
	@echo "  start-dev    - Start web server (dev build)"
	@echo "  stop         - Stop web server"
	@echo "  restart      - Restart web server"
	@echo "  quick        - Quick dev build and start"
	@echo ""
	@echo "Development tools:"
	@echo "  dev-frontend - Start frontend dev server"
	@echo "  dev-backend  - Build backend (dev mode)"
	@echo "  test         - Run tests"
	@echo "  fmt          - Format code"
	@echo "  lint         - Run linter"
	@echo "  check        - Quick syntax check"
	@echo ""
	@echo "Kafka tools:"
	@echo "  kafka-producer - Start Kafka message sender"
	@echo "  kafka-consumer - Start Kafka message consumer"
	@echo ""
	@echo "Database:"
	@echo "  setup-db     - Setup PostgreSQL database (info only)"
	@echo "  create-db    - Create alert_server database"
	@echo ""
	@echo "Cleanup:"
	@echo "  clean        - Clean all build artifacts"
	@echo "  clean-frontend - Clean only frontend build"