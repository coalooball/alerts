use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Workflow {
    pub state: String,
    pub remediation: String,
    pub last_update_time: String,
    pub comment: String,
    pub changed_by: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThreatIndicator {
    pub process_name: String,
    pub sha256: String,
    pub ttps: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NgavAlert {
    #[serde(rename = "type")]
    pub alert_type: String,
    pub id: String,
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
    pub workflow: Workflow,
    pub device_internal_ip: String,
    pub device_external_ip: String,
    pub alert_url: String,
    pub reason: String,
    pub reason_code: String,
    pub process_name: String,
    pub device_location: String,
    pub created_by_event_id: String,
    pub threat_indicators: Vec<ThreatIndicator>,
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
}

impl NgavAlert {
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
        format!("{}_{}", self.device_name, self.id)
    }

    pub fn is_critical(&self) -> bool {
        self.severity <= 2
    }

    pub fn is_malware(&self) -> bool {
        !self.threat_cause_threat_category.contains("NON_MALWARE")
    }

    pub fn has_mitre_ttps(&self) -> bool {
        self.threat_indicators.iter().any(|ti| 
            ti.ttps.iter().any(|ttp| ttp.starts_with("MITRE_"))
        )
    }

    pub fn get_mitre_ttps(&self) -> Vec<String> {
        self.threat_indicators.iter()
            .flat_map(|ti| &ti.ttps)
            .filter(|ttp| ttp.starts_with("MITRE_"))
            .cloned()
            .collect()
    }

    pub fn get_threat_summary(&self) -> String {
        format!("{} - {} ({})", 
                self.reason, 
                self.threat_cause_actor_name,
                self.threat_cause_threat_category)
    }

    pub fn is_blocked(&self) -> bool {
        self.policy_applied == "APPLIED" || 
        !self.blocked_threat_category.eq("UNKNOWN")
    }

    pub fn get_affected_processes(&self) -> Vec<String> {
        self.threat_indicators.iter()
            .map(|ti| ti.process_name.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }
}