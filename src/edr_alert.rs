use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Publisher {
    pub name: String,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Watchlist {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EdrAlert {
    pub schema: i32,
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
    pub parent_publisher: Vec<Publisher>,
    pub parent_reputation: String,
    pub parent_username: String,
    pub process_cmdline: String,
    pub process_guid: String,
    pub process_hash: Vec<String>,
    pub process_path: String,
    pub process_pid: u32,
    pub process_publisher: Vec<Publisher>,
    pub process_reputation: String,
    pub process_username: String,
    pub report_id: String,
    pub report_name: String,
    pub report_tags: Vec<String>,
    pub severity: u8,
    #[serde(rename = "type")]
    pub alert_type: String,
    pub watchlists: Vec<Watchlist>,
}

impl EdrAlert {
    pub fn get_severity_level(&self) -> &str {
        match self.severity {
            1 => "critical",
            2 => "high", 
            3 => "medium",
            4 => "low",
            _ => "unknown",
        }
    }

    pub fn get_alert_key(&self) -> String {
        format!("{}_{}", self.device_name, self.report_id)
    }

    pub fn is_critical(&self) -> bool {
        self.severity <= 2
    }

    pub fn contains_tag(&self, tag: &str) -> bool {
        self.report_tags.iter().any(|t| t.contains(tag))
    }

    pub fn get_process_info(&self) -> String {
        format!("{}[{}] - {}", 
                self.process_path, 
                self.process_pid, 
                self.process_cmdline)
    }
}