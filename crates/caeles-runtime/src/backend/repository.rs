use super::model::{CapsuleArtifact, CapsuleLogEntry, CapsuleMetadata, CapsuleStatus};
use crate::manifest::CapsuleManifest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleRecord {
    pub manifest: CapsuleManifest,
    pub meta: CapsuleMetadata,
    pub last_log: Option<CapsuleLogEntry>,
    pub artifacts: Vec<CapsuleArtifact>,
}

pub trait CapsuleRepository: Send + Sync {
    fn create_from_manifest(&self, manifest: CapsuleManifest) -> anyhow::Result<CapsuleRecord>;
    fn list(&self) -> anyhow::Result<Vec<CapsuleRecord>>;
    fn get(&self, id: &str) -> anyhow::Result<Option<CapsuleRecord>>;
    fn update_status(&self, id: &str, status: CapsuleStatus) -> anyhow::Result<()>;
    fn delete(&self, id: &str) -> anyhow::Result<()>;
    fn append_log(&self, log: CapsuleLogEntry) -> anyhow::Result<()>;
    fn add_artifact(&self, artifact: CapsuleArtifact) -> anyhow::Result<()>;
}

#[derive(Default)]
pub struct InMemoryRepository {
    inner: Mutex<HashMap<String, CapsuleRecord>>,
}

impl InMemoryRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn replace_all(&self, records: Vec<CapsuleRecord>) {
        let mut map = self.map();
        map.clear();
        for record in records {
            map.insert(record.meta.id.clone(), record);
        }
    }

    pub fn snapshot(&self) -> Vec<CapsuleRecord> {
        self.map().values().cloned().collect()
    }

    fn map(&self) -> MutexGuard<'_, HashMap<String, CapsuleRecord>> {
        self.inner.lock().expect("mutex poisoned")
    }
}

impl CapsuleRepository for InMemoryRepository {
    fn create_from_manifest(&self, manifest: CapsuleManifest) -> anyhow::Result<CapsuleRecord> {
        let mut map = self.map();
        if map.contains_key(&manifest.id) {
            anyhow::bail!("Cápsula com id '{}' já cadastrada", manifest.id);
        }
        let mut meta = CapsuleMetadata::new(
            manifest.id.clone(),
            manifest.name.clone(),
            manifest.version.clone(),
            PathBuf::from(&manifest.entry),
        );
        meta.status = CapsuleStatus::Ready;
        let record = CapsuleRecord {
            manifest: manifest.clone(),
            meta,
            last_log: None,
            artifacts: Vec::new(),
        };
        map.insert(manifest.id.clone(), record.clone());
        Ok(record)
    }

    fn list(&self) -> anyhow::Result<Vec<CapsuleRecord>> {
        let map = self.map();
        Ok(map.values().cloned().collect())
    }

    fn get(&self, id: &str) -> anyhow::Result<Option<CapsuleRecord>> {
        let map = self.map();
        Ok(map.get(id).cloned())
    }

    fn update_status(&self, id: &str, status: CapsuleStatus) -> anyhow::Result<()> {
        let mut map = self.map();
        if let Some(record) = map.get_mut(id) {
            record.meta.status = status;
            record.meta.touch();
            return Ok(());
        }
        anyhow::bail!("Cápsula '{}' não encontrada", id);
    }

    fn delete(&self, id: &str) -> anyhow::Result<()> {
        let mut map = self.map();
        if map.remove(id).is_some() {
            return Ok(());
        }
        anyhow::bail!("Cápsula '{}' não encontrada", id);
    }

    fn append_log(&self, log: CapsuleLogEntry) -> anyhow::Result<()> {
        let mut map = self.map();
        if let Some(record) = map.get_mut(&log.capsule_id) {
            record.last_log = Some(log);
            record.meta.touch();
            return Ok(());
        }
        anyhow::bail!("Cápsula '{}' não encontrada", log.capsule_id);
    }

    fn add_artifact(&self, artifact: CapsuleArtifact) -> anyhow::Result<()> {
        let mut map = self.map();
        if let Some(record) = map.get_mut(&artifact.capsule_id) {
            record.artifacts.push(artifact);
            record.meta.touch();
            return Ok(());
        }
        anyhow::bail!("Cápsula '{}' não encontrada", artifact.capsule_id);
    }
}
