# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based cybersecurity alert processing system that uses Apache Kafka for message streaming and ClickHouse for alert storage. The application processes security alerts from various sources (EDR, NGAV, DNS, Sysmon, etc.) and provides a web interface for configuration management and alert monitoring.

## Key Commands

### Building and Running
- `cargo build` - Build the project
- `cargo build --release` - Build optimized release version
- `cargo run` - Run the main application (auto-detects and loads alert files or sends sample alerts)
- `cargo run --bin kafka-sender` - Run the standalone Kafka message sender with CLI options  
- `cargo run --bin kafka-consumer` - Run the standalone Kafka message consumer with CLI options
- `cargo run --bin web-server` - Run the Axum web server with React frontend on http://localhost:3000
- `cargo run --bin web-server -- --init` - Initialize database (drops and recreates `alert_server` database)
- `cargo run --bin web-server -- --port 8080` - Run web server on custom port
- `cargo run --bin hash-password <password>` - Generate bcrypt hash for user passwords
- `cargo test` - Run tests
- `cargo test <test_name>` - Run a specific test
- `make build` - Build both frontend and backend
- `make start` - Build and start the web server  
- `make quick` - Quick development build and start
- `make restart` - Stop and restart the web server

### Development
- `cargo check` - Quick syntax and type checking
- `cargo fmt` - Format code using rustfmt
- `cargo clippy` - Run the Clippy linter for additional checks
- `make dev` - Build development version (backend + frontend)
- `make dev-frontend` - Start frontend development server (Vite)
- `make dev-backend` - Build backend in development mode
- `make start-dev` - Start web server with development build
- `make init-db` - Initialize database schema (drop and recreate with release build)
- `make init-db-dev` - Initialize database schema (drop and recreate with dev build)

### Database Access
- **Direct psql access**: Use Docker container for database queries:
  ```bash
  # Using docker-compose (if postgres service is running)
  sudo docker-compose -f docker/docker-compose.pg.yml exec postgres psql -U postgres -d alert_server -c "SELECT * FROM threat_events LIMIT 5;"
  
  # Using docker exec with container name
  sudo docker exec postgres-db psql -U postgres -d alert_server -c "SELECT * FROM threat_events LIMIT 5;"
  ```
- **sudo password**: `root@123`
- **Container name**: `postgres-db`

## Authentication System

### Login Credentials
- **Username**: `admin`
- **Password**: `admin123`

### Clear Login Session (Force Re-login)
To clear your login session and return to the login page, open browser console (F12) and execute:
```javascript
localStorage.removeItem('sessionToken'); window.location.reload();
```

### Session Management
- Sessions expire after 24 hours
- Session tokens are stored in localStorage
- Logging out invalidates the session on the server
- The system requires authentication to access all features

## Architecture

### Core Alert Types

1. **AlertMessage** (`src/alert.rs`): Generic alert structure with factory methods for different severity levels (critical, warning, info)

2. **EdrAlert** (`src/edr_alert.rs`): Carbon Black EDR-specific alert structure
   - Contains detailed process information, threat indicators, and device metadata
   - Methods for severity assessment and threat analysis
   - Supports watchlist matching and IOC detection

3. **NgavAlert** (`src/ngav_alert.rs`): Carbon Black NGAV (Next-Gen Antivirus) alert structure
   - Includes threat categorization, MITRE ATT&CK mapping, and policy enforcement status
   - Methods for threat analysis, MITRE TTP extraction, and blocked threat detection

### Authentication System (`src/auth.rs`)
The web server includes a JWT-based authentication system with session management:
- User credentials stored in PostgreSQL `users` table with bcrypt-hashed passwords
- JWT tokens for session management with 24-hour expiration
- Session tracking in `sessions` table with automatic cleanup
- Protected API endpoints require valid authentication tokens

### Kafka Infrastructure

The Kafka components are organized in `src/kafka/`:

- **KafkaConfig** (`src/kafka/config.rs`): Configuration management loaded from database or `config.toml`
- **KafkaProducer** (`src/kafka/producer.rs`): Handles sending alerts with auto-detection of file formats
- **KafkaConsumer** (`src/kafka/consumer.rs`): Processes incoming alerts with severity-based routing

### Database Components

- **Database** (`src/database.rs`): PostgreSQL connection and schema management
- **ConsumerService** (`src/consumer_service.rs`): Background service for consuming Kafka messages and storing in ClickHouse
- **ClickHouseConnection** (`src/clickhouse.rs`): Alert storage and retrieval with optimized DateTime64(3) schema

### Data Processing Flow

1. **File Auto-Detection**: The producer can automatically detect and load various alert file formats (EDR, NGAV JSONL files)
2. **Message Routing**: Consumer processes alerts differently based on severity levels with appropriate logging
3. **Alert Storage**: Consumed alerts are stored in ClickHouse tables (`common_alerts`, `edr_alerts`, `ngav_alerts`)
4. **Async Processing**: Uses tokio for concurrent producer/consumer operations

### Configuration Management

Configuration is managed through a PostgreSQL database with web UI:
- **Database Connection**: PostgreSQL at 10.26.64.224:5432 (database: alert_server, user: postgres)  
- **Kafka Settings**: Bootstrap servers, topics, consumer groups stored in `kafka_configs` table
- **ClickHouse Settings**: Connection details stored in `clickhouse_config` table
- **Data Source Mappings**: EDR/NGAV to Kafka node mappings in `data_source_configs` table
- **Multiple Active Configurations**: Support for multiple simultaneously active Kafka configurations
- **Fallback**: Uses `config.toml` if no database config is found

### Security Dataset Structure

The `atlasv2/data/attack/` directory contains organized cybersecurity datasets:
- **h1/h2**: Different attack scenarios
- **Data Sources**: EDR alerts, NGAV alerts, DNS logs, Firefox logs, Sysmon events, Microsoft Security logs
- **File Formats**: JSONL for alerts, XML for system logs
- **Naming Convention**: `{source}-{scenario}-{variant}.{ext}`

## Dependencies

- **rdkafka**: Apache Kafka client with cmake-build feature
- **serde/serde_json**: JSON serialization for alert processing
- **chrono**: Timestamp handling with UTC timezone
- **tokio**: Async runtime for concurrent operations
- **anyhow**: Error handling across async operations
- **toml/tempfile**: Configuration and temporary file management
- **axum/tower**: Web server framework with middleware support
- **sqlx**: Async PostgreSQL driver with compile-time checked queries
- **clickhouse**: ClickHouse client for alert storage
- **uuid**: UUID generation for database records

## CLI Tools

### kafka-sender Binary
Standalone tool for sending JSONL data to Kafka with rate control:
- `--bootstrap-servers <servers>` - Kafka bootstrap servers (required)
- `--topic <topic>` - Kafka topic to send to (required)
- `--group-id <id>` - Consumer group ID (required)
- `--data <file>` - JSONL data file path (required)
- `--data-type <type>` - Data type: alert, edr, ngav (required)
- `--rate <n>` - Messages per second (default: 10)
- `--max-messages <n>` - Message limit (0 = unlimited)
- `--config <file>` - Configuration file (default: config.toml)
- `--verbose` - Enable verbose logging

Example usage:
```bash
cargo run --bin kafka-sender -- \
  --bootstrap-servers 10.26.64.224:9093 \
  --topic alerts-edr \
  --group-id 1 \
  --data atlasv2/data/attack/h1/cbc-edr-alerts/edr-alerts-h1-m1.jsonl \
  --data-type edr \
  --rate 50
```

### kafka-consumer Binary
Standalone tool for consuming and displaying Kafka messages:
- `--config <file>` - Configuration file (default: config.toml)
- `--pretty` - Pretty print JSON messages
- `--metadata` - Show message metadata (offset, partition, timestamp)
- `--max-messages <n>` - Message limit (0 = unlimited)
- `--group-id <id>` - Consumer group ID override
- `--topic <topic>` - Topic name override
- `--offset-reset <mode>` - earliest, latest, or none (default: earliest)
- `--verbose` - Enable verbose logging

Example usage:
```bash
# Basic consumption with pretty printing
cargo run --bin kafka-consumer -- --pretty --metadata

# Consume specific topic with message limit
cargo run --bin kafka-consumer -- \
  --topic alerts-edr \
  --group-id consumer-group-1 \
  --max-messages 100 \
  --pretty
```

### web-server Binary
Axum-based web server with React frontend for comprehensive alert management:
- Serves React frontend on http://localhost:3000 (default)
- API endpoints for Kafka/ClickHouse connectivity testing
- Configuration management through database-backed web UI
- Real-time message consumption with ClickHouse storage
- Background consumer service for automated alert processing
- `--init` flag for database initialization (drops and recreates schema)
- `-p, --port <PORT>` - Specify port to listen on (default: 3000)

## Web API Endpoints

The web server provides the following REST API endpoints (all require authentication except `/api/auth/*`):

### Configuration Management
- `GET /api/config` - Get current active Kafka configuration (first active)
- `GET /api/configs` - List all saved Kafka configurations  
- `GET /api/configs/active` - List all currently active configurations
- `POST /api/config` - Save a new Kafka configuration
- `PUT /api/config/:id` - Update an existing configuration
- `DELETE /api/config/:id` - Delete a configuration
- `POST /api/config/:id/activate` - Set as only active configuration (deactivate others)
- `POST /api/config/:id/toggle` - Toggle configuration active status

### ClickHouse Configuration
- `GET /api/clickhouse-config` - Get current ClickHouse configuration
- `POST /api/clickhouse-config` - Save/update ClickHouse configuration
- `DELETE /api/clickhouse-config` - Delete ClickHouse configuration
- `GET /api/clickhouse/test` - Test ClickHouse connectivity
- `GET /api/clickhouse/init` - Initialize ClickHouse tables

### Data Source Configuration
- `GET /api/data-source-configs` - List all data source configurations
- `POST /api/data-source-config` - Save data source to Kafka mappings
- `DELETE /api/data-source-config/:data_type` - Delete data source configuration

### Connectivity & Messaging  
- `GET /api/test-connectivity` - Test Kafka connectivity with optional custom config
- `GET /api/consume-messages` - Consume latest messages from Kafka topic
- `GET /api/consumer-service/status` - Get consumer service status
- `POST /api/consumer-service/start` - Start background consumer service
- `POST /api/consumer-service/stop` - Stop background consumer service

### Authentication
- `POST /api/auth/login` - Login with username/password, returns JWT token
- `POST /api/auth/logout` - Logout and invalidate session
- `GET /api/auth/me` - Get current user information

## Frontend Development

The React frontend is located in `frontend/` directory:
- `cd frontend && npm install` - Install dependencies
- `npm run dev` - Start development server (with proxy to backend)
- `npm run build` - Build for production (outputs to `frontend/dist/`)
- Frontend provides UI for Kafka/ClickHouse configuration, connectivity testing, and real-time alert monitoring

## Runtime Requirements

- **PostgreSQL Database**: Running at 10.26.64.224:5432 (automatically creates `alert_server` database)
- **Database Schema**: Automatically initialized on first startup using `init.sql`
- **Kafka Instance**: As configured in database (default: 10.26.64.224:9093)
- **ClickHouse Instance**: As configured in database (default: 10.26.64.224:8123)
- **Frontend**: Built React app in `frontend/dist/` (served statically by web server)
- The application will use default configuration if database connection fails

## Docker Support

The project includes Docker configuration for easy deployment:
- `docker-compose.yml` - Full stack deployment with Kafka, ClickHouse, and PostgreSQL
- `docker-compose.init.yml` - Initial setup for database services
- `docker/` directory contains service-specific Docker Compose files:
  - `docker-compose.kafka.yml` - Kafka cluster setup
  - `docker-compose.clickhouse.yml` - ClickHouse configuration
  - `docker-compose.pg.yml` - PostgreSQL database setup

## Database Initialization Process

When the web server starts, it automatically:

1. **Connects** to PostgreSQL at `10.26.64.224:5432`
2. **Checks and creates** `alert_server` database if it doesn't exist
3. **Connects** to the `alert_server` database
4. **Checks** if tables exist (`kafka_configs`, `clickhouse_config`, `data_source_configs`)
5. **Reads and executes** `init.sql` if tables don't exist (fails if `init.sql` not found)  
6. **Creates default** Kafka and ClickHouse configurations as part of `init.sql`

To completely reinitialize the database (drop and recreate):
```bash
cargo run --bin web-server -- --init
# or
make init-db
```

## ClickHouse Schema

The system stores alerts in ClickHouse with three main tables:

1. **common_alerts**: Unified view of all alerts with common fields
2. **edr_alerts**: EDR-specific alert details with process information
3. **ngav_alerts**: NGAV-specific alert details with threat indicators

All tables use:
- DateTime64(3) for timestamp fields with millisecond precision
- Partitioning by month for efficient queries
- TTL of 365 days for automatic data cleanup
- Bloom filter indexes for efficient string searches

## Enhanced Configuration Features

### Multiple Active Configurations
- Support for multiple simultaneously active Kafka configurations
- Toggle individual configurations on/off
- "Set as Only Active" option to activate one and deactivate all others
- Data source to Kafka node mapping for EDR/NGAV routing

### Rich Configuration Management
- **Create**: Add new Kafka/ClickHouse configurations with full parameter control
- **Read**: View all configurations with detailed information
- **Update**: Edit existing configurations while preserving active status
- **Delete**: Remove configurations with confirmation dialog

### Interactive UI Features
- **Hover Tooltips**: Mouse over active configuration badges to see detailed settings
- **Real-time Updates**: Configuration changes immediately reflect across the interface
- **Visual Status**: Clear indicators for active/inactive configurations, consumer service status
- **Alert Monitoring**: View processed alerts with filtering and search capabilities
- **Bulk Operations**: Manage multiple configurations efficiently

### Background Consumer Service
- Automatically consumes messages from configured Kafka topics
- Stores alerts in ClickHouse with proper type mapping
- Supports multiple active Kafka configurations simultaneously
- Real-time status monitoring through web UI
- Graceful start/stop with error recovery

## File Structure

Key directories and files:
- `src/` - Rust source code
  - `alert.rs` - Generic alert structure with factory methods
  - `edr_alert.rs` - Carbon Black EDR alert definitions  
  - `ngav_alert.rs` - Carbon Black NGAV alert definitions
  - `auth.rs` - Authentication service with JWT and bcrypt
  - `kafka/` - Kafka producer/consumer implementation
  - `database.rs` - PostgreSQL connection and configuration management
  - `clickhouse.rs` - ClickHouse client for alert storage
  - `consumer_service.rs` - Background service for message processing
  - `bin/` - Executable binaries (kafka-sender, kafka-consumer, web-server, hash-password)
- `frontend/` - React web interface with Vite build system
- `atlasv2/data/attack/` - Cybersecurity datasets organized by scenarios
- `init.sql` - PostgreSQL database initialization script (includes user tables)
- `clickhouse_init.sql` - ClickHouse table creation script
- `config.toml` - Default Kafka configuration (fallback)
- `Makefile` - Build automation for frontend/backend