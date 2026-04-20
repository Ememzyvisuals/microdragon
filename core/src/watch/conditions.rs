// microdragon-core/src/watch/conditions.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchCondition {
    pub id: String,
    pub condition: String,
    pub action: String,
    pub condition_type: ConditionType,
    pub alert_level: AlertLevel,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    MarketPrice,
    TimeSchedule,
    EmailReceived,
    FileChange,
    SystemMetric,
    WebContent,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertLevel {
    Silent,   // Log only
    Normal,   // CLI notification
    Urgent,   // CLI + social notification
}
