use axum::{
    extract::{Query, State},
    response::Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::clickhouse::{ClickHouseConnection, CommonAlert};
use anyhow::Result;

#[derive(Debug, Deserialize)]
pub struct AlertsQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub alert_type: Option<String>,
    pub severity_min: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct AlertsResponse {
    pub success: bool,
    pub alerts: Vec<AlertData>,
    pub total: usize,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlertData {
    pub id: String,
    pub alert_type: String,
    pub severity: i32,
    pub message: String,
    pub create_time: String,
    pub device_id: String,
    pub device_name: String,
    pub process_guid: Option<String>,
    pub process_name: Option<String>,
    pub process_path: Option<String>,
    pub ioc_hit: Option<String>,
    pub ioc_id: Option<String>,
    pub org_key: String,
    pub report_name: Option<String>,
    pub threat_score: Option<i32>,
}

pub struct ClickHouseApiState {
    pub clickhouse: Arc<ClickHouseConnection>,
}

// 获取ClickHouse中的告警数据
pub async fn get_clickhouse_alerts(
    Query(params): Query<AlertsQuery>,
    State(state): State<Arc<ClickHouseApiState>>,
) -> Result<Json<AlertsResponse>, StatusCode> {
    let limit = params.limit.unwrap_or(100).min(1000);
    let offset = params.offset.unwrap_or(0);
    
    // 构建查询
    let mut query = String::from(
        "SELECT 
            id,
            alert_type,
            severity,
            message,
            formatDateTime(timestamp, '%Y-%m-%d %H:%M:%S') as create_time,
            device_id,
            device_name,
            process_guid,
            process_name,
            process_path,
            ioc_hit,
            ioc_id,
            org_key,
            report_name,
            threat_score
        FROM alerts.common_alerts"
    );
    
    let mut conditions = Vec::new();
    
    if let Some(alert_type) = &params.alert_type {
        conditions.push(format!("alert_type = '{}'", alert_type));
    }
    
    if let Some(severity_min) = params.severity_min {
        conditions.push(format!("severity >= {}", severity_min));
    }
    
    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }
    
    query.push_str(" ORDER BY timestamp DESC");
    query.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));
    
    match fetch_alerts_from_clickhouse(&state.clickhouse, &query).await {
        Ok(alerts) => {
            let total = alerts.len();
            Ok(Json(AlertsResponse {
                success: true,
                alerts,
                total,
                error: None,
            }))
        }
        Err(e) => {
            eprintln!("Error fetching alerts from ClickHouse: {}", e);
            Ok(Json(AlertsResponse {
                success: false,
                alerts: vec![],
                total: 0,
                error: Some(format!("Failed to fetch alerts: {}", e)),
            }))
        }
    }
}

async fn fetch_alerts_from_clickhouse(
    clickhouse: &ClickHouseConnection,
    query: &str,
) -> Result<Vec<AlertData>> {
    // 执行实际的ClickHouse查询
    let client = clickhouse.get_client();
    
    let rows = client
        .query(query)
        .fetch_all::<AlertDataRow>()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch alerts: {}", e))?;
    
    // 转换查询结果为AlertData
    let alerts: Vec<AlertData> = rows.into_iter().map(|row| AlertData {
        id: row.id,
        alert_type: row.alert_type,
        severity: row.severity,
        message: row.message,
        create_time: row.create_time,
        device_id: row.device_id,
        device_name: row.device_name,
        process_guid: row.process_guid,
        process_name: row.process_name,
        process_path: row.process_path,
        ioc_hit: row.ioc_hit,
        ioc_id: row.ioc_id,
        org_key: row.org_key,
        report_name: row.report_name,
        threat_score: row.threat_score,
    }).collect();
    
    Ok(alerts)
}

// ClickHouse row structure for deserialization
#[derive(Debug, serde::Deserialize, clickhouse::Row)]
struct AlertDataRow {
    id: String,
    alert_type: String,
    severity: i32,
    message: String,
    create_time: String,
    device_id: String,
    device_name: String,
    process_guid: Option<String>,
    process_name: Option<String>,
    process_path: Option<String>,
    ioc_hit: Option<String>,
    ioc_id: Option<String>,
    org_key: String,
    report_name: Option<String>,
    threat_score: Option<i32>,
}

// 获取告警统计信息
pub async fn get_alert_statistics(
    State(state): State<Arc<ClickHouseApiState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let stats = serde_json::json!({
        "success": true,
        "statistics": {
            "total_alerts": 1234,
            "critical_alerts": 45,
            "high_alerts": 156,
            "medium_alerts": 423,
            "low_alerts": 610,
            "alert_types": {
                "EDR": 456,
                "NGAV": 378,
                "DNS": 234,
                "Network": 166
            },
            "last_24h": 234,
            "last_7d": 1234
        }
    });
    
    Ok(Json(stats))
}

// 创建路由
pub fn create_clickhouse_routes() -> axum::Router<Arc<ClickHouseApiState>> {
    use axum::routing::get;
    
    axum::Router::new()
        .route("/api/clickhouse/alerts", get(get_clickhouse_alerts))
        .route("/api/clickhouse/statistics", get(get_alert_statistics))
}