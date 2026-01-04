use super::repository::CapsuleRecord;
use super::tasks::TaskInfo;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersistedState {
    pub capsules: Vec<CapsuleRecord>,
    pub tasks: Vec<TaskInfo>,
}

/// Armazena o estado do gerenciador em disco (JSON) para sobreviver a reinícios.
pub struct FileStateStore {
    path: PathBuf,
    inner: Mutex<PersistedState>,
}

impl FileStateStore {
    pub fn load_or_init(path: PathBuf) -> anyhow::Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let state = if path.exists() {
            let data = fs::read_to_string(&path)?;
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            PersistedState::default()
        };

        Ok(Self {
            path,
            inner: Mutex::new(state),
        })
    }

    pub fn load(&self) -> PersistedState {
        self.inner.lock().expect("mutex poisoned").clone()
    }

    pub fn save(&self, state: &PersistedState) -> anyhow::Result<()> {
        let mut guard = self.inner.lock().expect("mutex poisoned");
        *guard = state.clone();
        let json = serde_json::to_string_pretty(&*guard)?;
        fs::write(&self.path, json)?;
        Ok(())
    }
}
