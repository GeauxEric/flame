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

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Condvar, Mutex};

use chrono::Utc;
use lazy_static::lazy_static;

use crate::model;
use crate::model::{
    Executor, ExecutorID, ExecutorInfo, ExecutorPtr, Session, SessionID, SessionInfo, SessionPtr,
    Task, TaskID, TaskState,
};
use common::ptr::CondPtr;
use common::FlameError;
use common::{lock_cond_ptr, lock_ptr};

mod engine;

lazy_static! {
    static ref INSTANCE: Arc<Storage> = Arc::new(Storage {
        max_ssn_id: Mutex::new(0),
        max_task_ids: Arc::new(Mutex::new(HashMap::new())),
        engine: None,
        sessions: Arc::new(Mutex::new(HashMap::new())),
        executors: Arc::new(Mutex::new(HashMap::new())),
    });
}

pub fn instance() -> Arc<Storage> {
    Arc::clone(&INSTANCE)
}

pub struct Storage {
    max_ssn_id: Mutex<i64>,
    max_task_ids: Arc<Mutex<HashMap<SessionID, Mutex<i64>>>>,
    engine: Option<Arc<dyn engine::Engine>>,
    sessions: Arc<Mutex<HashMap<SessionID, SessionPtr>>>,
    executors: Arc<Mutex<HashMap<ExecutorID, ExecutorPtr>>>,
}

impl Storage {
    fn next_ssn_id(&self) -> Result<i64, FlameError> {
        let mut id = lock_ptr!(self.max_ssn_id)?;
        *id = *id + 1;

        Ok(*id.deref())
    }

    fn next_task_id(&self, ssn_id: &SessionID) -> Result<i64, FlameError> {
        let mut id_list = lock_ptr!(self.max_task_ids)?;
        if !id_list.contains_key(ssn_id) {
            id_list.insert(*ssn_id, Mutex::new(0));
        }

        let id = id_list.get(ssn_id).unwrap();
        let mut id = lock_ptr!(id)?;
        *id = *id + 1;

        Ok(*id.deref())
    }

    pub fn snapshot(&self) -> Result<model::SnapShot, FlameError> {
        let mut res = model::SnapShot {
            sessions: vec![],
            executors: vec![],
        };

        {
            let ssn_map = lock_ptr!(self.sessions)?;

            for (_, ssn) in ssn_map.deref() {
                let ssn = lock_cond_ptr!(ssn)?;
                let info = SessionInfo::from(&(*ssn));
                res.sessions.push(info);
            }
        }

        {
            let exe_map = lock_ptr!(self.executors)?;

            for (_, exe) in exe_map.deref() {
                let exe = lock_cond_ptr!(exe)?;
                res.executors.push(ExecutorInfo::from(&(*exe).clone()));
            }
        }

        Ok(res)
    }

    pub fn create_session(&self, app: String, slots: i32) -> Result<Session, FlameError> {
        let mut ssn_map = lock_ptr!(self.sessions)?;

        let mut ssn = Session::default();
        ssn.id = self.next_ssn_id()?;
        ssn.slots = slots;
        ssn.application = app;
        ssn.creation_time = Utc::now();

        ssn_map.insert(ssn.id, CondPtr::new(ssn.clone()));

        Ok(ssn)
    }

    pub fn get_session(&self, id: SessionID) -> Result<Session, FlameError> {
        let ssn_ptr = self.get_session_ptr(id)?;
        let ssn = lock_cond_ptr!(ssn_ptr)?;
        Ok(ssn.clone())
    }

    fn get_session_ptr(&self, id: SessionID) -> Result<SessionPtr, FlameError> {
        let ssn_map = lock_ptr!(self.sessions)?;
        let ssn = ssn_map
            .get(&id)
            .ok_or(FlameError::NotFound(id.to_string()))?;

        Ok(ssn.clone())
    }

    pub fn delete_session(&self, _id: SessionID) -> Result<(), FlameError> {
        todo!()
    }

    pub fn update_session(&self, _ssn: &Session) -> Result<Session, FlameError> {
        todo!()
    }

    pub fn list_session(&self) -> Result<Vec<Session>, FlameError> {
        let mut ssn_list = vec![];
        let ssn_map = lock_ptr!(self.sessions)?;

        for (_, ssn) in ssn_map.deref() {
            let ssn = lock_cond_ptr!(ssn)?;
            ssn_list.push((*ssn).clone());
        }

        Ok(ssn_list)
    }

    pub fn create_task(
        &self,
        ssn_id: SessionID,
        task_input: Option<String>,
    ) -> Result<Task, FlameError> {
        let ssn_map = lock_ptr!(self.sessions)?;
        let ssn = ssn_map
            .get(&ssn_id)
            .ok_or(FlameError::NotFound(ssn_id.to_string()))?;

        let mut ssn = lock_cond_ptr!(ssn)?;

        let state = TaskState::Pending;
        let task_id = self.next_task_id(&ssn_id)?;

        let task = Task {
            id: task_id,
            ssn_id,
            input: task_input.clone(),
            output: None,
            creation_time: Utc::now(),
            completion_time: None,
            state,
        };

        let task_ptr = CondPtr::new(task.clone());
        ssn.tasks.insert(task_id, task_ptr.clone());
        if !ssn.tasks_index.contains_key(&state) {
            ssn.tasks_index.insert(state, HashMap::new());
        }
        ssn.tasks_index
            .get_mut(&state)
            .unwrap()
            .insert(0, task_ptr.clone());

        Ok(task)
    }

    pub fn get_task(&self, ssn_id: SessionID, id: TaskID) -> Result<Task, FlameError> {
        let ssn_map = lock_ptr!(self.sessions)?;

        let ssn = ssn_map
            .get(&ssn_id)
            .ok_or(FlameError::NotFound(ssn_id.to_string()))?;

        let ssn = lock_cond_ptr!(ssn)?;
        let task = ssn
            .tasks
            .get(&id)
            .ok_or(FlameError::NotFound(id.to_string()))?;
        let task = lock_cond_ptr!(task)?;
        Ok(task.clone())
    }

    pub fn update_task_state(&self, t: &Task) -> Result<Task, FlameError> {
        let ssn_map = lock_ptr!(self.sessions)?;

        let ssn = ssn_map
            .get(&t.ssn_id)
            .ok_or(FlameError::NotFound(t.ssn_id.to_string()))?;

        let ssn = lock_cond_ptr!(ssn)?;
        let task = ssn
            .tasks
            .get(&t.id)
            .ok_or(FlameError::NotFound(t.id.to_string()))?;

        let mut task = lock_cond_ptr!(task)?;
        task.state = t.state;

        Ok((*task).clone())
    }

    // fn delete_task(&self, _ssn_id: SessionID, _id: TaskID) -> Result<(), FlameError> {
    //     todo!()
    // }

    pub fn register_executor(&self, e: &Executor) -> Result<(), FlameError> {
        let mut exe_map = lock_ptr!(self.executors)?;
        let exe = CondPtr::new(e.clone());
        exe_map.insert(e.id.clone(), exe);

        Ok(())
    }

    fn get_executor_ptr(&self, id: ExecutorID) -> Result<ExecutorPtr, FlameError> {
        let exe_map = lock_ptr!(self.executors)?;
        let exe = exe_map
            .get(&id)
            .ok_or(FlameError::NotFound(id.to_string()))?;

        Ok(exe.clone())
    }

    pub fn bind_executor(&self, id: ExecutorID) -> Result<Session, FlameError> {
        let exe_ptr = self.get_executor_ptr(id)?;
        let exe = exe_ptr.wait_while(|e| e.ssn_id.is_some())?;
        let ssn_id = exe
            .ssn_id
            .ok_or(FlameError::Internal("concurrent error".to_string()))?;
        let ssn_ptr = self.get_session_ptr(ssn_id)?;
        let ssn = lock_cond_ptr!(ssn_ptr)?;

        Ok((*ssn).clone())
    }

    pub fn unregister_executor(&self, _id: ExecutorID) -> Result<(), FlameError> {
        todo!()
    }

    pub fn get_executor(&self, _id: ExecutorID) -> Result<Executor, FlameError> {
        todo!()
    }
}
