use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapsuleStatus {
    Draft,
    Ready,
    Running,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub entry: PathBuf,
    pub created_at: u64,
    pub updated_at: u64,
    pub status: CapsuleStatus,
}

impl CapsuleMetadata {
    pub fn new(id: String, name: String, version: String, entry: PathBuf) -> Self {
        let now = unix_timestamp();
        Self {
            id,
            name,
            version,
            entry,
            created_at: now,
            updated_at: now,
            status: CapsuleStatus::Draft,
        }
    }

    pub fn touch(&mut self) {
        self.updated_at = unix_timestamp();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleLogEntry {
    pub capsule_id: String,
    pub message: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleArtifact {
    pub capsule_id: String,
    pub kind: String,
    pub path: PathBuf,
    pub created_at: u64,
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
