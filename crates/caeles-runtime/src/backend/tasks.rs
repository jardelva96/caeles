//! Planejamento inicial para pipelines de build e ciclo de vida de cápsulas.
//! Estruturas de alto nível para integrar com futuras tarefas (build, publish, deploy).

use serde::{Deserialize, Serialize};
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskKind {
    Build,
    Publish,
    Deploy,
    Start,
    Stop,
    Remove,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedTask {
    pub capsule_id: String,
    pub kind: TaskKind,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Queued,
    Running,
    Done,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub id: String,
    pub task: PlannedTask,
    pub state: TaskState,
    pub detail: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

pub trait TaskQueue: Send + Sync {
    fn enqueue(&self, task: PlannedTask) -> anyhow::Result<TaskInfo>;
    fn list(&self) -> anyhow::Result<Vec<TaskInfo>>;
    fn mark_running(&self, id: &str, detail: Option<String>) -> anyhow::Result<()>;
    fn mark_done(&self, id: &str, detail: Option<String>) -> anyhow::Result<()>;
    fn mark_failed(&self, id: &str, detail: Option<String>) -> anyhow::Result<()>;
}

#[derive(Default)]
pub struct InMemoryTaskQueue {
    inner: Mutex<Vec<TaskInfo>>,
    counter: Mutex<u64>,
}

impl InMemoryTaskQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn replace_all(&self, tasks: Vec<TaskInfo>) {
        let mut guard = self.inner();
        *guard = tasks;
        self.reset_counter_from(&guard);
    }

    pub fn snapshot(&self) -> Vec<TaskInfo> {
        self.inner().clone()
    }

    fn inner(&self) -> MutexGuard<'_, Vec<TaskInfo>> {
        self.inner.lock().expect("mutex poisoned")
    }

    fn next_id(&self) -> String {
        let mut guard = self.counter.lock().expect("mutex poisoned");
        *guard += 1;
        format!("task-{}", guard)
    }

    fn reset_counter_from(&self, tasks: &[TaskInfo]) {
        let mut guard = self.counter.lock().expect("mutex poisoned");
        let max_id = tasks
            .iter()
            .filter_map(|t| t.id.strip_prefix("task-"))
            .filter_map(|n| n.parse::<u64>().ok())
            .max()
            .unwrap_or(0);
        *guard = max_id;
    }

    fn transition(&self, id: &str, state: TaskState, detail: Option<String>) -> anyhow::Result<()> {
        let mut guard = self.inner();
        if let Some(task) = guard.iter_mut().find(|t| t.id == id) {
            task.state = state;
            task.updated_at = unix_timestamp();
            task.detail = detail;
            return Ok(());
        }
        anyhow::bail!("Tarefa '{}' não encontrada", id);
    }
}

impl TaskQueue for InMemoryTaskQueue {
    fn enqueue(&self, task: PlannedTask) -> anyhow::Result<TaskInfo> {
        let now = unix_timestamp();
        let info = TaskInfo {
            id: self.next_id(),
            task,
            state: TaskState::Queued,
            detail: None,
            created_at: now,
            updated_at: now,
        };
        self.inner().push(info.clone());
        Ok(info)
    }

    fn list(&self) -> anyhow::Result<Vec<TaskInfo>> {
        Ok(self.inner().clone())
    }

    fn mark_running(&self, id: &str, detail: Option<String>) -> anyhow::Result<()> {
        self.transition(id, TaskState::Running, detail)
    }

    fn mark_done(&self, id: &str, detail: Option<String>) -> anyhow::Result<()> {
        self.transition(id, TaskState::Done, detail)
    }

    fn mark_failed(&self, id: &str, detail: Option<String>) -> anyhow::Result<()> {
        self.transition(id, TaskState::Failed, detail)
    }
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
