use anyhow::Result;
use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use std::fs;
use chrono::{DateTime, Utc};

use crate::{edr_alert::EdrAlert, ngav_alert::NgavAlert, database::ClickHouseConfigRow};

#[derive(Clone)]
pub struct ClickHouseConnection {
    client: Client,
    config: ClickHouseConfigRow,
}

#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct CommonAlert {
    pub id: String,
    pub data_type: String,
    pub create_time: DateTime<Utc>,
    pub device_id: u64,
    pub device_name: String,
    pub device_os: String,
    pub device_internal_ip: String,
    pub device_external_ip: String,
    pub org_key: String,
    pub severity: u8,
    pub alert_type: String,
    pub threat_category: String,
    pub device_username: String,
    pub raw_data: String,
    pub processed_time: DateTime<Utc>,
    pub kafka_topic: String,
    pub kafka_partition: u32,
    pub kafka_offset: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct EdrAlertRow {
    pub id: String,
    pub schema: u32,
    pub create_time: DateTime<Utc>,
    pub device_external_ip: String,
    pub device_id: u64,
    pub device_internal_ip: String,
    pub device_name: String,
    pub device_os: String,
    pub ioc_hit: String,
    pub ioc_id: String,
    pub org_key: String,
    pub parent_cmdline: String,
    pub parent_guid: String,
    pub parent_hash: Vec<String>,
    pub parent_path: String,
    pub parent_pid: u32,
    pub parent_publisher: Vec<String>,
    pub parent_reputation: String,
    pub parent_username: String,
    pub process_cmdline: String,
    pub process_guid: String,
    pub process_hash: Vec<String>,
    pub process_path: String,
    pub process_pid: u32,
    pub process_publisher: Vec<String>,
    pub process_reputation: String,
    pub process_username: String,
    pub report_id: String,
    pub report_name: String,
    pub report_tags: Vec<String>,
    pub severity: u8,
    pub alert_type: String,
    pub watchlists: Vec<String>,
    pub processed_time: DateTime<Utc>,
    pub kafka_topic: String,
    pub kafka_partition: u32,
    pub kafka_offset: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct NgavAlertRow {
    pub id: String,
    pub alert_type: String,
    pub legacy_alert_id: String,
    pub org_key: String,
    pub create_time: DateTime<Utc>,
    pub last_update_time: DateTime<Utc>,
    pub first_event_time: DateTime<Utc>,
    pub last_event_time: DateTime<Utc>,
    pub threat_id: String,
    pub severity: u8,
    pub category: String,
    pub device_id: u64,
    pub device_os: String,
    pub device_os_version: String,
    pub device_name: String,
    pub device_username: String,
    pub policy_id: u64,
    pub policy_name: String,
    pub target_value: String,
    pub workflow_state: String,
    pub workflow_remediation: String,
    pub workflow_last_update_time: String,
    pub workflow_comment: String,
    pub workflow_changed_by: String,
    pub device_internal_ip: String,
    pub device_external_ip: String,
    pub alert_url: String,
    pub reason: String,
    pub reason_code: String,
    pub process_name: String,
    pub device_location: String,
    pub created_by_event_id: String,
    pub threat_indicators: Vec<String>,
    pub threat_cause_actor_sha256: String,
    pub threat_cause_actor_name: String,
    pub threat_cause_actor_process_pid: String,
    pub threat_cause_reputation: String,
    pub threat_cause_threat_category: String,
    pub threat_cause_vector: String,
    pub threat_cause_cause_event_id: String,
    pub blocked_threat_category: String,
    pub not_blocked_threat_category: String,
    pub kill_chain_status: Vec<String>,
    pub run_state: String,
    pub policy_applied: String,
    pub processed_time: DateTime<Utc>,
    pub kafka_topic: String,
    pub kafka_partition: u32,
    pub kafka_offset: u64,
}

impl ClickHouseConnection {
    pub async fn new(config: ClickHouseConfigRow) -> Result<Self> {
        let url = if config.use_tls {
            format!("https://{}:{}", config.host, config.port)
        } else {
            format!("http://{}:{}", config.host, config.port)
        };

        let client = Client::default()
            .with_url(url)
            .with_user(config.username.clone())
            .with_password(config.password.clone().unwrap_or_default())
            .with_database(config.database_name.clone());

        Ok(Self { client, config })
    }

    pub async fn test_connection(&self) -> Result<bool> {
        let query = "SELECT 1";
        let result: u8 = self.client.query(query).fetch_one().await?;
        Ok(result == 1)
    }

    pub async fn initialize_tables(&self) -> Result<()> {
        log::info!("🔧 Initializing ClickHouse tables...");

        // First, create the database if it doesn't exist
        log::info!("📊 Creating ClickHouse database 'alerts' if not exists...");
        self.client.query("CREATE DATABASE IF NOT EXISTS alerts").execute().await
            .map_err(|e| anyhow::anyhow!("Failed to create ClickHouse database: {}", e))?;

        // Check if tables exist
        let tables_exist = self.check_tables_exist().await?;
        
        if tables_exist {
            log::info!("✅ ClickHouse tables already exist, skipping initialization");
            return Ok(());
        }

        log::info!("📄 Creating ClickHouse tables...");
        
        // Create tables directly with SQL
        self.create_common_alerts_table().await?;
        self.create_edr_alerts_table().await?;
        self.create_ngav_alerts_table().await?;

        log::info!("✅ ClickHouse tables initialized successfully");
        Ok(())
    }

    async fn create_common_alerts_table(&self) -> Result<()> {
        let query = r#"
            CREATE TABLE IF NOT EXISTS alerts.common_alerts (
                id String,
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
                kafka_offset UInt64
            ) ENGINE = MergeTree()
            PARTITION BY toYYYYMM(create_time)
            ORDER BY (data_type, create_time, device_id)
        "#;
        
        self.client.query(query).execute().await?;
        Ok(())
    }

    async fn create_edr_alerts_table(&self) -> Result<()> {
        let query = r#"
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
        "#;
        
        self.client.query(query).execute().await?;
        Ok(())
    }

    async fn create_ngav_alerts_table(&self) -> Result<()> {
        let query = r#"
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
                workflow_last_update_time String,
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
        "#;
        
        self.client.query(query).execute().await?;
        Ok(())
    }

    async fn check_tables_exist(&self) -> Result<bool> {
        let query = r#"
            SELECT count(*) as table_count 
            FROM system.tables 
            WHERE database = 'alerts' AND name IN ('common_alerts', 'edr_alerts', 'ngav_alerts')
        "#;
        
        let count: u64 = self.client.query(query).fetch_one().await?;
        
        Ok(count == 3)
    }

    pub async fn insert_common_alert(&self, alert: &CommonAlert) -> Result<()> {
        let query = r#"
            INSERT INTO alerts.common_alerts (
                id, data_type, create_time, device_id, device_name, device_os,
                device_internal_ip, device_external_ip, org_key, severity, alert_type,
                threat_category, device_username, raw_data, processed_time,
                kafka_topic, kafka_partition, kafka_offset
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        self.client.query(query)
            .bind(&alert.id)
            .bind(&alert.data_type)
            .bind(&alert.create_time)
            .bind(&alert.device_id)
            .bind(&alert.device_name)
            .bind(&alert.device_os)
            .bind(&alert.device_internal_ip)
            .bind(&alert.device_external_ip)
            .bind(&alert.org_key)
            .bind(&alert.severity)
            .bind(&alert.alert_type)
            .bind(&alert.threat_category)
            .bind(&alert.device_username)
            .bind(&alert.raw_data)
            .bind(&alert.processed_time)
            .bind(&alert.kafka_topic)
            .bind(&alert.kafka_partition)
            .bind(&alert.kafka_offset)
            .execute()
            .await?;

        Ok(())
    }

    pub async fn insert_edr_alert(&self, alert: &EdrAlertRow) -> Result<()> {
        let query = r#"
            INSERT INTO alerts.edr_alerts (
                id, schema, create_time, device_external_ip, device_id, device_internal_ip,
                device_name, device_os, ioc_hit, ioc_id, org_key, parent_cmdline,
                parent_guid, parent_hash, parent_path, parent_pid, parent_publisher,
                parent_reputation, parent_username, process_cmdline, process_guid,
                process_hash, process_path, process_pid, process_publisher,
                process_reputation, process_username, report_id, report_name,
                report_tags, severity, alert_type, watchlists, processed_time,
                kafka_topic, kafka_partition, kafka_offset
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        self.client.query(query)
            .bind(&alert.id)
            .bind(&alert.schema)
            .bind(&alert.create_time)
            .bind(&alert.device_external_ip)
            .bind(&alert.device_id)
            .bind(&alert.device_internal_ip)
            .bind(&alert.device_name)
            .bind(&alert.device_os)
            .bind(&alert.ioc_hit)
            .bind(&alert.ioc_id)
            .bind(&alert.org_key)
            .bind(&alert.parent_cmdline)
            .bind(&alert.parent_guid)
            .bind(&alert.parent_hash)
            .bind(&alert.parent_path)
            .bind(&alert.parent_pid)
            .bind(&alert.parent_publisher)
            .bind(&alert.parent_reputation)
            .bind(&alert.parent_username)
            .bind(&alert.process_cmdline)
            .bind(&alert.process_guid)
            .bind(&alert.process_hash)
            .bind(&alert.process_path)
            .bind(&alert.process_pid)
            .bind(&alert.process_publisher)
            .bind(&alert.process_reputation)
            .bind(&alert.process_username)
            .bind(&alert.report_id)
            .bind(&alert.report_name)
            .bind(&alert.report_tags)
            .bind(&alert.severity)
            .bind(&alert.alert_type)
            .bind(&alert.watchlists)
            .bind(&alert.processed_time)
            .bind(&alert.kafka_topic)
            .bind(&alert.kafka_partition)
            .bind(&alert.kafka_offset)
            .execute()
            .await?;

        Ok(())
    }

    pub async fn insert_ngav_alert(&self, alert: &NgavAlertRow) -> Result<()> {
        let query = r#"
            INSERT INTO alerts.ngav_alerts (
                id, alert_type, legacy_alert_id, org_key, create_time, last_update_time,
                first_event_time, last_event_time, threat_id, severity, category,
                device_id, device_os, device_os_version, device_name, device_username,
                policy_id, policy_name, target_value, workflow_state, workflow_remediation,
                workflow_last_update_time, workflow_comment, workflow_changed_by,
                device_internal_ip, device_external_ip, alert_url, reason, reason_code,
                process_name, device_location, created_by_event_id, threat_indicators,
                threat_cause_actor_sha256, threat_cause_actor_name, threat_cause_actor_process_pid,
                threat_cause_reputation, threat_cause_threat_category, threat_cause_vector,
                threat_cause_cause_event_id, blocked_threat_category, not_blocked_threat_category,
                kill_chain_status, run_state, policy_applied, processed_time,
                kafka_topic, kafka_partition, kafka_offset
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        self.client.query(query)
            .bind(&alert.id)
            .bind(&alert.alert_type)
            .bind(&alert.legacy_alert_id)
            .bind(&alert.org_key)
            .bind(&alert.create_time)
            .bind(&alert.last_update_time)
            .bind(&alert.first_event_time)
            .bind(&alert.last_event_time)
            .bind(&alert.threat_id)
            .bind(&alert.severity)
            .bind(&alert.category)
            .bind(&alert.device_id)
            .bind(&alert.device_os)
            .bind(&alert.device_os_version)
            .bind(&alert.device_name)
            .bind(&alert.device_username)
            .bind(&alert.policy_id)
            .bind(&alert.policy_name)
            .bind(&alert.target_value)
            .bind(&alert.workflow_state)
            .bind(&alert.workflow_remediation)
            .bind(&alert.workflow_last_update_time)
            .bind(&alert.workflow_comment)
            .bind(&alert.workflow_changed_by)
            .bind(&alert.device_internal_ip)
            .bind(&alert.device_external_ip)
            .bind(&alert.alert_url)
            .bind(&alert.reason)
            .bind(&alert.reason_code)
            .bind(&alert.process_name)
            .bind(&alert.device_location)
            .bind(&alert.created_by_event_id)
            .bind(&alert.threat_indicators)
            .bind(&alert.threat_cause_actor_sha256)
            .bind(&alert.threat_cause_actor_name)
            .bind(&alert.threat_cause_actor_process_pid)
            .bind(&alert.threat_cause_reputation)
            .bind(&alert.threat_cause_threat_category)
            .bind(&alert.threat_cause_vector)
            .bind(&alert.threat_cause_cause_event_id)
            .bind(&alert.blocked_threat_category)
            .bind(&alert.not_blocked_threat_category)
            .bind(&alert.kill_chain_status)
            .bind(&alert.run_state)
            .bind(&alert.policy_applied)
            .bind(&alert.processed_time)
            .bind(&alert.kafka_topic)
            .bind(&alert.kafka_partition)
            .bind(&alert.kafka_offset)
            .execute()
            .await?;

        Ok(())
    }

    pub async fn get_common_alerts(&self, limit: u32, offset: u32) -> Result<Vec<CommonAlert>> {
        let query = format!(
            "SELECT * FROM alerts.common_alerts ORDER BY create_time DESC LIMIT {} OFFSET {}",
            limit, offset
        );

        let alerts = self.client.query(&query).fetch_all().await?;
        Ok(alerts)
    }

    pub async fn get_common_alert_by_id(&self, id: &str) -> Result<Option<CommonAlert>> {
        let query = "SELECT * FROM alerts.common_alerts WHERE id = ?";
        
        let mut cursor = self.client.query(query).bind(id).fetch()?;
        
        if let Some(row) = cursor.next().await? {
            Ok(Some(row))
        } else {
            Ok(None)
        }
    }

    pub async fn get_edr_alert_by_id(&self, id: &str) -> Result<Option<EdrAlertRow>> {
        let query = "SELECT * FROM alerts.edr_alerts WHERE id = ?";
        
        let mut cursor = self.client.query(query).bind(id).fetch()?;
        
        if let Some(row) = cursor.next().await? {
            Ok(Some(row))
        } else {
            Ok(None)
        }
    }

    pub async fn get_ngav_alert_by_id(&self, id: &str) -> Result<Option<NgavAlertRow>> {
        let query = "SELECT * FROM alerts.ngav_alerts WHERE id = ?";
        
        let mut cursor = self.client.query(query).bind(id).fetch()?;
        
        if let Some(row) = cursor.next().await? {
            Ok(Some(row))
        } else {
            Ok(None)
        }
    }

    pub async fn get_alert_count_by_type(&self) -> Result<serde_json::Value> {
        let query = r#"
            SELECT 
                data_type,
                count(*) as count,
                max(create_time) as latest_time
            FROM alerts.common_alerts 
            GROUP BY data_type
        "#;

        #[derive(Debug, Row, Deserialize)]
        struct AlertCountRow {
            data_type: String,
            count: u64,
            latest_time: DateTime<Utc>,
        }

        let rows: Vec<AlertCountRow> = self.client.query(query).fetch_all().await?;
        
        let mut result = serde_json::Map::new();
        for row in rows {
            result.insert(row.data_type, serde_json::json!({
                "count": row.count,
                "latest_time": row.latest_time
            }));
        }

        Ok(serde_json::Value::Object(result))
    }
}

// Conversion functions from original alert types to ClickHouse row types
impl From<&EdrAlert> for CommonAlert {
    fn from(edr: &EdrAlert) -> Self {
        let create_time = chrono::DateTime::parse_from_rfc3339(&edr.create_time)
            .unwrap_or_else(|_| chrono::Utc::now().into())
            .with_timezone(&chrono::Utc);

        Self {
            id: edr.get_alert_key(),
            data_type: "edr".to_string(),
            create_time,
            device_id: edr.device_id,
            device_name: edr.device_name.clone(),
            device_os: edr.device_os.clone(),
            device_internal_ip: edr.device_internal_ip.clone(),
            device_external_ip: edr.device_external_ip.clone(),
            org_key: edr.org_key.clone(),
            severity: edr.severity,
            alert_type: edr.alert_type.clone(),
            threat_category: "".to_string(), // EDR doesn't have threat_category
            device_username: edr.process_username.clone(),
            raw_data: serde_json::to_string(edr).unwrap_or_default(),
            processed_time: chrono::Utc::now(),
            kafka_topic: "".to_string(),
            kafka_partition: 0,
            kafka_offset: 0,
        }
    }
}

impl From<&NgavAlert> for CommonAlert {
    fn from(ngav: &NgavAlert) -> Self {
        let create_time = chrono::DateTime::parse_from_rfc3339(&ngav.create_time)
            .unwrap_or_else(|_| chrono::Utc::now().into())
            .with_timezone(&chrono::Utc);

        Self {
            id: ngav.get_alert_key(),
            data_type: "ngav".to_string(),
            create_time,
            device_id: ngav.device_id,
            device_name: ngav.device_name.clone(),
            device_os: ngav.device_os.clone(),
            device_internal_ip: ngav.device_internal_ip.clone(),
            device_external_ip: ngav.device_external_ip.clone(),
            org_key: ngav.org_key.clone(),
            severity: ngav.severity,
            alert_type: ngav.alert_type.clone(),
            threat_category: ngav.threat_cause_threat_category.clone(),
            device_username: ngav.device_username.clone(),
            raw_data: serde_json::to_string(ngav).unwrap_or_default(),
            processed_time: chrono::Utc::now(),
            kafka_topic: "".to_string(),
            kafka_partition: 0,
            kafka_offset: 0,
        }
    }
}

impl From<&EdrAlert> for EdrAlertRow {
    fn from(edr: &EdrAlert) -> Self {
        let create_time = chrono::DateTime::parse_from_rfc3339(&edr.create_time)
            .unwrap_or_else(|_| chrono::Utc::now().into())
            .with_timezone(&chrono::Utc);

        Self {
            id: edr.get_alert_key(),
            schema: edr.schema as u32,
            create_time,
            device_external_ip: edr.device_external_ip.clone(),
            device_id: edr.device_id,
            device_internal_ip: edr.device_internal_ip.clone(),
            device_name: edr.device_name.clone(),
            device_os: edr.device_os.clone(),
            ioc_hit: edr.ioc_hit.clone(),
            ioc_id: edr.ioc_id.clone(),
            org_key: edr.org_key.clone(),
            parent_cmdline: edr.parent_cmdline.clone(),
            parent_guid: edr.parent_guid.clone(),
            parent_hash: edr.parent_hash.clone(),
            parent_path: edr.parent_path.clone(),
            parent_pid: edr.parent_pid,
            parent_publisher: edr.parent_publisher.iter().map(|p| p.name.clone()).collect(),
            parent_reputation: edr.parent_reputation.clone(),
            parent_username: edr.parent_username.clone(),
            process_cmdline: edr.process_cmdline.clone(),
            process_guid: edr.process_guid.clone(),
            process_hash: edr.process_hash.clone(),
            process_path: edr.process_path.clone(),
            process_pid: edr.process_pid,
            process_publisher: edr.process_publisher.iter().map(|p| p.name.clone()).collect(),
            process_reputation: edr.process_reputation.clone(),
            process_username: edr.process_username.clone(),
            report_id: edr.report_id.clone(),
            report_name: edr.report_name.clone(),
            report_tags: edr.report_tags.clone(),
            severity: edr.severity,
            alert_type: edr.alert_type.clone(),
            watchlists: edr.watchlists.iter().map(|w| w.name.clone()).collect(),
            processed_time: chrono::Utc::now(),
            kafka_topic: "".to_string(),
            kafka_partition: 0,
            kafka_offset: 0,
        }
    }
}

impl From<&NgavAlert> for NgavAlertRow {
    fn from(ngav: &NgavAlert) -> Self {
        let create_time = chrono::DateTime::parse_from_rfc3339(&ngav.create_time)
            .unwrap_or_else(|_| chrono::Utc::now().into())
            .with_timezone(&chrono::Utc);
        
        let last_update_time = chrono::DateTime::parse_from_rfc3339(&ngav.last_update_time)
            .unwrap_or_else(|_| chrono::Utc::now().into())
            .with_timezone(&chrono::Utc);
        
        let first_event_time = chrono::DateTime::parse_from_rfc3339(&ngav.first_event_time)
            .unwrap_or_else(|_| chrono::Utc::now().into())
            .with_timezone(&chrono::Utc);
        
        let last_event_time = chrono::DateTime::parse_from_rfc3339(&ngav.last_event_time)
            .unwrap_or_else(|_| chrono::Utc::now().into())
            .with_timezone(&chrono::Utc);

        Self {
            id: ngav.get_alert_key(),
            alert_type: ngav.alert_type.clone(),
            legacy_alert_id: ngav.legacy_alert_id.clone(),
            org_key: ngav.org_key.clone(),
            create_time,
            last_update_time,
            first_event_time,
            last_event_time,
            threat_id: ngav.threat_id.clone(),
            severity: ngav.severity,
            category: ngav.category.clone(),
            device_id: ngav.device_id,
            device_os: ngav.device_os.clone(),
            device_os_version: ngav.device_os_version.clone(),
            device_name: ngav.device_name.clone(),
            device_username: ngav.device_username.clone(),
            policy_id: ngav.policy_id,
            policy_name: ngav.policy_name.clone(),
            target_value: ngav.target_value.clone(),
            workflow_state: ngav.workflow.state.clone(),
            workflow_remediation: ngav.workflow.remediation.clone(),
            workflow_last_update_time: ngav.workflow.last_update_time.clone(),
            workflow_comment: ngav.workflow.comment.clone(),
            workflow_changed_by: ngav.workflow.changed_by.clone(),
            device_internal_ip: ngav.device_internal_ip.clone(),
            device_external_ip: ngav.device_external_ip.clone(),
            alert_url: ngav.alert_url.clone(),
            reason: ngav.reason.clone(),
            reason_code: ngav.reason_code.clone(),
            process_name: ngav.process_name.clone(),
            device_location: ngav.device_location.clone(),
            created_by_event_id: ngav.created_by_event_id.clone(),
            threat_indicators: ngav.threat_indicators.iter()
                .map(|ti| format!("{}:{}", ti.process_name, ti.sha256))
                .collect(),
            threat_cause_actor_sha256: ngav.threat_cause_actor_sha256.clone(),
            threat_cause_actor_name: ngav.threat_cause_actor_name.clone(),
            threat_cause_actor_process_pid: ngav.threat_cause_actor_process_pid.clone(),
            threat_cause_reputation: ngav.threat_cause_reputation.clone(),
            threat_cause_threat_category: ngav.threat_cause_threat_category.clone(),
            threat_cause_vector: ngav.threat_cause_vector.clone(),
            threat_cause_cause_event_id: ngav.threat_cause_cause_event_id.clone(),
            blocked_threat_category: ngav.blocked_threat_category.clone(),
            not_blocked_threat_category: ngav.not_blocked_threat_category.clone(),
            kill_chain_status: ngav.kill_chain_status.clone(),
            run_state: ngav.run_state.clone(),
            policy_applied: ngav.policy_applied.clone(),
            processed_time: chrono::Utc::now(),
            kafka_topic: "".to_string(),
            kafka_partition: 0,
            kafka_offset: 0,
        }
    }
}