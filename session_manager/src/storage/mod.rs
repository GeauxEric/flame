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

mod engine;
mod util;

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use chrono::Utc;

use crate::model::{
    Executor, ExecutorID, FlameError, Session, SessionID, SessionState, Task, TaskID,
};

pub fn new() -> Arc<Storage> {
    Arc::new(Storage {
        max_ssn_id: Mutex::new(0),
        engine: None,
        sessions: Arc::new(Mutex::new(HashMap::new())),
        executors: Arc::new(Mutex::new(HashMap::new())),
    })
}

pub struct Storage {
    max_ssn_id: Mutex<i64>,
    engine: Option<Arc<dyn engine::Engine>>,
    sessions: Arc<Mutex<HashMap<SessionID, Arc<Session>>>>,
    executors: Arc<Mutex<HashMap<ExecutorID, Arc<Executor>>>>,
}

pub struct SnapShot {
    pub sessions: Vec<Session>,
    pub executors: Vec<Executor>,
}

impl Storage {
    pub async fn snapshot(&self) -> Result<SnapShot, FlameError> {
        todo!()
    }

    pub async fn create_session(&self, app: String, slots: i32) -> Result<Session, FlameError> {
        let mut ssn_map = self
            .sessions
            .lock()
            .map_err(|_| FlameError::Mutex("mem session".to_string()))?;

        let ssn = Session {
            id: util::next_id(&self.max_ssn_id)?,
            application: app,
            slots,
            tasks: vec![],
            creation_time: Utc::now(),
            completion_time: None,
            state: SessionState::Open,
            desired: 0.0,
            allocated: 0.0,
        };
        let res = ssn.clone();

        ssn_map.insert(ssn.id, Arc::new(ssn));

        Ok(res)
    }

    async fn get_session(&self, id: SessionID) -> Result<Session, FlameError> {
        let mut ssn_map = self
            .sessions
            .lock()
            .map_err(|_| FlameError::Mutex("mem session".to_string()))?;

        let ssn = ssn_map.get(&id);
        match ssn {
            None => Err(FlameError::NotFound(id.to_string())),
            Some(s) => Ok(s.deref().clone()),
        }
    }

    async fn delete_session(&self, id: SessionID) -> Result<(), FlameError> {
        todo!()
    }

    async fn update_session(&self, ssn: &Session) -> Result<Session, FlameError> {
        todo!()
    }

    async fn find_session(&self) -> Result<Vec<Session>, FlameError> {
        todo!()
    }

    async fn create_task(&self, id: SessionID, task_input: &String) -> Result<Task, FlameError> {
        todo!()
    }

    async fn get_task(&self, ssn_id: SessionID, id: TaskID) -> Result<Task, FlameError> {
        todo!()
    }

    async fn delete_task(&self, ssn_id: SessionID, id: TaskID) -> Result<(), FlameError> {
        todo!()
    }

    async fn update_task(&self, t: &Task) -> Result<Task, FlameError> {
        todo!()
    }

    async fn register_executor(&self, e: &Executor) -> Result<(), FlameError> {
        todo!()
    }

    async fn get_executor(&self, id: ExecutorID) -> Result<Executor, FlameError> {
        todo!()
    }

    async fn unregister_executor(&self, id: ExecutorID) -> Result<(), FlameError> {
        todo!()
    }

    async fn update_executor(&self, e: &Executor) -> Result<Executor, FlameError> {
        todo!()
    }
}
