# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based cybersecurity alert processing system that uses Apache Kafka for message streaming. The application processes security alerts from various sources (EDR, NGAV, DNS, Sysmon, etc.) and demonstrates producer-consumer patterns for alert management.

## Key Commands

### Building and Running
- `cargo build` - Build the project
- `cargo run` - Run the main application (auto-detects and loads alert files or sends sample alerts)
- `cargo run --bin kafka-sender` - Run the standalone Kafka message sender with CLI options  
- `cargo run --bin kafka-consumer` - Run the standalone Kafka message consumer with CLI options
- `cargo run --bin web-server` - Run the Axum web server with React frontend on http://localhost:3000
- `cargo test` - Run tests (if any exist)

### Development
- `cargo check` - Quick syntax and type checking
- `cargo fmt` - Format code using rustfmt
- `cargo clippy` - Run the Clippy linter for additional checks

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

### Kafka Infrastructure

The Kafka components are organized in `src/kafka/`:

- **KafkaConfig** (`src/kafka/config.rs`): Configuration management loaded from `config.toml`
- **KafkaProducer** (`src/kafka/producer.rs`): Handles sending alerts with auto-detection of file formats
- **KafkaConsumer** (`src/kafka/consumer.rs`): Processes incoming alerts with severity-based routing

### Data Processing Flow

1. **File Auto-Detection**: The producer can automatically detect and load various alert file formats (EDR, NGAV JSONL files)
2. **Message Routing**: Consumer processes alerts differently based on severity levels with appropriate logging
3. **Async Processing**: Uses tokio for concurrent producer/consumer operations

### Configuration

Configuration is now managed through a PostgreSQL database instead of config files:
- **Database Connection**: PostgreSQL at 10.26.64.224:5432 (database: alert_server, user: postgres)  
- **Kafka Settings**: Bootstrap servers, topics, consumer groups stored in `kafka_configs` table
- **Producer/Consumer Options**: Timeouts, retry logic, offset management configurable per configuration
- **Active Configuration**: Only one configuration can be active at a time
- **Fallback**: Uses default configuration if no database config is found

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

## CLI Tools

### kafka-sender Binary
Standalone tool for sending JSONL data to Kafka with rate control:
- `--data <file>` - JSONL data file path
- `--data-type <type>` - Data type: alert, edr, ngav
- `--rate <n>` - Messages per second (default: 10)
- `--max-messages <n>` - Message limit (0 = unlimited)
- `--config <file>` - Configuration file (default: config.toml)
- `--verbose` - Enable verbose logging

### kafka-consumer Binary
Standalone tool for consuming and displaying Kafka messages:
- `--pretty` - Pretty print JSON messages
- `--metadata` - Show message metadata (offset, partition, timestamp)
- `--max-messages <n>` - Message limit (0 = unlimited)
- `--group-id <id>` - Consumer group ID override
- `--topic <topic>` - Topic name override
- `--offset-reset <mode>` - earliest, latest, or none
- `--verbose` - Enable verbose logging

### web-server Binary
Axum-based web server with React frontend for Kafka management:
- Serves React frontend on http://localhost:3000
- API endpoints for Kafka connectivity testing
- Web interface for sending/receiving Kafka messages
- Real-time message consumption and display
- Configuration management through web UI

## Web API Endpoints

The web server provides the following REST API endpoints:

### Configuration Management
- `GET /api/config` - Get current active Kafka configuration (first active)
- `GET /api/configs` - List all saved Kafka configurations  
- `GET /api/configs/active` - List all currently active configurations
- `POST /api/config` - Save a new Kafka configuration
- `PUT /api/config/:id` - Update an existing configuration
- `DELETE /api/config/:id` - Delete a configuration
- `POST /api/config/:id/activate` - Set as only active configuration (deactivate others)
- `POST /api/config/:id/toggle` - Toggle configuration active status

### Connectivity & Messaging  
- `GET /api/test-connectivity` - Test Kafka connectivity with optional custom config
- `GET /api/consume-messages` - Consume latest messages from Kafka topic

## Frontend Development

The React frontend is located in `frontend/` directory:
- `cd frontend && npm install` - Install dependencies
- `npm run dev` - Start development server (with proxy to backend)
- `npm run build` - Build for production (outputs to `frontend/dist/`)

## Runtime Requirements

- **PostgreSQL Database**: Running at 10.26.64.224:5432 (automatically creates `alert_server` database)
- **Database Schema**: Automatically initialized on first startup using `init.sql`
- **Kafka Instance**: As configured in database (default: 10.26.64.224:9093)
- **Frontend**: Built React app in `frontend/dist/` (served statically by web server)
- The application will use default configuration if database connection fails

## Database Initialization Process

When the web server starts, it automatically:

1. **Connects** to PostgreSQL at `10.26.64.224:5432`
2. **Checks and creates** `alert_server` database if it doesn't exist
3. **Connects** to the `alert_server` database
4. **Checks** if `kafka_configs` table exists
5. **Reads and executes** `init.sql` if tables don't exist (fails if `init.sql` not found)  
6. **Creates default** Kafka configurations as part of `init.sql`

## Enhanced Configuration Features

### Multiple Active Configurations
- Support for multiple simultaneously active Kafka configurations
- Toggle individual configurations on/off
- "Set as Only Active" option to activate one and deactivate all others

### Rich Configuration Management
- **Create**: Add new Kafka configurations with full parameter control
- **Read**: View all configurations with detailed information
- **Update**: Edit existing configurations while preserving active status
- **Delete**: Remove configurations with confirmation dialog

### Interactive UI Features
- **Hover Tooltips**: Mouse over active configuration badges to see detailed settings
- **Real-time Updates**: Configuration changes immediately reflect across the interface
- **Visual Status**: Clear indicators for active/inactive configurations
- **Bulk Operations**: Manage multiple configurations efficiently