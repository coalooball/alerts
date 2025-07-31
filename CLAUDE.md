# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based cybersecurity alert processing system that uses Apache Kafka for message streaming. The application processes security alerts from various sources (EDR, NGAV, DNS, Sysmon, etc.) and demonstrates producer-consumer patterns for alert management.

## Key Commands

### Building and Running
- `cargo build` - Build the project
- `cargo run` - Run the main application (auto-detects and loads alert files or sends sample alerts)
- `cargo run --bin test_kafka` - Run the Kafka connection test utility
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

Configuration is managed through `config.toml` with separate sections for producer and consumer settings:
- Kafka broker settings (default: localhost:9092)
- Producer timeouts and retry logic
- Consumer offset and commit settings

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

## Runtime Requirements

- Running Kafka instance on localhost:9092
- The application will fall back to sample alerts if dataset files are not found
- Consumer runs in background task while producer sends alerts in main thread