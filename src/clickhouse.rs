use anyhow::Result;
use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::fs;

use crate::{edr_alert::EdrAlert, ngav_alert::NgavAlert, database::ClickHouseConfigRow};

#[derive(Debug)]
pub struct AlertFilters {
    pub device_name: Option<String>,
    pub device_ip: Option<String>,
    pub alert_type: Option<String>,
    pub threat_category: Option<String>,
    pub severity: Option<u32>,
    pub data_type: Option<String>,
    pub kafka_source: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

// Helper function to parse ISO 8601 datetime strings and format for ClickHouse DateTime64(3)
fn parse_datetime_for_clickhouse(datetime_str: &str) -> String {
    // Try parsing with Z suffix first
    if let Ok(dt) = DateTime::parse_from_rfc3339(datetime_str) {
        // Convert to ClickHouse DateTime64(3) format: YYYY-MM-DD HH:MM:SS.sss
        return dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string();
    }
    
    // Fallback to current time if parsing fails
    log::warn!("Failed to parse datetime '{}', using current time", datetime_str);
    Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}


#[derive(Clone)]
pub struct ClickHouseConnection {
    client: Client,
    config: ClickHouseConfigRow,
}

#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct CommonAlert {
    pub id: String,
    pub original_id: String,
    pub data_type: String,
    pub create_time: String,
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
    pub processed_time: String,
    pub kafka_topic: String,
    pub kafka_partition: u32,
    pub kafka_offset: u64,
    pub kafka_config_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct EdrAlertRow {
    pub id: String,
    pub schema: u32,
    pub create_time: String,
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
    pub processed_time: String,
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
    pub create_time: String,
    pub last_update_time: String,
    pub first_event_time: String,
    pub last_event_time: String,
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
    pub processed_time: String,
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

    // Get the ClickHouse client for direct queries
    pub fn get_client(&self) -> &Client {
        &self.client
    }

    // Helper function to clean string fields and ensure valid UTF-8
    fn clean_string(s: String) -> String {
        if s.is_empty() {
            s
        } else {
            // Convert to bytes and back with lossy conversion to handle invalid UTF-8
            let bytes = s.into_bytes();
            let cleaned = String::from_utf8_lossy(&bytes).to_string();
            
            // Log if we found any issues
            if cleaned.contains('\u{FFFD}') {
                log::warn!("Found and cleaned invalid UTF-8 characters in string");
                // Remove replacement characters
                cleaned.chars().filter(|&c| c != '\u{FFFD}').collect()
            } else {
                cleaned
            }
        }
    }

    // Helper function to clean CommonAlert
    fn clean_common_alert(&self, mut alert: CommonAlert) -> CommonAlert {
        alert.id = Self::clean_string(alert.id);
        alert.original_id = Self::clean_string(alert.original_id);
        alert.data_type = Self::clean_string(alert.data_type);
        alert.create_time = Self::clean_string(alert.create_time);
        alert.device_name = Self::clean_string(alert.device_name);
        alert.device_os = Self::clean_string(alert.device_os);
        alert.device_internal_ip = Self::clean_string(alert.device_internal_ip);
        alert.device_external_ip = Self::clean_string(alert.device_external_ip);
        alert.org_key = Self::clean_string(alert.org_key);
        alert.alert_type = Self::clean_string(alert.alert_type);
        alert.threat_category = Self::clean_string(alert.threat_category);
        alert.device_username = Self::clean_string(alert.device_username);
        alert.raw_data = Self::clean_string(alert.raw_data);
        alert.processed_time = Self::clean_string(alert.processed_time);
        alert.kafka_topic = Self::clean_string(alert.kafka_topic);
        alert.kafka_config_name = Self::clean_string(alert.kafka_config_name);
        alert
    }

    pub async fn test_connection(&self) -> Result<bool> {
        let query = "SELECT 1";
        let result: u8 = self.client.query(query).fetch_one().await?;
        Ok(result == 1)
    }

    pub async fn initialize_tables(&self) -> Result<()> {
        log::info!("🔧 Initializing ClickHouse tables...");

        // Read and execute SQL from clickhouse_init.sql
        self.execute_sql_file("clickhouse_init.sql").await?;

        log::info!("✅ ClickHouse tables initialized successfully");
        Ok(())
    }

    /// Force reinitialize tables (drop and recreate)
    pub async fn reinitialize_tables(&self) -> Result<()> {
        log::info!("🔄 Force reinitializing ClickHouse tables (drop and recreate)...");

        // Always drop existing tables in reinitialize mode
        log::info!("🗑️ Dropping existing tables...");
        self.drop_existing_tables().await?;

        log::info!("📄 Creating fresh ClickHouse tables from SQL file...");
        
        // Read and execute SQL from clickhouse_init.sql
        self.execute_sql_file("clickhouse_init.sql").await?;

        log::info!("✅ ClickHouse tables reinitialized successfully");
        Ok(())
    }

    async fn execute_sql_file(&self, filename: &str) -> Result<()> {
        log::info!("📄 Reading SQL from file: {}", filename);
        
        // Read the SQL file
        let sql_content = fs::read_to_string(filename)
            .map_err(|e| anyhow::anyhow!("Failed to read SQL file '{}': {}", filename, e))?;
        
        // Remove comments and clean up the content
        let cleaned_content = sql_content
            .lines()
            .filter(|line| !line.trim().starts_with("--") && !line.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        
        // Split SQL content by semicolons and execute each statement
        let statements: Vec<&str> = cleaned_content
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        
        for (i, statement) in statements.iter().enumerate() {
            if statement.is_empty() {
                continue;
            }
            
            log::info!("Executing SQL statement {}: {}", i + 1, statement.lines().next().unwrap_or(statement));
            
            if let Err(e) = self.client.query(statement).execute().await {
                log::error!("Failed to execute SQL statement {}: {} - Error: {}", i + 1, statement.lines().next().unwrap_or(statement), e);
                // Continue with other statements instead of failing completely
            } else {
                log::info!("Successfully executed SQL statement {}", i + 1);
            }
        }
        
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

    async fn drop_existing_tables(&self) -> Result<()> {
        // Drop tables in dependency order
        let tables = ["common_alerts", "edr_alerts", "ngav_alerts"];
        
        for table in &tables {
            let query = format!("DROP TABLE IF EXISTS alerts.{}", table);
            if let Err(e) = self.client.query(&query).execute().await {
                log::warn!("Failed to drop table {}: {}", table, e);
            } else {
                log::info!("Dropped table alerts.{}", table);
            }
        }
        
        Ok(())
    }

    pub async fn insert_common_alert(&self, alert: &CommonAlert) -> Result<()> {
        let query = r#"
            INSERT INTO alerts.common_alerts (
                original_id, data_type, create_time, device_id, device_name, device_os,
                device_internal_ip, device_external_ip, org_key, severity, alert_type,
                threat_category, device_username, raw_data,
                kafka_topic, kafka_partition, kafka_offset, kafka_config_name
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        self.client.query(query)
            .bind(&alert.original_id)
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
            .bind(&alert.kafka_topic)
            .bind(&alert.kafka_partition)
            .bind(&alert.kafka_offset)
            .bind(&alert.kafka_config_name)
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
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
            r#"SELECT 
                toString(id) as id, 
                toString(original_id) as original_id, 
                toString(data_type) as data_type, 
                toString(create_time) as create_time,
                device_id, 
                toString(device_name) as device_name, 
                toString(device_os) as device_os,
                toString(device_internal_ip) as device_internal_ip, 
                toString(device_external_ip) as device_external_ip, 
                toString(org_key) as org_key,
                severity, 
                toString(alert_type) as alert_type, 
                toString(threat_category) as threat_category, 
                toString(device_username) as device_username,
                toString(raw_data) as raw_data, 
                toString(processed_time) as processed_time,
                toString(kafka_topic) as kafka_topic, 
                kafka_partition, kafka_offset, 
                toString(kafka_config_name) as kafka_config_name
            FROM alerts.common_alerts 
            ORDER BY processed_time DESC, id DESC 
            LIMIT {} OFFSET {}"#,
            limit, offset
        );

        log::debug!("Executing query: {}", query);
        
        // Try to fetch data with error handling
        match self.client.query(&query).fetch_all().await {
            Ok(alerts) => {
                log::info!("Successfully fetched {} alerts from ClickHouse", alerts.len());
                // Clean all alerts to ensure valid UTF-8
                let cleaned_alerts: Vec<CommonAlert> = alerts.into_iter()
                    .map(|alert| self.clean_common_alert(alert))
                    .collect();
                Ok(cleaned_alerts)
            },
            Err(e) => {
                log::error!("Failed to fetch alerts from ClickHouse: {}", e);
                // Try a simpler query to see if there's data corruption
                self.get_alerts_with_fallback(limit, offset).await
            }
        }
    }

    async fn get_alerts_with_fallback(&self, limit: u32, offset: u32) -> Result<Vec<CommonAlert>> {
        log::warn!("Using fallback method to fetch alerts due to UTF-8 issues");
        
        // Try a very basic query first to see if we can get any data
        let simple_query = format!(
            "SELECT id, toString(device_name) as device_name, severity FROM alerts.common_alerts LIMIT {} OFFSET {}",
            limit, offset
        );
        
        match self.client.query(&simple_query).fetch_all::<(String, String, u8)>().await {
            Ok(rows) => {
                log::info!("Fallback query returned {} rows", rows.len());
                // Create minimal alert objects
                let alerts = rows.into_iter().map(|(id, device_name, severity)| {
                    CommonAlert {
                        id: Self::clean_string(id),
                        original_id: "".to_string(),
                        data_type: "unknown".to_string(),
                        create_time: "".to_string(),
                        device_id: 0,
                        device_name: Self::clean_string(device_name),
                        device_os: "".to_string(),
                        device_internal_ip: "".to_string(),
                        device_external_ip: "".to_string(),
                        org_key: "".to_string(),
                        severity,
                        alert_type: "".to_string(),
                        threat_category: "".to_string(),
                        device_username: "".to_string(),
                        raw_data: "".to_string(),
                        processed_time: "".to_string(),
                        kafka_topic: "".to_string(),
                        kafka_partition: 0,
                        kafka_offset: 0,
                        kafka_config_name: "".to_string(),
                    }
                }).collect();
                Ok(alerts)
            },
            Err(e) => {
                log::error!("Even fallback query failed: {}", e);
                // Return empty result rather than failing completely
                Ok(vec![])
            }
        }
    }

    pub async fn get_common_alerts_with_filters(&self, limit: u32, offset: u32, filters: &AlertFilters) -> Result<Vec<CommonAlert>> {
        let mut query = String::from(
            r#"SELECT 
                toString(id) as id, 
                toString(original_id) as original_id, 
                toString(data_type) as data_type, 
                toString(create_time) as create_time,
                device_id, 
                toString(device_name) as device_name, 
                toString(device_os) as device_os,
                toString(device_internal_ip) as device_internal_ip, 
                toString(device_external_ip) as device_external_ip, 
                toString(org_key) as org_key,
                severity, 
                toString(alert_type) as alert_type, 
                toString(threat_category) as threat_category, 
                toString(device_username) as device_username,
                toString(raw_data) as raw_data, 
                toString(processed_time) as processed_time,
                toString(kafka_topic) as kafka_topic, 
                kafka_partition, kafka_offset, 
                toString(kafka_config_name) as kafka_config_name
            FROM alerts.common_alerts"#
        );

        let mut conditions = Vec::new();

        // Add filter conditions
        if let Some(ref device_name) = filters.device_name {
            if !device_name.is_empty() {
                conditions.push(format!("lower(device_name) LIKE lower('%{}%')", device_name.replace("'", "''")));
            }
        }

        if let Some(ref device_ip) = filters.device_ip {
            if !device_ip.is_empty() {
                conditions.push(format!(
                    "(lower(device_internal_ip) LIKE lower('%{}%') OR lower(device_external_ip) LIKE lower('%{}%'))", 
                    device_ip.replace("'", "''"), device_ip.replace("'", "''")
                ));
            }
        }

        if let Some(ref alert_type) = filters.alert_type {
            if !alert_type.is_empty() {
                conditions.push(format!("lower(alert_type) LIKE lower('%{}%')", alert_type.replace("'", "''")));
            }
        }

        if let Some(ref threat_category) = filters.threat_category {
            if !threat_category.is_empty() {
                conditions.push(format!("lower(threat_category) LIKE lower('%{}%')", threat_category.replace("'", "''")));
            }
        }

        if let Some(severity) = filters.severity {
            conditions.push(format!("severity = {}", severity));
        }

        if let Some(ref data_type) = filters.data_type {
            if !data_type.is_empty() {
                conditions.push(format!("lower(data_type) = lower('{}')", data_type.replace("'", "''")));
            }
        }

        if let Some(ref kafka_source) = filters.kafka_source {
            if !kafka_source.is_empty() {
                conditions.push(format!("lower(kafka_config_name) LIKE lower('%{}%')", kafka_source.replace("'", "''")));
            }
        }

        if let Some(ref date_from) = filters.date_from {
            if !date_from.is_empty() {
                conditions.push(format!("create_time >= '{}'", date_from.replace("'", "''")));
            }
        }

        if let Some(ref date_to) = filters.date_to {
            if !date_to.is_empty() {
                conditions.push(format!("create_time <= '{}'", date_to.replace("'", "''")));
            }
        }

        // Add WHERE clause if we have conditions
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        // Add ordering and pagination
        query.push_str(&format!(
            " ORDER BY processed_time DESC, id DESC LIMIT {} OFFSET {}",
            limit, offset
        ));

        log::debug!("Executing filtered query: {}", query);
        
        // Try to fetch data with error handling
        match self.client.query(&query).fetch_all().await {
            Ok(alerts) => {
                log::info!("Successfully fetched {} filtered alerts from ClickHouse", alerts.len());
                // Clean all alerts to ensure valid UTF-8
                let cleaned_alerts: Vec<CommonAlert> = alerts.into_iter()
                    .map(|alert| self.clean_common_alert(alert))
                    .collect();
                Ok(cleaned_alerts)
            },
            Err(e) => {
                log::error!("Failed to fetch filtered alerts from ClickHouse: {}", e);
                // Fall back to unfiltered query if filtering fails
                self.get_common_alerts(limit, offset).await
            }
        }
    }

    pub async fn get_alerts_count(&self) -> Result<u64> {
        let query = "SELECT count(*) as count FROM alerts.common_alerts";
        
        let count: u64 = self.client.query(query).fetch_one().await?;
        
        Ok(count)
    }

    pub async fn get_filtered_alerts_count(&self, filters: &AlertFilters) -> Result<u64> {
        let mut query = String::from("SELECT count(*) as count FROM alerts.common_alerts");
        let mut conditions = Vec::new();

        // Add the same filter conditions as in get_common_alerts_with_filters
        if let Some(ref device_name) = filters.device_name {
            if !device_name.is_empty() {
                conditions.push(format!("lower(device_name) LIKE lower('%{}%')", device_name.replace("'", "''")));
            }
        }

        if let Some(ref device_ip) = filters.device_ip {
            if !device_ip.is_empty() {
                conditions.push(format!(
                    "(lower(device_internal_ip) LIKE lower('%{}%') OR lower(device_external_ip) LIKE lower('%{}%'))", 
                    device_ip.replace("'", "''"), device_ip.replace("'", "''")
                ));
            }
        }

        if let Some(ref alert_type) = filters.alert_type {
            if !alert_type.is_empty() {
                conditions.push(format!("lower(alert_type) LIKE lower('%{}%')", alert_type.replace("'", "''")));
            }
        }

        if let Some(ref threat_category) = filters.threat_category {
            if !threat_category.is_empty() {
                conditions.push(format!("lower(threat_category) LIKE lower('%{}%')", threat_category.replace("'", "''")));
            }
        }

        if let Some(severity) = filters.severity {
            conditions.push(format!("severity = {}", severity));
        }

        if let Some(ref data_type) = filters.data_type {
            if !data_type.is_empty() {
                conditions.push(format!("lower(data_type) = lower('{}')", data_type.replace("'", "''")));
            }
        }

        if let Some(ref kafka_source) = filters.kafka_source {
            if !kafka_source.is_empty() {
                conditions.push(format!("lower(kafka_config_name) LIKE lower('%{}%')", kafka_source.replace("'", "''")));
            }
        }

        if let Some(ref date_from) = filters.date_from {
            if !date_from.is_empty() {
                conditions.push(format!("create_time >= '{}'", date_from.replace("'", "''")));
            }
        }

        if let Some(ref date_to) = filters.date_to {
            if !date_to.is_empty() {
                conditions.push(format!("create_time <= '{}'", date_to.replace("'", "''")));
            }
        }

        // Add WHERE clause if we have conditions
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        log::debug!("Executing filtered count query: {}", query);
        
        match self.client.query(&query).fetch_one().await {
            Ok(count) => Ok(count),
            Err(e) => {
                log::error!("Failed to get filtered count from ClickHouse: {}", e);
                // Fall back to total count if filtering fails
                self.get_alerts_count().await
            }
        }
    }

    // Debug function to check what's in the database
    pub async fn debug_table_contents(&self) -> Result<()> {
        log::info!("🔍 Debugging ClickHouse table contents...");
        
        // Check if table exists
        let table_check = "SELECT count(*) FROM system.tables WHERE database = 'alerts' AND name = 'common_alerts'";
        let table_exists: u64 = self.client.query(table_check).fetch_one().await?;
        log::info!("Table exists: {}", table_exists > 0);
        
        if table_exists > 0 {
            // Check row count
            let count: u64 = self.get_alerts_count().await?;
            log::info!("Total rows: {}", count);
            
            if count > 0 {
                // Try to get just the first row with minimal fields
                let simple_query = "SELECT toString(id) as id, toString(device_name) as device_name FROM alerts.common_alerts LIMIT 1";
                match self.client.query(simple_query).fetch_one::<(String, String)>().await {
                    Ok((id, device_name)) => {
                        log::info!("Sample row - ID: '{}', Device: '{}'", 
                            id.chars().take(50).collect::<String>(), 
                            device_name.chars().take(50).collect::<String>());
                    },
                    Err(e) => {
                        log::error!("Failed to fetch sample row: {}", e);
                    }
                }
            }
        }
        
        Ok(())
    }

    pub async fn get_common_alert_by_id(&self, id: &str) -> Result<Option<CommonAlert>> {
        let query = r#"SELECT 
            toString(id) as id, 
            toString(original_id) as original_id, 
            toString(data_type) as data_type, 
            toString(create_time) as create_time,
            device_id, 
            toString(device_name) as device_name, 
            toString(device_os) as device_os,
            toString(device_internal_ip) as device_internal_ip, 
            toString(device_external_ip) as device_external_ip, 
            toString(org_key) as org_key,
            severity, 
            toString(alert_type) as alert_type, 
            toString(threat_category) as threat_category, 
            toString(device_username) as device_username,
            toString(raw_data) as raw_data, 
            toString(processed_time) as processed_time,
            toString(kafka_topic) as kafka_topic, 
            kafka_partition, kafka_offset, 
            toString(kafka_config_name) as kafka_config_name
        FROM alerts.common_alerts WHERE id = ?"#;
        
        log::debug!("Getting alert detail for ID: {}", id);
        
        match self.client.query(query).bind(id).fetch() {
            Ok(mut cursor) => {
                match cursor.next().await {
                    Ok(Some(row)) => {
                        log::debug!("Found alert with ID: {}", id);
                        Ok(Some(self.clean_common_alert(row)))
                    },
                    Ok(None) => {
                        log::debug!("No alert found with ID: {}", id);
                        Ok(None)
                    },
                    Err(e) => {
                        log::error!("Failed to fetch alert row for ID {}: {}", id, e);
                        // Try a fallback query with minimal fields
                        self.get_alert_by_id_fallback(id).await
                    }
                }
            },
            Err(e) => {
                log::error!("Failed to prepare query for alert ID {}: {}", id, e);
                Err(e.into())
            }
        }
    }

    async fn get_alert_by_id_fallback(&self, id: &str) -> Result<Option<CommonAlert>> {
        log::warn!("Using fallback method to get alert detail for ID: {}", id);
        
        // Try a very basic query with only essential fields
        let simple_query = "SELECT toString(id) as id, toString(device_name) as device_name, severity FROM alerts.common_alerts WHERE id = ?";
        
        match self.client.query(simple_query).bind(id).fetch_one::<(String, String, u8)>().await {
            Ok((alert_id, device_name, severity)) => {
                log::info!("Fallback query found alert: {}", alert_id);
                Ok(Some(CommonAlert {
                    id: Self::clean_string(alert_id),
                    original_id: "".to_string(),
                    data_type: "unknown".to_string(),
                    create_time: "".to_string(),
                    device_id: 0,
                    device_name: Self::clean_string(device_name),
                    device_os: "".to_string(),
                    device_internal_ip: "".to_string(),
                    device_external_ip: "".to_string(),
                    org_key: "".to_string(),
                    severity,
                    alert_type: "".to_string(),
                    threat_category: "".to_string(),
                    device_username: "".to_string(),
                    raw_data: "{\"error\": \"Data retrieval failed, showing minimal info\"}".to_string(),
                    processed_time: "".to_string(),
                    kafka_topic: "".to_string(),
                    kafka_partition: 0,
                    kafka_offset: 0,
                    kafka_config_name: "".to_string(),
                }))
            },
            Err(e) => {
                log::error!("Even fallback query failed for ID {}: {}", id, e);
                Ok(None)
            }
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
                toString(data_type) as data_type,
                count(*) as count,
                toString(max(processed_time)) as latest_time
            FROM alerts.common_alerts 
            GROUP BY data_type
        "#;

        #[derive(Debug, Row, Deserialize)]
        struct AlertCountRow {
            data_type: String,
            count: u64,
            latest_time: String,
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

    pub async fn get_alert_stats(&self, minutes: u32) -> Result<(f64, u64, std::collections::HashMap<String, u64>, std::collections::HashMap<String, u64>)> {
        use std::collections::HashMap;
        
        log::debug!("Getting alert stats for last {} minutes", minutes);
        
        // Get total message count
        let total_query = "SELECT count(*) as total FROM alerts.common_alerts";
        let total: u64 = self.client.query(total_query).fetch_one().await
            .map_err(|e| anyhow::anyhow!("Failed to get total count: {}", e))?;
        
        log::debug!("Total alerts: {}", total);
        
        // Get message rate (messages per second in the last N minutes)
        let rate_query = format!(r#"
            SELECT count(*) as count
            FROM alerts.common_alerts
            WHERE processed_time >= now() - INTERVAL {} MINUTE
        "#, minutes);
        
        let recent_count: u64 = self.client.query(&rate_query).fetch_one().await
            .map_err(|e| anyhow::anyhow!("Failed to get recent count: {}", e))?;
        let message_rate = recent_count as f64 / (minutes as f64 * 60.0);
        
        log::debug!("Recent count: {}, message rate: {:.2}/sec", recent_count, message_rate);
        
        // Get type breakdown
        let type_query = r#"
            SELECT toString(data_type) as data_type, count(*) as count
            FROM alerts.common_alerts
            GROUP BY data_type
        "#;
        
        #[derive(Debug, Row, Deserialize)]
        struct TypeCount {
            data_type: String,
            count: u64,
        }
        
        let type_rows: Vec<TypeCount> = self.client.query(type_query).fetch_all().await
            .map_err(|e| anyhow::anyhow!("Failed to get type breakdown: {}", e))?;
        let mut type_breakdown = HashMap::new();
        for row in type_rows {
            type_breakdown.insert(row.data_type, row.count);
        }
        
        log::debug!("Type breakdown: {:?}", type_breakdown);
        
        // Get severity breakdown  
        let severity_query = r#"
            SELECT 
                CASE 
                    WHEN severity >= 8 THEN 'critical'
                    WHEN severity >= 5 THEN 'warning'
                    ELSE 'info'
                END as severity_level,
                count(*) as count
            FROM alerts.common_alerts
            GROUP BY severity_level
        "#;
        
        #[derive(Debug, Row, Deserialize)]
        struct SeverityCount {
            severity_level: String,
            count: u64,
        }
        
        let severity_rows: Vec<SeverityCount> = self.client.query(severity_query).fetch_all().await
            .map_err(|e| anyhow::anyhow!("Failed to get severity breakdown: {}", e))?;
        let mut severity_breakdown = HashMap::new();
        for row in severity_rows {
            severity_breakdown.insert(row.severity_level, row.count);
        }
        
        log::debug!("Severity breakdown: {:?}", severity_breakdown);
        
        Ok((message_rate, total, type_breakdown, severity_breakdown))
    }

    // Analysis methods for the Alert Analysis dashboard
    pub async fn get_severity_distribution(&self, time_range: &str) -> Result<Vec<serde_json::Value>> {
        let minutes = parse_time_range_to_minutes(time_range);
        let query = format!(r#"
            SELECT 
                CASE 
                    WHEN severity >= 9 THEN '严重'
                    WHEN severity >= 7 THEN '高危'
                    WHEN severity >= 5 THEN '中危'
                    WHEN severity >= 3 THEN '低危'
                    ELSE '信息'
                END as name,
                count(*) as value,
                CASE 
                    WHEN severity >= 9 THEN '#ff4444'
                    WHEN severity >= 7 THEN '#ff8800'
                    WHEN severity >= 5 THEN '#ffcc00'
                    WHEN severity >= 3 THEN '#00aa00'
                    ELSE '#0088cc'
                END as color
            FROM alerts.common_alerts
            WHERE processed_time >= now() - INTERVAL {} MINUTE
            GROUP BY name, color
            ORDER BY 
                CASE name
                    WHEN '严重' THEN 1
                    WHEN '高危' THEN 2
                    WHEN '中危' THEN 3
                    WHEN '低危' THEN 4
                    ELSE 5
                END
        "#, minutes);

        #[derive(Debug, Row, Deserialize)]
        struct SeverityRow {
            name: String,
            value: u64,
            color: String,
        }

        let rows: Vec<SeverityRow> = self.client.query(&query).fetch_all().await?;
        let result: Vec<serde_json::Value> = rows.into_iter().map(|row| {
            serde_json::json!({
                "name": row.name,
                "value": row.value,
                "color": row.color
            })
        }).collect();

        Ok(result)
    }

    pub async fn get_time_series_trend(&self, time_range: &str) -> Result<Vec<serde_json::Value>> {
        let minutes = parse_time_range_to_minutes(time_range);
        let interval = if minutes <= 60 { 15 } else if minutes <= 360 { 30 } else { 60 };
        
        let query = format!(r#"
            SELECT 
                formatDateTime(toStartOfInterval(processed_time, INTERVAL {} MINUTE), '%H:%M') as time,
                count(*) as alerts
            FROM alerts.common_alerts
            WHERE processed_time >= now() - INTERVAL {} MINUTE
            GROUP BY toStartOfInterval(processed_time, INTERVAL {} MINUTE)
            ORDER BY time
        "#, interval, minutes, interval);

        #[derive(Debug, Row, Deserialize)]
        struct TrendRow {
            time: String,
            alerts: u64,
        }

        let rows: Vec<TrendRow> = self.client.query(&query).fetch_all().await?;
        let result: Vec<serde_json::Value> = rows.into_iter().map(|row| {
            serde_json::json!({
                "time": row.time,
                "alerts": row.alerts
            })
        }).collect();

        Ok(result)
    }

    pub async fn get_type_clustering(&self, time_range: &str) -> Result<Vec<serde_json::Value>> {
        let minutes = parse_time_range_to_minutes(time_range);
        let query = format!(r#"
            WITH total_count AS (
                SELECT count(*) as total 
                FROM alerts.common_alerts 
                WHERE processed_time >= now() - INTERVAL {} MINUTE
            )
            SELECT 
                CASE 
                    WHEN data_type = 'edr' THEN 'EDR告警'
                    WHEN data_type = 'ngav' THEN 'NGAV告警'
                    WHEN data_type = 'dns' THEN 'DNS异常'
                    WHEN data_type = 'sysmon' THEN 'Sysmon事件'
                    ELSE '其他'
                END as type,
                count(*) as count,
                round(count(*) * 100.0 / total_count.total, 1) as percentage
            FROM alerts.common_alerts, total_count
            WHERE processed_time >= now() - INTERVAL {} MINUTE
            GROUP BY type, total_count.total
            ORDER BY count DESC
        "#, minutes, minutes);

        #[derive(Debug, Row, Deserialize)]
        struct TypeRow {
            r#type: String,
            count: u64,
            percentage: f64,
        }

        let rows: Vec<TypeRow> = self.client.query(&query).fetch_all().await?;
        let result: Vec<serde_json::Value> = rows.into_iter().map(|row| {
            serde_json::json!({
                "type": row.r#type,
                "count": row.count,
                "percentage": row.percentage
            })
        }).collect();

        Ok(result)
    }
}

// Helper function to parse time range strings to minutes
fn parse_time_range_to_minutes(time_range: &str) -> u32 {
    match time_range {
        "15m" => 15,
        "30m" => 30,
        "1h" => 60,
        "3h" => 180,
        "6h" => 360,
        "12h" => 720,
        "24h" => 1440,
        "3d" => 4320,
        "7d" => 10080,
        _ => 60, // default to 1 hour
    }
}

// Conversion functions from original alert types to ClickHouse row types
impl From<&EdrAlert> for CommonAlert {
    fn from(edr: &EdrAlert) -> Self {

        Self {
            id: Uuid::new_v4().to_string(),
            original_id: edr.get_alert_key(),
            data_type: "edr".to_string(),
            create_time: parse_datetime_for_clickhouse(&edr.create_time),
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
            processed_time: "".to_string(), // Will use ClickHouse DEFAULT now64()
            kafka_topic: "".to_string(),
            kafka_partition: 0,
            kafka_offset: 0,
            kafka_config_name: "".to_string(),
        }
    }
}

impl From<&NgavAlert> for CommonAlert {
    fn from(ngav: &NgavAlert) -> Self {

        Self {
            id: Uuid::new_v4().to_string(),
            original_id: ngav.get_alert_key(),
            data_type: "ngav".to_string(),
            create_time: parse_datetime_for_clickhouse(&ngav.create_time),
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
            processed_time: "".to_string(), // Will use ClickHouse DEFAULT now64()
            kafka_topic: "".to_string(),
            kafka_partition: 0,
            kafka_offset: 0,
            kafka_config_name: "".to_string(),
        }
    }
}

impl From<&EdrAlert> for EdrAlertRow {
    fn from(edr: &EdrAlert) -> Self {

        Self {
            id: edr.get_alert_key(),
            schema: edr.schema as u32,
            create_time: parse_datetime_for_clickhouse(&edr.create_time),
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
            processed_time: "".to_string(), // Will use ClickHouse DEFAULT now64()
            kafka_topic: "".to_string(),
            kafka_partition: 0,
            kafka_offset: 0,
        }
    }
}

impl From<&NgavAlert> for NgavAlertRow {
    fn from(ngav: &NgavAlert) -> Self {

        Self {
            id: ngav.get_alert_key(),
            alert_type: ngav.alert_type.clone(),
            legacy_alert_id: ngav.legacy_alert_id.clone(),
            org_key: ngav.org_key.clone(),
            create_time: parse_datetime_for_clickhouse(&ngav.create_time),
            last_update_time: parse_datetime_for_clickhouse(&ngav.last_update_time),
            first_event_time: parse_datetime_for_clickhouse(&ngav.first_event_time),
            last_event_time: parse_datetime_for_clickhouse(&ngav.last_event_time),
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
            workflow_last_update_time: parse_datetime_for_clickhouse(&ngav.workflow.last_update_time),
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
            processed_time: "".to_string(), // Will use ClickHouse DEFAULT now64()
            kafka_topic: "".to_string(),
            kafka_partition: 0,
            kafka_offset: 0,
        }
    }
}