//! Planejamento inicial para pipelines de build e ciclo de vida de cápsulas.
//! Estruturas de alto nível para integrar com futuras tarefas (build, publish, deploy).

use serde::{Deserialize, Serialize};

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
pub struct TaskProgress {
    pub task: PlannedTask,
    pub state: TaskState,
    pub detail: Option<String>,
}
