-- ClickHouse initialization script for alert storage
-- This script creates the necessary tables for storing EDR, NGAV and common alert data

-- Create database if not exists
CREATE DATABASE IF NOT EXISTS alerts;

USE alerts;

-- Common alerts table with shared fields from EDR and NGAV
CREATE TABLE IF NOT EXISTS alerts.common_alerts (
    id String DEFAULT generateUUIDv4(),
    original_id String,
    data_type Enum8('edr' = 1, 'ngav' = 2, 'other' = 3),
    create_time DateTime64(3),
    device_id UInt64,
    device_name String,
    device_os String,
    device_internal_ip String,
    device_external_ip String,
    org_key String,
    severity UInt8,
    alert_type String,
    threat_category String,
    device_username String,
    raw_data String,
    processed_time DateTime64(3) DEFAULT now64(),
    kafka_topic String,
    kafka_partition UInt32,
    kafka_offset UInt64,
    kafka_config_name String
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(create_time)
ORDER BY (processed_time, id)
TTL create_time + INTERVAL 365 DAY;

-- EDR specific alerts table
CREATE TABLE IF NOT EXISTS alerts.edr_alerts (
    id String,
    schema UInt32,
    create_time DateTime64(3),
    device_external_ip String,
    device_id UInt64,
    device_internal_ip String,
    device_name String,
    device_os String,
    ioc_hit String,
    ioc_id String,
    org_key String,
    parent_cmdline String,
    parent_guid String,
    parent_hash Array(String),
    parent_path String,
    parent_pid UInt32,
    parent_publisher Array(String),
    parent_reputation String,
    parent_username String,
    process_cmdline String,
    process_guid String,
    process_hash Array(String),
    process_path String,
    process_pid UInt32,
    process_publisher Array(String),
    process_reputation String,
    process_username String,
    report_id String,
    report_name String,
    report_tags Array(String),
    severity UInt8,
    alert_type String,
    watchlists Array(String),
    processed_time DateTime64(3) DEFAULT now64(),
    kafka_topic String,
    kafka_partition UInt32,
    kafka_offset UInt64
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(create_time)
ORDER BY (create_time, device_id, report_id)
TTL create_time + INTERVAL 365 DAY;

-- NGAV specific alerts table
CREATE TABLE IF NOT EXISTS alerts.ngav_alerts (
    id String,
    alert_type String,
    legacy_alert_id String,
    org_key String,
    create_time DateTime64(3),
    last_update_time DateTime64(3),
    first_event_time DateTime64(3),
    last_event_time DateTime64(3),
    threat_id String,
    severity UInt8,
    category String,
    device_id UInt64,
    device_os String,
    device_os_version String,
    device_name String,
    device_username String,
    policy_id UInt64,
    policy_name String,
    target_value String,
    workflow_state String,
    workflow_remediation String,
    workflow_last_update_time DateTime64(3),
    workflow_comment String,
    workflow_changed_by String,
    device_internal_ip String,
    device_external_ip String,
    alert_url String,
    reason String,
    reason_code String,
    process_name String,
    device_location String,
    created_by_event_id String,
    threat_indicators Array(String),
    threat_cause_actor_sha256 String,
    threat_cause_actor_name String,
    threat_cause_actor_process_pid String,
    threat_cause_reputation String,
    threat_cause_threat_category String,
    threat_cause_vector String,
    threat_cause_cause_event_id String,
    blocked_threat_category String,
    not_blocked_threat_category String,
    kill_chain_status Array(String),
    run_state String,
    policy_applied String,
    processed_time DateTime64(3) DEFAULT now64(),
    kafka_topic String,
    kafka_partition UInt32,
    kafka_offset UInt64
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(create_time)
ORDER BY (create_time, device_id, threat_id)
TTL create_time + INTERVAL 365 DAY;

-- Create indexes for better query performance
-- Common alerts indexes
CREATE INDEX IF NOT EXISTS idx_common_alerts_device_name ON alerts.common_alerts (device_name) TYPE bloom_filter(0.01);
CREATE INDEX IF NOT EXISTS idx_common_alerts_severity ON alerts.common_alerts (severity) TYPE minmax;
CREATE INDEX IF NOT EXISTS idx_common_alerts_data_type ON alerts.common_alerts (data_type) TYPE set(0);

-- EDR alerts indexes
CREATE INDEX IF NOT EXISTS idx_edr_alerts_device_name ON alerts.edr_alerts (device_name) TYPE bloom_filter(0.01);
CREATE INDEX IF NOT EXISTS idx_edr_alerts_severity ON alerts.edr_alerts (severity) TYPE minmax;
CREATE INDEX IF NOT EXISTS idx_edr_alerts_report_name ON alerts.edr_alerts (report_name) TYPE bloom_filter(0.01);
CREATE INDEX IF NOT EXISTS idx_edr_alerts_process_path ON alerts.edr_alerts (process_path) TYPE bloom_filter(0.01);

-- NGAV alerts indexes
CREATE INDEX IF NOT EXISTS idx_ngav_alerts_device_name ON alerts.ngav_alerts (device_name) TYPE bloom_filter(0.01);
CREATE INDEX IF NOT EXISTS idx_ngav_alerts_severity ON alerts.ngav_alerts (severity) TYPE minmax;
CREATE INDEX IF NOT EXISTS idx_ngav_alerts_category ON alerts.ngav_alerts (category) TYPE bloom_filter(0.01);
CREATE INDEX IF NOT EXISTS idx_ngav_alerts_threat_category ON alerts.ngav_alerts (threat_cause_threat_category) TYPE bloom_filter(0.01);