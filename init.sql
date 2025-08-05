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
    full_name VARCHAR(255),
    phone VARCHAR(50),
    department VARCHAR(255),
    timezone VARCHAR(100) DEFAULT 'UTC',
    language VARCHAR(10) DEFAULT 'en',
    email_notifications BOOLEAN DEFAULT true,
    sms_notifications BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for users table
CREATE INDEX IF NOT EXISTS idx_users_username ON users (username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users (email);
CREATE INDEX IF NOT EXISTS idx_users_active ON users (is_active);
CREATE INDEX IF NOT EXISTS idx_users_role ON users (role);

-- Add profile columns to existing users table if they don't exist
ALTER TABLE users ADD COLUMN IF NOT EXISTS full_name VARCHAR(255);
ALTER TABLE users ADD COLUMN IF NOT EXISTS phone VARCHAR(50);
ALTER TABLE users ADD COLUMN IF NOT EXISTS department VARCHAR(255);
ALTER TABLE users ADD COLUMN IF NOT EXISTS timezone VARCHAR(100) DEFAULT 'UTC';
ALTER TABLE users ADD COLUMN IF NOT EXISTS language VARCHAR(10) DEFAULT 'en';
ALTER TABLE users ADD COLUMN IF NOT EXISTS email_notifications BOOLEAN DEFAULT true;
ALTER TABLE users ADD COLUMN IF NOT EXISTS sms_notifications BOOLEAN DEFAULT false;

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

-- ================================
-- 威胁事件管理相关表结构
-- ================================

-- ================================
-- 1. 威胁事件主表 (threat_events)
-- ================================
CREATE TABLE IF NOT EXISTS threat_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- 基本信息
    title VARCHAR(500) NOT NULL,                          -- 威胁事件标题
    description TEXT,                                      -- 详细描述
    event_type VARCHAR(100) NOT NULL,                     -- 事件类型 (malware, persistence, lateral_movement, data_exfiltration, etc.)
    severity INTEGER NOT NULL CHECK (severity BETWEEN 1 AND 5), -- 严重程度 1=critical, 2=high, 3=medium, 4=low, 5=info
    threat_category VARCHAR(100),                          -- 威胁类别 (APT, ransomware, trojan, etc.)
    
    -- 时间信息
    event_start_time TIMESTAMP WITH TIME ZONE,            -- 威胁事件开始时间
    event_end_time TIMESTAMP WITH TIME ZONE,              -- 威胁事件结束时间
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),    -- 记录创建时间
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),    -- 记录更新时间
    
    -- MITRE ATT&CK 信息
    mitre_tactics JSONB,                                   -- MITRE战术 ['initial_access', 'execution', ...]
    mitre_techniques JSONB,                                -- MITRE技术 ['T1566', 'T1059', ...]
    kill_chain_phases JSONB,                               -- 杀伤链阶段
    
    -- 影响范围
    affected_devices JSONB,                                -- 受影响设备列表 [{'device_id': 123, 'device_name': 'WIN-32-H1', ...}]
    affected_users JSONB,                                  -- 受影响用户列表
    affected_processes JSONB,                              -- 涉及进程信息
    
    -- 威胁指标
    iocs JSONB,                                           -- IOCs (IP, domain, hash, etc.)
    threat_actors JSONB,                                   -- 威胁行为者信息
    
    -- 处理状态
    status VARCHAR(50) NOT NULL DEFAULT 'open',           -- 处理状态 (open, investigating, resolved, false_positive)
    priority VARCHAR(20) DEFAULT 'medium',                -- 优先级 (critical, high, medium, low)
    assigned_to UUID REFERENCES users(id),                -- 分配给的用户
    resolution_notes TEXT,                                 -- 处理备注
    
    -- 元数据
    creation_method VARCHAR(20) NOT NULL DEFAULT 'manual', -- 创建方式 (auto, manual)
    data_sources JSONB,                                    -- 数据源列表 ['edr', 'ngav', 'sysmon', ...]
    confidence_score DECIMAL(3,2) CHECK (confidence_score BETWEEN 0 AND 1.0), -- 置信度
    
    -- 索引字段
    org_key VARCHAR(50),                                   -- 组织标识
    tags JSONB,                                           -- 标签列表
    
    -- 审计字段
    created_by UUID REFERENCES users(id),                 -- 创建人
    updated_by UUID REFERENCES users(id)                  -- 最后更新人
);

-- 创建威胁事件索引
CREATE INDEX IF NOT EXISTS idx_threat_events_severity ON threat_events (severity);
CREATE INDEX IF NOT EXISTS idx_threat_events_status ON threat_events (status);
CREATE INDEX IF NOT EXISTS idx_threat_events_event_type ON threat_events (event_type);
CREATE INDEX IF NOT EXISTS idx_threat_events_created_at ON threat_events (created_at);
CREATE INDEX IF NOT EXISTS idx_threat_events_event_start_time ON threat_events (event_start_time);
CREATE INDEX IF NOT EXISTS idx_threat_events_assigned_to ON threat_events (assigned_to);
CREATE INDEX IF NOT EXISTS idx_threat_events_org_key ON threat_events (org_key);
CREATE INDEX IF NOT EXISTS idx_threat_events_creation_method ON threat_events (creation_method);

-- GIN索引用于JSONB字段查询
CREATE INDEX IF NOT EXISTS idx_threat_events_mitre_techniques ON threat_events USING GIN (mitre_techniques);
CREATE INDEX IF NOT EXISTS idx_threat_events_affected_devices ON threat_events USING GIN (affected_devices);
CREATE INDEX IF NOT EXISTS idx_threat_events_iocs ON threat_events USING GIN (iocs);
CREATE INDEX IF NOT EXISTS idx_threat_events_tags ON threat_events USING GIN (tags);

-- ================================
-- 2. 告警数据存储表 (alert_data)
-- ================================
CREATE TABLE IF NOT EXISTS alert_data (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- 告警基本信息
    alert_id VARCHAR(255) NOT NULL,                       -- 原始告警ID
    alert_type VARCHAR(50) NOT NULL,                      -- 告警类型 (edr, ngav, sysmon, etc.)
    source_system VARCHAR(100) NOT NULL,                  -- 来源系统
    
    -- 告警内容
    raw_data JSONB NOT NULL,                             -- 原始告警数据(完整JSON)
    parsed_data JSONB,                                    -- 解析后的标准化数据
    
    -- 时间信息
    alert_timestamp TIMESTAMP WITH TIME ZONE NOT NULL,    -- 告警时间
    ingestion_time TIMESTAMP WITH TIME ZONE DEFAULT NOW(), -- 数据摄入时间
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- 设备信息 (从告警数据提取)
    device_id BIGINT,                                     -- 设备ID
    device_name VARCHAR(255),                             -- 设备名称
    device_os VARCHAR(100),                               -- 操作系统
    device_ip VARCHAR(45),                                -- 设备IP
    
    -- 告警分类
    severity INTEGER CHECK (severity BETWEEN 1 AND 5),    -- 严重程度 
    category VARCHAR(100),                                 -- 告警类别
    
    -- 组织信息
    org_key VARCHAR(50),                                  -- 组织标识
    
    -- 处理状态
    processing_status VARCHAR(50) DEFAULT 'unprocessed',  -- 处理状态 (unprocessed, processed, ignored)
    
    -- 唯一约束
    UNIQUE(alert_id, alert_type, source_system)
);

-- 创建告警数据索引
CREATE INDEX IF NOT EXISTS idx_alert_data_alert_type ON alert_data (alert_type);
CREATE INDEX IF NOT EXISTS idx_alert_data_alert_timestamp ON alert_data (alert_timestamp);
CREATE INDEX IF NOT EXISTS idx_alert_data_device_id ON alert_data (device_id);
CREATE INDEX IF NOT EXISTS idx_alert_data_device_name ON alert_data (device_name);
CREATE INDEX IF NOT EXISTS idx_alert_data_severity ON alert_data (severity);
CREATE INDEX IF NOT EXISTS idx_alert_data_org_key ON alert_data (org_key);
CREATE INDEX IF NOT EXISTS idx_alert_data_processing_status ON alert_data (processing_status);
CREATE INDEX IF NOT EXISTS idx_alert_data_ingestion_time ON alert_data (ingestion_time);

-- GIN索引用于JSONB查询
CREATE INDEX IF NOT EXISTS idx_alert_data_raw_data ON alert_data USING GIN (raw_data);
CREATE INDEX IF NOT EXISTS idx_alert_data_parsed_data ON alert_data USING GIN (parsed_data);

-- ================================
-- 3. 告警标注表 (alert_annotations)
-- ================================
CREATE TABLE IF NOT EXISTS alert_annotations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- 关联告警 (使用字符串ID，不使用外键约束因为告警数据存储在ClickHouse)
    alert_data_id VARCHAR(255) NOT NULL,
    
    -- 标注信息
    annotation_type VARCHAR(50) NOT NULL,                 -- 标注类型 (threat_indicator, false_positive, benign, malicious, etc.)
    labels JSONB,                                         -- 标签列表 ['malware', 'persistence', 'c2_communication']
    confidence DECIMAL(3,2) CHECK (confidence BETWEEN 0 AND 1.0), -- 标注置信度
    
    -- 威胁分析
    is_malicious BOOLEAN,                                 -- 是否恶意
    threat_level VARCHAR(20),                             -- 威胁等级 (critical, high, medium, low, info)
    mitre_techniques JSONB,                               -- 相关MITRE技术
    attack_stage VARCHAR(100),                            -- 攻击阶段
    
    -- 标注内容
    title VARCHAR(500),                                   -- 标注标题
    description TEXT,                                     -- 标注描述
    notes TEXT,                                          -- 标注备注
    
    -- 元数据
    annotation_method VARCHAR(20) DEFAULT 'manual',       -- 标注方式 (manual, auto, ml_assisted)
    data_source VARCHAR(100),                             -- 标注数据来源
    
    -- 审计信息
    annotated_by UUID NOT NULL REFERENCES users(id),     -- 标注人
    annotated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),  -- 标注时间
    reviewed_by UUID REFERENCES users(id),               -- 审核人
    reviewed_at TIMESTAMP WITH TIME ZONE,                -- 审核时间
    review_status VARCHAR(20) DEFAULT 'pending',         -- 审核状态 (pending, approved, rejected)
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 创建告警标注索引
CREATE INDEX IF NOT EXISTS idx_alert_annotations_alert_data_id ON alert_annotations (alert_data_id);
CREATE INDEX IF NOT EXISTS idx_alert_annotations_annotation_type ON alert_annotations (annotation_type);
CREATE INDEX IF NOT EXISTS idx_alert_annotations_is_malicious ON alert_annotations (is_malicious);
CREATE INDEX IF NOT EXISTS idx_alert_annotations_threat_level ON alert_annotations (threat_level);
CREATE INDEX IF NOT EXISTS idx_alert_annotations_annotated_by ON alert_annotations (annotated_by);
CREATE INDEX IF NOT EXISTS idx_alert_annotations_annotated_at ON alert_annotations (annotated_at);
CREATE INDEX IF NOT EXISTS idx_alert_annotations_review_status ON alert_annotations (review_status);

-- GIN索引
CREATE INDEX IF NOT EXISTS idx_alert_annotations_labels ON alert_annotations USING GIN (labels);
CREATE INDEX IF NOT EXISTS idx_alert_annotations_mitre_techniques ON alert_annotations USING GIN (mitre_techniques);

-- ================================
-- 4. 威胁事件关联表 (threat_event_alerts)
-- ================================
CREATE TABLE IF NOT EXISTS threat_event_alerts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- 关联关系
    threat_event_id UUID NOT NULL REFERENCES threat_events(id) ON DELETE CASCADE,
    alert_data_id UUID NOT NULL REFERENCES alert_data(id) ON DELETE CASCADE,
    
    -- 关联信息
    correlation_type VARCHAR(50) NOT NULL,                -- 关联类型 (temporal, spatial, indicator, behavior, etc.)
    correlation_score DECIMAL(3,2) CHECK (correlation_score BETWEEN 0 AND 1.0), -- 关联度分数
    
    -- 在威胁事件中的角色
    role_in_event VARCHAR(100),                           -- 在事件中的角色 (initial_access, persistence, lateral_movement, etc.)
    sequence_order INTEGER,                               -- 时间序列顺序
    
    -- 关联依据
    correlation_reason TEXT,                              -- 关联原因描述
    correlation_evidence JSONB,                           -- 关联证据 (共同IOCs, 相同设备, 时间邻近等)
    
    -- 元数据
    correlation_method VARCHAR(20) DEFAULT 'manual',      -- 关联方式 (manual, auto, ml_based)
    confidence DECIMAL(3,2) CHECK (confidence BETWEEN 0 AND 1.0), -- 关联置信度
    
    -- 审计信息
    created_by UUID NOT NULL REFERENCES users(id),       -- 创建关联的用户
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- 唯一约束
    UNIQUE(threat_event_id, alert_data_id)
);

-- 创建威胁事件关联索引
CREATE INDEX IF NOT EXISTS idx_threat_event_alerts_threat_event_id ON threat_event_alerts (threat_event_id);
CREATE INDEX IF NOT EXISTS idx_threat_event_alerts_alert_data_id ON threat_event_alerts (alert_data_id);
CREATE INDEX IF NOT EXISTS idx_threat_event_alerts_correlation_type ON threat_event_alerts (correlation_type);
CREATE INDEX IF NOT EXISTS idx_threat_event_alerts_correlation_score ON threat_event_alerts (correlation_score);
CREATE INDEX IF NOT EXISTS idx_threat_event_alerts_sequence_order ON threat_event_alerts (sequence_order);
CREATE INDEX IF NOT EXISTS idx_threat_event_alerts_created_by ON threat_event_alerts (created_by);
CREATE INDEX IF NOT EXISTS idx_threat_event_alerts_created_at ON threat_event_alerts (created_at);

-- GIN索引
CREATE INDEX IF NOT EXISTS idx_threat_event_alerts_correlation_evidence ON threat_event_alerts USING GIN (correlation_evidence);

-- ================================
-- 5. 威胁事件时间线表 (threat_event_timeline)
-- ================================
CREATE TABLE IF NOT EXISTS threat_event_timeline (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- 关联威胁事件
    threat_event_id UUID NOT NULL REFERENCES threat_events(id) ON DELETE CASCADE,
    
    -- 时间线条目
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,          -- 事件时间点
    event_type VARCHAR(100) NOT NULL,                     -- 事件类型
    title VARCHAR(500) NOT NULL,                          -- 事件标题
    description TEXT,                                     -- 事件描述
    
    -- 关联数据
    alert_data_id UUID REFERENCES alert_data(id),        -- 关联的告警(可选)
    related_entities JSONB,                               -- 相关实体 (IPs, domains, files, etc.)
    
    -- 元数据
    severity VARCHAR(20),                                 -- 该时间点事件的严重程度
    phase VARCHAR(100),                                   -- 攻击阶段
    automated BOOLEAN DEFAULT false,                      -- 是否自动生成
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_by UUID REFERENCES users(id)
);

-- 创建时间线索引
CREATE INDEX IF NOT EXISTS idx_threat_event_timeline_threat_event_id ON threat_event_timeline (threat_event_id);
CREATE INDEX IF NOT EXISTS idx_threat_event_timeline_timestamp ON threat_event_timeline (timestamp);
CREATE INDEX IF NOT EXISTS idx_threat_event_timeline_event_type ON threat_event_timeline (event_type);
CREATE INDEX IF NOT EXISTS idx_threat_event_timeline_alert_data_id ON threat_event_timeline (alert_data_id);

-- ================================
-- 6. 威胁事件相关触发器
-- ================================

-- 创建更新时间戳的触发器
CREATE TRIGGER update_threat_events_updated_at
    BEFORE UPDATE ON threat_events
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_alert_data_updated_at
    BEFORE UPDATE ON alert_data
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_alert_annotations_updated_at
    BEFORE UPDATE ON alert_annotations
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_threat_event_alerts_updated_at
    BEFORE UPDATE ON threat_event_alerts
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();