-- PostgreSQL initialization script for Kafka Alerts system
-- This script creates the necessary tables and initial data for alert_server database

-- Create extension for UUID generation
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create kafka_configs table
CREATE TABLE IF NOT EXISTS kafka_configs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,
    bootstrap_servers VARCHAR(1000) NOT NULL,
    topic VARCHAR(255) NOT NULL,
    group_id VARCHAR(255) NOT NULL,
    message_timeout_ms INTEGER NOT NULL DEFAULT 5000,
    request_timeout_ms INTEGER NOT NULL DEFAULT 5000,
    retry_backoff_ms INTEGER NOT NULL DEFAULT 100,
    retries INTEGER NOT NULL DEFAULT 3,
    auto_offset_reset VARCHAR(50) NOT NULL DEFAULT 'earliest',
    enable_auto_commit BOOLEAN NOT NULL DEFAULT true,
    auto_commit_interval_ms INTEGER NOT NULL DEFAULT 1000,
    is_active BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create index for faster queries
CREATE INDEX IF NOT EXISTS idx_kafka_configs_active ON kafka_configs (is_active);
CREATE INDEX IF NOT EXISTS idx_kafka_configs_name ON kafka_configs (name);

-- Insert default Kafka configuration
INSERT INTO kafka_configs (
    name,
    bootstrap_servers,
    topic,
    group_id,
    message_timeout_ms,
    request_timeout_ms,
    retry_backoff_ms,
    retries,
    auto_offset_reset,
    enable_auto_commit,
    auto_commit_interval_ms,
    is_active
) VALUES (
    'default',
    '10.26.64.224:9093',
    'alerts',
    'alerts-consumer-group',
    5000,
    5000,
    100,
    3,
    'earliest',
    true,
    1000,
    true
) ON CONFLICT (name) DO NOTHING;

-- Insert development Kafka configuration
INSERT INTO kafka_configs (
    name,
    bootstrap_servers,
    topic,
    group_id,
    message_timeout_ms,
    request_timeout_ms,
    retry_backoff_ms,
    retries,
    auto_offset_reset,
    enable_auto_commit,
    auto_commit_interval_ms,
    is_active
) VALUES (
    'development',
    'localhost:9092',
    'alerts-dev',
    'alerts-dev-group',
    5000,
    5000,
    100,
    3,
    'earliest',
    true,
    1000,
    false
) ON CONFLICT (name) DO NOTHING;

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE 'plpgsql';

-- Create trigger to automatically update updated_at
DROP TRIGGER IF EXISTS update_kafka_configs_updated_at ON kafka_configs;
CREATE TRIGGER update_kafka_configs_updated_at
    BEFORE UPDATE ON kafka_configs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();