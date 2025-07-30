# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based alert processing system that uses Apache Kafka for message streaming. The application demonstrates producer-consumer patterns for alert management with different severity levels (critical, warning, info).

## Key Commands

### Building and Running
- `cargo build` - Build the project
- `cargo run` - Run the main application (sends sample alerts and consumes them)
- `cargo run --bin test_kafka` - Run the Kafka connection test utility
- `cargo test` - Run tests (if any exist)

### Development
- `cargo check` - Quick syntax and type checking
- `cargo fmt` - Format code using rustfmt
- `cargo clippy` - Run the Clippy linter for additional checks

## Architecture

### Core Components

1. **AlertMessage** (`src/alert.rs`): Core data structure representing alerts with:
   - ID, severity level, message content, and timestamp
   - Factory methods for different severity levels (critical, warning, info)
   - Serde serialization for JSON conversion

2. **KafkaProducer** (`src/kafka.rs`): Handles sending alerts to Kafka topics
   - Configurable with timeouts and retry logic
   - Supports both single alerts and batch operations
   - Uses async/await patterns with tokio

3. **KafkaConsumer** (`src/kafka.rs`): Processes incoming alerts from Kafka
   - Subscribes to the "alerts" topic by default
   - Processes alerts based on severity level with appropriate logging
   - Includes error handling for malformed messages

4. **KafkaConfig** (`src/kafka.rs`): Configuration management for Kafka connections
   - Default settings point to localhost:9092
   - Topic defaults to "alerts"
   - Consumer group defaults to "alerts-consumer-group"

### Data Flow
- Main application creates both producer and consumer instances
- Producer sends sample alerts to the Kafka topic
- Consumer runs in a separate task, processing alerts as they arrive
- Alerts are processed differently based on their severity level

### External Dependencies
- **rdkafka**: Apache Kafka client with cmake-build feature enabled
- **tokio**: Async runtime with full feature set
- **serde/serde_json**: JSON serialization
- **anyhow**: Error handling
- **log/env_logger**: Logging infrastructure
- **chrono**: Timestamp generation

### Test Infrastructure
- `test_kafka.rs`: Standalone Kafka connectivity test
- Tests basic producer functionality without the full alert system

### Dataset
The repository includes `atlasv2/data/` containing various security-related datasets (EDR alerts, DNS logs, Firefox logs, Sysmon events, etc.) organized by attack scenarios (h1, h2) and different data sources. These appear to be cybersecurity datasets for analysis or testing purposes.

## Notes
- The application requires a running Kafka instance on localhost:9092
- All alerts are timestamped using UTC
- Consumer processes messages with different log levels based on alert severity
- Error handling includes retry logic for producer operations