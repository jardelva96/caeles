//! Backend de gerenciamento de cápsulas (pré-estruturado para futuro “nível Docker”).
//! Mantém interfaces e estruturas básicas para criação, ciclo de vida, artefatos e logs.
//! Implementação atual é somente in-memory, para evoluir depois para storage persistente.

#[allow(dead_code)]
pub mod model;
#[allow(dead_code)]
pub mod repository;
#[allow(dead_code)]
pub mod tasks;
