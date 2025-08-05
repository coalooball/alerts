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
    'edr',
    '127.0.0.1:9092',
    'alerts-edr',
    '1',
    5000,
    5000,
    100,
    3,
    'earliest',
    true,
    1000,
    true
) ON CONFLICT (name) DO NOTHING;

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
    'ngav',
    '127.0.0.1:9092',
    'alerts-ngav',
    '2',
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

-- Create clickhouse_config table (only one configuration allowed)
CREATE TABLE IF NOT EXISTS clickhouse_config (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL DEFAULT 'default' UNIQUE,
    host VARCHAR(255) NOT NULL,
    port INTEGER NOT NULL DEFAULT 8123,
    database_name VARCHAR(255) NOT NULL,
    username VARCHAR(255) NOT NULL DEFAULT 'default',
    password VARCHAR(255),
    use_tls BOOLEAN NOT NULL DEFAULT false,
    connection_timeout_ms INTEGER NOT NULL DEFAULT 10000,
    request_timeout_ms INTEGER NOT NULL DEFAULT 30000,
    max_connections INTEGER NOT NULL DEFAULT 10,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create index for clickhouse_config
CREATE INDEX IF NOT EXISTS idx_clickhouse_config_name ON clickhouse_config (name);

-- Insert default ClickHouse configuration
INSERT INTO clickhouse_config (
    name,
    host,
    port,
    database_name,
    username,
    password,
    use_tls,
    connection_timeout_ms,
    request_timeout_ms,
    max_connections
) VALUES (
    'default',
    '127.0.0.1',
    8123,
    'default',
    'default',
    'default',
    false,
    10000,
    30000,
    10
) ON CONFLICT (name) DO NOTHING;

-- Create trigger to automatically update updated_at for clickhouse_config
DROP TRIGGER IF EXISTS update_clickhouse_config_updated_at ON clickhouse_config;
CREATE TRIGGER update_clickhouse_config_updated_at
    BEFORE UPDATE ON clickhouse_config
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create data_source_configs table for EDR/NGAV Kafka node mappings
CREATE TABLE IF NOT EXISTS data_source_configs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    data_type VARCHAR(50) NOT NULL, -- 'edr' or 'ngav'
    kafka_config_id UUID NOT NULL REFERENCES kafka_configs(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(data_type, kafka_config_id)
);

-- Create indexes for data_source_configs
CREATE INDEX IF NOT EXISTS idx_data_source_configs_data_type ON data_source_configs (data_type);
CREATE INDEX IF NOT EXISTS idx_data_source_configs_kafka_id ON data_source_configs (kafka_config_id);

-- Create trigger to automatically update updated_at for data_source_configs
DROP TRIGGER IF EXISTS update_data_source_configs_updated_at ON data_source_configs;
CREATE TRIGGER update_data_source_configs_updated_at
    BEFORE UPDATE ON data_source_configs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Insert default data source configurations
INSERT INTO data_source_configs (data_type, kafka_config_id) 
SELECT 'edr', id FROM kafka_configs WHERE name = 'edr'
ON CONFLICT (data_type, kafka_config_id) DO NOTHING;

INSERT INTO data_source_configs (data_type, kafka_config_id) 
SELECT 'ngav', id FROM kafka_configs WHERE name = 'ngav'
ON CONFLICT (data_type, kafka_config_id) DO NOTHING;

-- Create users table for authentication
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for users table
CREATE INDEX IF NOT EXISTS idx_users_username ON users (username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users (email);
CREATE INDEX IF NOT EXISTS idx_users_active ON users (is_active);
CREATE INDEX IF NOT EXISTS idx_users_role ON users (role);

-- Create trigger to automatically update updated_at for users
DROP TRIGGER IF EXISTS update_users_updated_at ON users;
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create user_sessions table for session management
CREATE TABLE IF NOT EXISTS user_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_token VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for user_sessions table
CREATE INDEX IF NOT EXISTS idx_user_sessions_token ON user_sessions (session_token);
CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions (user_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_expires ON user_sessions (expires_at);

-- Insert default admin user (password: admin123)
-- Hash for "admin123" using bcrypt with cost 12
INSERT INTO users (
    username,
    email,
    password_hash,
    role,
    is_active
) VALUES (
    'admin',
    'admin@localhost',
    '$2b$12$H4FMivSts0pj2E/lxoEbweHT1Mdy1Y0lMqCQ7gqNsEz967wkwqRp2',
    'admin',
    true
) ON CONFLICT (username) DO NOTHING;