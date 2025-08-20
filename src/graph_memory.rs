// 内存图数据库实现 - 无需安装外部数据库
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::{dijkstra, all_simple_paths};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertNode {
    pub id: String,
    pub alert_type: String,
    pub severity: u8,
    pub device_id: String,
    pub timestamp: String,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationEdge {
    pub relation_type: String,
    pub weight: f32,
    pub properties: HashMap<String, String>,
}

pub struct InMemoryGraphDB {
    graph: DiGraph<AlertNode, RelationEdge>,
    node_index_map: HashMap<String, NodeIndex>,
}

impl InMemoryGraphDB {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_index_map: HashMap::new(),
        }
    }

    // 添加告警节点
    pub fn add_alert(&mut self, alert: AlertNode) -> NodeIndex {
        let alert_id = alert.id.clone();
        let node_idx = self.graph.add_node(alert);
        self.node_index_map.insert(alert_id, node_idx);
        node_idx
    }

    // 添加关联关系
    pub fn add_relation(&mut self, from_id: &str, to_id: &str, relation: RelationEdge) {
        if let (Some(&from_idx), Some(&to_idx)) = 
            (self.node_index_map.get(from_id), self.node_index_map.get(to_id)) {
            self.graph.add_edge(from_idx, to_idx, relation);
        }
    }

    // 查找相关告警
    pub fn find_related_alerts(&self, alert_id: &str, max_depth: usize) -> Vec<AlertNode> {
        let mut result = Vec::new();
        
        if let Some(&start_idx) = self.node_index_map.get(alert_id) {
            // 使用BFS查找指定深度内的所有节点
            let mut visited = HashMap::new();
            let mut queue = vec![(start_idx, 0)];
            
            while let Some((node_idx, depth)) = queue.pop() {
                if depth > max_depth {
                    continue;
                }
                
                if visited.contains_key(&node_idx) {
                    continue;
                }
                
                visited.insert(node_idx, depth);
                
                if let Some(node) = self.graph.node_weight(node_idx) {
                    result.push(node.clone());
                }
                
                // 添加邻居节点
                for neighbor in self.graph.neighbors(node_idx) {
                    if !visited.contains_key(&neighbor) {
                        queue.push((neighbor, depth + 1));
                    }
                }
            }
        }
        
        result
    }

    // 导出为前端可视化格式
    pub fn export_to_vis_format(&self, center_alert_id: &str, max_depth: usize) -> VisGraphData {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        
        let related = self.find_related_alerts(center_alert_id, max_depth);
        
        // 构建节点
        for alert in related {
            nodes.push(VisNode {
                id: alert.id.clone(),
                label: format!("{} ({})", alert.alert_type, alert.severity),
                group: alert.severity.to_string(),
                title: format!("Device: {}\nTime: {}", alert.device_id, alert.timestamp),
            });
        }
        
        // 构建边
        for edge_ref in self.graph.edge_references() {
            let source_node = &self.graph[edge_ref.source()];
            let target_node = &self.graph[edge_ref.target()];
            let edge_weight = edge_ref.weight();
            
            edges.push(VisEdge {
                from: source_node.id.clone(),
                to: target_node.id.clone(),
                label: edge_weight.relation_type.clone(),
                value: edge_weight.weight,
            });
        }
        
        VisGraphData { nodes, edges }
    }
}

// 前端可视化数据格式 (vis.js格式)
#[derive(Debug, Serialize, Deserialize)]
pub struct VisGraphData {
    pub nodes: Vec<VisNode>,
    pub edges: Vec<VisEdge>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VisNode {
    pub id: String,
    pub label: String,
    pub group: String,
    pub title: String,  // hover时显示
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VisEdge {
    pub from: String,
    pub to: String,
    pub label: String,
    pub value: f32,
}

// 测试和示例
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_operations() {
        let mut graph = InMemoryGraphDB::new();
        
        // 添加告警节点
        let alert1 = AlertNode {
            id: "alert1".to_string(),
            alert_type: "EDR".to_string(),
            severity: 8,
            device_id: "device1".to_string(),
            timestamp: "2024-01-01T10:00:00Z".to_string(),
            properties: HashMap::new(),
        };
        
        let alert2 = AlertNode {
            id: "alert2".to_string(),
            alert_type: "NGAV".to_string(),
            severity: 6,
            device_id: "device1".to_string(),
            timestamp: "2024-01-01T10:05:00Z".to_string(),
            properties: HashMap::new(),
        };
        
        graph.add_alert(alert1);
        graph.add_alert(alert2);
        
        // 添加关联关系
        let relation = RelationEdge {
            relation_type: "same_device".to_string(),
            weight: 0.8,
            properties: HashMap::new(),
        };
        
        graph.add_relation("alert1", "alert2", relation);
        
        // 查找相关告警
        let related = graph.find_related_alerts("alert1", 2);
        assert_eq!(related.len(), 2);
    }
}