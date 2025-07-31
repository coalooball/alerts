# Kafka Alerts System

A Rust-based cybersecurity alert processing system that uses Apache Kafka for message streaming. The application processes security alerts from various sources (EDR, NGAV, DNS, Sysmon, etc.) and demonstrates producer-consumer patterns for alert management.

## Features

- **Multi-format Alert Processing**: Supports various cybersecurity alert formats (EDR, NGAV, generic alerts)
- **Kafka Integration**: Full producer-consumer implementation with configurable settings
- **Rate-controlled Sending**: Precise control over message transmission rates
- **Async Processing**: High-performance async/await patterns using Tokio
- **Comprehensive Logging**: Detailed logging with configurable verbosity levels

## Quick Start

### Prerequisites

- Rust 1.70+ 
- Apache Kafka running on localhost:9092
- Security datasets (optional, sample data will be generated if not available)

### Build the Project

```bash
# Build all binaries
cargo build

# Build specific binary
cargo build --bin kafka-sender
```

### Run the Main Application

```bash
# Run with auto-detection of alert files
cargo run

# Run Kafka connection test
cargo run --bin test_kafka
```

## Command Line Tools

### kafka-sender

A standalone binary for sending JSONL data to Kafka with configurable parameters.

#### Usage

```bash
kafka-sender [OPTIONS] --data <DATA> --data-type <DATA_TYPE>
```

#### Options

- `-c, --config <CONFIG>` - Path to configuration file (default: config.toml)
- `-d, --data <DATA>` - Path to JSONL data file (required)
- `-t, --data-type <DATA_TYPE>` - Data type: alert, edr, ngav (required)
- `-r, --rate <RATE>` - Send rate in messages per second (default: 10)
- `-m, --max-messages <MAX_MESSAGES>` - Maximum messages to send (0 = unlimited)
- `-v, --verbose` - Enable verbose logging

#### Examples

```bash
# Send generic alert data at 5 messages/second
./target/debug/kafka-sender \
  --data test_alerts.jsonl \
  --data-type alert \
  --rate 5

# Send EDR alerts with rate limiting and message count limit
./target/debug/kafka-sender \
  --config config.toml \
  --data atlasv2/data/attack/h1/cbc-edr-alerts/edr-alerts-h1-m1.jsonl \
  --data-type edr \
  --rate 20 \
  --max-messages 100 \
  --verbose

# Send NGAV alerts at high rate
./target/debug/kafka-sender \
  --data atlasv2/data/attack/h1/cbc-ngav-alerts/ngav-alerts-h1-m1.jsonl \
  --data-type ngav \
  --rate 50
```

### kafka-consumer

A standalone binary for consuming and displaying Kafka messages with automatic format detection and formatting options.

#### Usage

```bash
kafka-consumer [OPTIONS]
```

#### Options

- `-c, --config <CONFIG>` - Path to configuration file (default: config.toml)
- `-g, --group-id <GROUP_ID>` - Consumer group ID (overrides config file)
- `-t, --topic <TOPIC>` - Kafka topic to consume from (overrides config file)
- `-o, --offset-reset <OFFSET_RESET>` - Auto offset reset: earliest, latest, none (default: earliest)
- `-p, --pretty` - Pretty print JSON messages
- `-d, --metadata` - Show message metadata (offset, partition, timestamp)
- `-m, --max-messages <MAX_MESSAGES>` - Maximum messages to consume (0 = unlimited)
- `-v, --verbose` - Enable verbose logging

#### Examples

```bash
# Basic consumption with default settings
./target/debug/kafka-consumer

# Pretty print JSON messages
./target/debug/kafka-consumer --pretty

# Show message metadata with pretty printing
./target/debug/kafka-consumer --metadata --pretty

# Consume from specific topic and consumer group
./target/debug/kafka-consumer \
  --topic alerts \
  --group-id my-consumer-group \
  --pretty

# Limit consumption to 100 messages with metadata
./target/debug/kafka-consumer \
  --max-messages 100 \
  --metadata \
  --pretty \
  --verbose

# Consume from latest offset (skip existing messages)
./target/debug/kafka-consumer \
  --offset-reset latest \
  --pretty
```

## Data Types Supported

### Generic Alerts (alert)
```json
{"id": "alert-001", "level": "critical", "message": "Database connection failed", "timestamp": 1234567890}
```

### EDR Alerts (edr)
Carbon Black EDR alerts with detailed process information, threat indicators, and device metadata.

### NGAV Alerts (ngav)  
Carbon Black NGAV alerts with threat categorization, MITRE ATT&CK mapping, and policy enforcement status.

## Configuration

The system uses `config.toml` for Kafka configuration:

```toml
[kafka]
bootstrap_servers = "localhost:9092"
topic = "alerts"
group_id = "alerts-consumer-group"

[kafka.producer]
message_timeout_ms = 5000
request_timeout_ms = 5000
retry_backoff_ms = 100
retries = 3

[kafka.consumer]
auto_offset_reset = "earliest"
enable_auto_commit = true
auto_commit_interval_ms = 1000
```

## Dataset Structure

The repository includes cybersecurity datasets in `atlasv2/data/attack/`:

- **h1/h2**: Different attack scenarios  
- **Data Sources**: EDR alerts, NGAV alerts, DNS logs, Firefox logs, Sysmon events
- **File Formats**: JSONL for alerts, XML for system logs
- **Naming Convention**: `{source}-{scenario}-{variant}.{ext}`

## Development

### Commands

```bash
# Quick syntax check
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy

# Run tests
cargo test
```

### Architecture

- **Core Alert Types**: Generic AlertMessage, EdrAlert, NgavAlert with specialized methods
- **Kafka Infrastructure**: Modular producer/consumer with auto-detection capabilities  
- **Async Processing**: Tokio-based concurrent operations
- **Configuration Management**: TOML-based configuration with fallback defaults

## Runtime Requirements

- Running Kafka instance on localhost:9092
- The application will fall back to sample alerts if dataset files are not found
- Consumer runs in background task while producer sends alerts in main thread

## License

This project is for educational and defensive security purposes only.