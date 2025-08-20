use anyhow::{Result, Context};
use neo4rs::{Graph, query, Query};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

// Neo4j配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neo4jConfig {
    pub uri: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

impl Default for Neo4jConfig {
    fn default() -> Self {
        Self {
            uri: "bolt://localhost:7687".to_string(),
            username: "neo4j".to_string(),
            password: "alerts123".to_string(),
            database: "alerts".to_string(),
        }
    }
}

// 告警节点数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertNodeData {
    pub id: String,
    pub alert_type: String,
    pub severity: i64,
    pub create_time: String,
    pub device_id: String,
    pub device_name: String,
    pub process_guid: String,
    pub process_path: String,
    pub org_key: String,
    pub report_name: String,
    pub ioc_hit: Option<String>,
}

// 关联关系数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipData {
    pub from_id: String,
    pub to_id: String,
    pub relation_type: String,
    pub correlation_score: f64,
    pub correlation_reason: String,
}

// 图谱数据结构（用于前端展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphVisualizationData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub statistics: GraphStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: String,  // Alert, Device, Process, IOC, User
    pub group: i64,          // 用于颜色分组（基于severity）
    pub size: f64,           // 节点大小
    pub title: String,       // hover提示
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub label: String,
    pub edge_type: String,
    pub width: f64,          // 边的粗细（基于correlation_score）
    pub arrows: String,      // "to", "from", "to,from"
    pub dashes: bool,        // 是否虚线
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStatistics {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub node_types: HashMap<String, usize>,
    pub edge_types: HashMap<String, usize>,
    pub max_severity: i64,
}

// Neo4j客户端
pub struct Neo4jClient {
    graph: Arc<Graph>,
}

impl Neo4jClient {
    pub async fn new(config: Neo4jConfig) -> Result<Self> {
        let graph = Graph::new(
            &config.uri,
            &config.username,
            &config.password,
        ).await.context("Failed to connect to Neo4j")?;
        
        Ok(Self {
            graph: Arc::new(graph),
        })
    }

    // 初始化数据库约束和索引
    pub async fn init_database(&self) -> Result<()> {
        // 创建唯一性约束
        let constraints = vec![
            "CREATE CONSTRAINT IF NOT EXISTS FOR (a:Alert) REQUIRE a.id IS UNIQUE",
            "CREATE CONSTRAINT IF NOT EXISTS FOR (d:Device) REQUIRE d.device_id IS UNIQUE",
            "CREATE CONSTRAINT IF NOT EXISTS FOR (p:Process) REQUIRE p.process_guid IS UNIQUE",
            "CREATE CONSTRAINT IF NOT EXISTS FOR (i:IOC) REQUIRE i.ioc_id IS UNIQUE",
            "CREATE CONSTRAINT IF NOT EXISTS FOR (u:User) REQUIRE u.username IS UNIQUE",
        ];

        for constraint in constraints {
            self.graph.run(query(constraint)).await?;
        }

        // 创建索引以提高查询性能
        let indexes = vec![
            "CREATE INDEX IF NOT EXISTS FOR (a:Alert) ON (a.severity)",
            "CREATE INDEX IF NOT EXISTS FOR (a:Alert) ON (a.create_time)",
            "CREATE INDEX IF NOT EXISTS FOR (a:Alert) ON (a.alert_type)",
            "CREATE INDEX IF NOT EXISTS FOR (d:Device) ON (d.device_name)",
            "CREATE INDEX IF NOT EXISTS FOR (p:Process) ON (p.process_path)",
        ];

        for index in indexes {
            self.graph.run(query(index)).await?;
        }

        println!("Neo4j database initialized with constraints and indexes");
        Ok(())
    }

    // 创建告警节点并建立关系
    pub async fn create_alert_with_relations(&self, alert: &crate::edr_alert::EdrAlert) -> Result<String> {
        let alert_id = format!("alert_{}", alert.report_id);
        
        // 构建创建告警及其关系的查询
        let query_str = r#"
            // 创建或合并告警节点
            MERGE (a:Alert {id: $alert_id})
            SET a.alert_type = $alert_type,
                a.severity = $severity,
                a.create_time = $create_time,
                a.org_key = $org_key,
                a.report_name = $report_name,
                a.ioc_hit = $ioc_hit
            
            // 创建或合并设备节点
            MERGE (d:Device {device_id: $device_id})
            SET d.device_name = $device_name,
                d.device_os = $device_os,
                d.internal_ip = $internal_ip,
                d.external_ip = $external_ip
            
            // 创建或合并进程节点
            MERGE (p:Process {process_guid: $process_guid})
            SET p.process_path = $process_path,
                p.process_cmdline = $process_cmdline,
                p.process_pid = $process_pid,
                p.username = $username
            
            // 建立关系
            MERGE (a)-[:TRIGGERED_ON]->(d)
            MERGE (a)-[:INVOLVES_PROCESS]->(p)
            
            // 如果有父进程，创建父子关系
            WITH a, d, p
            CALL {
                WITH p
                MATCH (parent:Process {process_guid: $parent_guid})
                MERGE (parent)-[:PARENT_OF]->(p)
            }
            
            RETURN a.id as alert_id
        "#;

        let query = Query::new(query_str.to_string())
            .param("alert_id", alert_id.clone())
            .param("alert_type", alert.alert_type.clone())
            .param("severity", alert.severity as i64)
            .param("create_time", alert.create_time.clone())
            .param("org_key", alert.org_key.clone())
            .param("report_name", alert.report_name.clone())
            .param("ioc_hit", alert.ioc_hit.clone())
            .param("device_id", alert.device_id.to_string())
            .param("device_name", alert.device_name.clone())
            .param("device_os", alert.device_os.clone())
            .param("internal_ip", alert.device_internal_ip.clone())
            .param("external_ip", alert.device_external_ip.clone())
            .param("process_guid", alert.process_guid.clone())
            .param("process_path", alert.process_path.clone())
            .param("process_cmdline", alert.process_cmdline.clone())
            .param("process_pid", alert.process_pid as i64)
            .param("username", alert.process_username.clone())
            .param("parent_guid", alert.parent_guid.clone());

        self.graph.run(query).await?;
        
        Ok(alert_id)
    }

    // 自动关联相似告警
    pub async fn correlate_alerts(&self, time_window_hours: i64) -> Result<i64> {
        let query_str = r#"
            // 查找时间窗口内的告警
            MATCH (a1:Alert), (a2:Alert)
            WHERE a1.id < a2.id
              AND datetime(a1.create_time) > datetime() - duration({hours: $hours})
              AND datetime(a2.create_time) > datetime() - duration({hours: $hours})
              
            // 计算相似度
            WITH a1, a2,
                 CASE
                   // 同一设备
                   WHEN EXISTS((a1)-[:TRIGGERED_ON]->()<-[:TRIGGERED_ON]-(a2)) THEN 0.3
                   ELSE 0
                 END +
                 CASE
                   // 相同IOC
                   WHEN a1.ioc_hit IS NOT NULL AND a1.ioc_hit = a2.ioc_hit THEN 0.4
                   ELSE 0
                 END +
                 CASE
                   // 相同告警类型
                   WHEN a1.alert_type = a2.alert_type THEN 0.2
                   ELSE 0
                 END +
                 CASE
                   // 相似严重程度
                   WHEN abs(a1.severity - a2.severity) <= 2 THEN 0.1
                   ELSE 0
                 END as correlation_score
                 
            WHERE correlation_score >= 0.5
            
            // 创建关联关系
            MERGE (a1)-[r:CORRELATED_WITH]->(a2)
            SET r.correlation_score = correlation_score,
                r.correlation_time = datetime(),
                r.correlation_type = CASE
                    WHEN EXISTS((a1)-[:TRIGGERED_ON]->()<-[:TRIGGERED_ON]-(a2)) THEN 'same_device'
                    WHEN a1.ioc_hit = a2.ioc_hit THEN 'same_ioc'
                    ELSE 'behavioral'
                END
                
            RETURN count(r) as correlations_created
        "#;

        let query = Query::new(query_str.to_string())
            .param("hours", time_window_hours);
        
        let mut result = self.graph.execute(query).await?;
        
        if let Some(row) = result.next().await? {
            let count: i64 = row.get("correlations_created")?;
            Ok(count)
        } else {
            Ok(0)
        }
    }

    // 获取告警的关联图谱
    pub async fn get_alert_graph(&self, alert_id: &str, depth: i64) -> Result<GraphVisualizationData> {
        let query_str = r#"
            // 获取中心告警及其关联
            MATCH path = (center:Alert {id: $alert_id})-[*0..$depth]-(connected)
            WITH center, connected, relationships(path) as rels, nodes(path) as nodes_in_path
            
            // 收集所有节点和关系
            WITH 
                collect(DISTINCT center) + collect(DISTINCT connected) as all_nodes,
                collect(DISTINCT rels) as all_rels
            
            UNWIND all_nodes as node
            
            // 返回节点和关系
            WITH node, all_rels,
                 labels(node)[0] as node_label,
                 properties(node) as node_props
                 
            RETURN 
                collect(DISTINCT {
                    id: coalesce(node_props.id, node_props.device_id, node_props.process_guid, node_props.ioc_id, toString(id(node))),
                    label: node_label,
                    properties: node_props
                }) as nodes,
                all_rels as relationships
        "#;

        let query = Query::new(query_str.to_string())
            .param("alert_id", alert_id)
            .param("depth", depth);
        
        let mut result = self.graph.execute(query).await?;
        
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut statistics = GraphStatistics {
            total_nodes: 0,
            total_edges: 0,
            node_types: HashMap::new(),
            edge_types: HashMap::new(),
            max_severity: 0,
        };

        // 处理查询结果
        if let Some(row) = result.next().await? {
            // 处理节点
            if let Ok(node_list) = row.get::<Vec<serde_json::Value>>("nodes") {
                for node_data in node_list {
                    if let Some(obj) = node_data.as_object() {
                        let node = self.parse_node(obj)?;
                        
                        // 更新统计
                        *statistics.node_types.entry(node.node_type.clone()).or_insert(0) += 1;
                        if node.node_type == "Alert" {
                            statistics.max_severity = statistics.max_severity.max(node.group);
                        }
                        
                        nodes.push(node);
                    }
                }
            }
            
            // 处理关系（edges）
            // 这里需要更复杂的处理来提取关系信息
        }

        statistics.total_nodes = nodes.len();
        statistics.total_edges = edges.len();

        Ok(GraphVisualizationData {
            nodes,
            edges,
            statistics,
        })
    }

    // 解析节点数据
    fn parse_node(&self, node_data: &serde_json::Map<String, serde_json::Value>) -> Result<GraphNode> {
        let label = node_data.get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();
        
        let properties = node_data.get("properties")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        
        let id = properties.get("id")
            .or_else(|| properties.get("device_id"))
            .or_else(|| properties.get("process_guid"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        let severity = properties.get("severity")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        
        let node_label = match label.as_str() {
            "Alert" => format!("Alert: {}", 
                properties.get("alert_type").and_then(|v| v.as_str()).unwrap_or("Unknown")),
            "Device" => format!("Device: {}",
                properties.get("device_name").and_then(|v| v.as_str()).unwrap_or("Unknown")),
            "Process" => format!("Process: {}",
                properties.get("process_path").and_then(|v| v.as_str())
                    .and_then(|p| p.split('/').last()).unwrap_or("Unknown")),
            _ => label.clone(),
        };
        
        let title = format!("{}\nID: {}\nProperties: {:?}", label, id, properties);
        
        let node_size = if label == "Alert" { 30.0 } else { 20.0 };
        
        Ok(GraphNode {
            id,
            label: node_label,
            node_type: label.clone(),
            group: severity,
            size: node_size,
            title,
            properties: properties.into_iter()
                .map(|(k, v)| (k, v.clone()))
                .collect(),
        })
    }

    // 检测横向移动
    pub async fn detect_lateral_movement(&self, org_key: &str, hours: i64) -> Result<Vec<String>> {
        let query_str = r#"
            // 查找横向移动模式
            MATCH (d1:Device)<-[:TRIGGERED_ON]-(a1:Alert {org_key: $org_key})
            WHERE datetime(a1.create_time) > datetime() - duration({hours: $hours})
            
            MATCH path = (a1)-[:CORRELATED_WITH*1..3]-(a2:Alert)-[:TRIGGERED_ON]->(d2:Device)
            WHERE d1.device_id <> d2.device_id
              AND a2.org_key = $org_key
              
            // 返回可疑的横向移动路径
            RETURN DISTINCT 
                d1.device_name as source_device,
                d2.device_name as target_device,
                length(path) as hop_count,
                [n in nodes(path) WHERE 'Alert' IN labels(n) | n.id] as alert_chain
            ORDER BY hop_count
        "#;

        let query = Query::new(query_str.to_string())
            .param("org_key", org_key)
            .param("hours", hours);
        
        let mut result = self.graph.execute(query).await?;
        let mut movements = Vec::new();
        
        while let Some(row) = result.next().await? {
            let source: String = row.get("source_device")?;
            let target: String = row.get("target_device")?;
            let hop_count: i64 = row.get("hop_count")?;
            
            movements.push(format!(
                "Lateral movement detected: {} -> {} ({}  hops)",
                source, target, hop_count
            ));
        }
        
        Ok(movements)
    }

    // 清理过期数据
    pub async fn cleanup_old_data(&self, days: i64) -> Result<i64> {
        let query_str = r#"
            MATCH (a:Alert)
            WHERE datetime(a.create_time) < datetime() - duration({days: $days})
            DETACH DELETE a
            RETURN count(a) as deleted_count
        "#;

        let query = Query::new(query_str.to_string())
            .param("days", days);
        
        let mut result = self.graph.execute(query).await?;
        
        if let Some(row) = result.next().await? {
            let count: i64 = row.get("deleted_count")?;
            Ok(count)
        } else {
            Ok(0)
        }
    }
}