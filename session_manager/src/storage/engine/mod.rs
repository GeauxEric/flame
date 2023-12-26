/*
Copyright 2023 The xflops Authors.
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at
    http://www.apache.org/licenses/LICENSE-2.0
Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use std::sync::Arc;

use async_trait::async_trait;

use crate::FlameError;
use common::apis::{Session, SessionID, Task, TaskID};

mod sqlite;

pub type EnginePtr = Arc<dyn Engine>;

#[async_trait]
pub trait Engine: Send + Sync + 'static {
    async fn persist_session(&self, ssn: &Session) -> Result<(), FlameError>;
    async fn get_session(&self, id: SessionID) -> Result<Session, FlameError>;
    async fn delete_session(&self, id: SessionID) -> Result<(), FlameError>;
    async fn update_session(&self, ssn: &Session) -> Result<(), FlameError>;
    async fn find_session(&self) -> Result<Vec<Session>, FlameError>;

    async fn persist_task(&self, task: &Task) -> Result<(), FlameError>;
    async fn get_task(&self, ssn_id: SessionID, id: TaskID) -> Result<Task, FlameError>;
    async fn delete_task(&self, ssn_id: SessionID, id: TaskID) -> Result<(), FlameError>;
    async fn update_task(&self, t: &Task) -> Result<(), FlameError>;
}

pub async fn connect() -> Result<EnginePtr, FlameError> {
    Ok(sqlite::SqliteEngine::new_ptr()?)
}
