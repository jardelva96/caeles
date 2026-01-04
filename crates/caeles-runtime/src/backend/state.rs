use super::repository::InMemoryRepository;
use super::storage::{FileStateStore, PersistedState};
use super::tasks::InMemoryTaskQueue;
use std::path::PathBuf;
use std::sync::Arc;

/// Estado compartilhado do backend (repositório + fila de tarefas + persistência opcional).
pub struct AppState {
    pub repo: Arc<InMemoryRepository>,
    pub tasks: Arc<InMemoryTaskQueue>,
    pub store: Option<Arc<FileStateStore>>,
}

impl AppState {
    pub fn new(state_path: Option<PathBuf>) -> anyhow::Result<Self> {
        let repo = Arc::new(InMemoryRepository::new());
        let tasks = Arc::new(InMemoryTaskQueue::new());
        let store = match state_path {
            Some(path) => Some(Arc::new(FileStateStore::load_or_init(path)?)),
            None => None,
        };

        if let Some(store) = &store {
            let persisted = store.load();
            repo.replace_all(persisted.capsules);
            tasks.replace_all(persisted.tasks);
        }

        Ok(Self { repo, tasks, store })
    }

    pub fn snapshot(&self) -> PersistedState {
        PersistedState {
            capsules: self.repo.snapshot(),
            tasks: self.tasks.snapshot(),
        }
    }

    pub fn persist(&self) -> anyhow::Result<()> {
        if let Some(store) = &self.store {
            store.save(&self.snapshot())?;
        }
        Ok(())
    }
}
