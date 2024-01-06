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

use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll};

use common::apis::{
    CommonData, Executor, ExecutorID, ExecutorPtr, Session, SessionID, SessionPtr, SessionState,
    Task, TaskGID, TaskID, TaskInput, TaskOutput, TaskPtr, TaskState,
};
use common::ptr::{self, MutexPtr};
use common::{lock_ptr, trace::TraceFn, trace_fn, FlameError};

use crate::model::{ExecutorInfo, SessionInfo, SnapShot, SnapShotPtr};
use crate::storage::engine::EnginePtr;

mod engine;
mod states;

pub type StoragePtr = Arc<Storage>;

#[derive(Clone)]
pub struct Storage {
    engine: EnginePtr,
    sessions: MutexPtr<HashMap<SessionID, SessionPtr>>,
    executors: MutexPtr<HashMap<ExecutorID, ExecutorPtr>>,
}

pub async fn new_ptr() -> Result<StoragePtr, FlameError> {
    Ok(Arc::new(Storage {
        engine: engine::connect().await?,
        sessions: ptr::new_ptr(HashMap::new()),
        executors: ptr::new_ptr(HashMap::new()),
    }))
}

impl Storage {
    pub fn clone_ptr(&self) -> StoragePtr {
        Arc::new(self.clone())
    }

    pub fn snapshot(&self) -> Result<SnapShotPtr, FlameError> {
        let mut res = SnapShot {
            sessions: HashMap::new(),
            ssn_index: HashMap::new(),
            executors: HashMap::new(),
            exec_index: HashMap::new(),
        };

        {
            let ssn_map = lock_ptr!(self.sessions)?;
            for ssn in ssn_map.deref().values() {
                let ssn = lock_ptr!(ssn)?;
                let info = SessionInfo::from(&(*ssn));
                res.add_session(Rc::new(info));
            }
        }

        {
            let exe_map = lock_ptr!(self.executors)?;
            for exe in exe_map.deref().values() {
                let exe = lock_ptr!(exe)?;
                let info = ExecutorInfo::from(&(*exe).clone());
                res.add_executor(Rc::new(info));
            }
        }

        Ok(Rc::new(RefCell::new(res)))
    }

    pub async fn create_session(
        &self,
        app: String,
        slots: i32,
        common_data: Option<CommonData>,
    ) -> Result<Session, FlameError> {
        let ssn = self.engine.create_session(app, slots, common_data).await?;

        let mut ssn_map = lock_ptr!(self.sessions)?;
        ssn_map.insert(ssn.id, SessionPtr::new(ssn.clone().into()));

        Ok(ssn)
    }

    pub async fn close_session(&self, id: SessionID) -> Result<(), FlameError> {
        let ssn = self.engine.close_session(id).await?;

        let ssn_ptr = self.get_session_ptr(ssn.id)?;
        let mut ssn = lock_ptr!(ssn_ptr)?;
        ssn.status.state = SessionState::Closed;

        Ok(())
    }

    pub fn get_session(&self, id: SessionID) -> Result<Session, FlameError> {
        let ssn_ptr = self.get_session_ptr(id)?;
        let ssn = lock_ptr!(ssn_ptr)?;
        Ok(ssn.clone())
    }

    pub fn get_session_ptr(&self, id: SessionID) -> Result<SessionPtr, FlameError> {
        let ssn_map = lock_ptr!(self.sessions)?;
        let ssn = ssn_map
            .get(&id)
            .ok_or(FlameError::NotFound(id.to_string()))?;

        Ok(ssn.clone())
    }

    pub fn get_task_ptr(&self, ssn_id: SessionID, task_id: TaskID) -> Result<TaskPtr, FlameError> {
        let ssn_map = lock_ptr!(self.sessions)?;
        let ssn_ptr = ssn_map
            .get(&ssn_id)
            .ok_or(FlameError::NotFound(ssn_id.to_string()))?;

        let ssn = lock_ptr!(ssn_ptr)?;
        let task_ptr = ssn
            .tasks
            .get(&task_id)
            .ok_or(FlameError::NotFound(ssn_id.to_string()))?;

        Ok(task_ptr.clone())
    }

    pub async fn delete_session(&self, id: SessionID) -> Result<(), FlameError> {
        let ssn = self.engine.delete_session(id).await?;

        let mut ssn_map = lock_ptr!(self.sessions)?;
        ssn_map.remove(&ssn.id);

        Ok(())
    }

    pub fn list_session(&self) -> Result<Vec<Session>, FlameError> {
        let mut ssn_list = vec![];
        let ssn_map = lock_ptr!(self.sessions)?;

        for ssn in ssn_map.deref().values() {
            let ssn = lock_ptr!(ssn)?;
            ssn_list.push((*ssn).clone());
        }

        Ok(ssn_list)
    }

    pub async fn create_task(
        &self,
        ssn_id: SessionID,
        task_input: Option<TaskInput>,
    ) -> Result<Task, FlameError> {
        let task = self.engine.create_task(ssn_id, task_input).await?;

        let ssn = self.get_session_ptr(ssn_id)?;
        let mut ssn = lock_ptr!(ssn)?;
        ssn.add_task(&task);

        Ok(task)
    }

    pub fn get_task(&self, ssn_id: SessionID, id: TaskID) -> Result<Task, FlameError> {
        let ssn_map = lock_ptr!(self.sessions)?;

        let ssn = ssn_map
            .get(&ssn_id)
            .ok_or(FlameError::NotFound(ssn_id.to_string()))?;

        let ssn = lock_ptr!(ssn)?;
        let task = ssn
            .tasks
            .get(&id)
            .ok_or(FlameError::NotFound(id.to_string()))?;
        let task = lock_ptr!(task)?;
        Ok(task.clone())
    }

    pub async fn update_task_state(
        &self,
        ssn: SessionPtr,
        task: TaskPtr,
        state: TaskState,
    ) -> Result<(), FlameError> {
        let ssn_id = {
            let ssn_ptr = lock_ptr!(ssn)?;
            ssn_ptr.id
        };

        let task_id = {
            let task_ptr = lock_ptr!(task)?;
            task_ptr.id
        };

        let task = self
            .engine
            .update_task_state(ssn_id, task_id, state)
            .await?;

        let mut ssn_ptr = lock_ptr!(ssn)?;
        ssn_ptr.add_task(&task);

        Ok(())
    }

    pub async fn watch_task(&self, ssn_id: SessionID, task_id: TaskID) -> Result<Task, FlameError> {
        let task_ptr = self.get_task_ptr(ssn_id, task_id)?;
        WatchTaskFuture::new(self.clone_ptr(), &task_ptr)?.await?;

        let task = lock_ptr!(task_ptr)?;
        Ok((*task).clone())
    }

    pub fn register_executor(&self, e: &Executor) -> Result<(), FlameError> {
        let mut exe_map = lock_ptr!(self.executors)?;
        let exe = ExecutorPtr::new(e.clone().into());
        exe_map.insert(e.id.clone(), exe);

        Ok(())
    }

    pub fn get_executor_ptr(&self, id: ExecutorID) -> Result<ExecutorPtr, FlameError> {
        let exe_map = lock_ptr!(self.executors)?;
        let exe = exe_map
            .get(&id)
            .ok_or(FlameError::NotFound(id.to_string()))?;

        Ok(exe.clone())
    }

    pub async fn wait_for_session(&self, id: ExecutorID) -> Result<Session, FlameError> {
        let exe_ptr = self.get_executor_ptr(id)?;
        let ssn_id = WaitForSsnFuture::new(&exe_ptr).await?;

        let ssn_ptr = self.get_session_ptr(ssn_id)?;
        let ssn = lock_ptr!(ssn_ptr)?;

        Ok((*ssn).clone())
    }

    pub async fn bind_session(&self, id: ExecutorID, ssn_id: SessionID) -> Result<(), FlameError> {
        trace_fn!("Storage::bind_session");

        let exe_ptr = self.get_executor_ptr(id)?;
        let state = states::from(Arc::new(self.clone()), exe_ptr)?;

        let ssn_ptr = self.get_session_ptr(ssn_id)?;
        state.bind_session(ssn_ptr).await?;

        Ok(())
    }

    pub async fn bind_session_completed(&self, id: ExecutorID) -> Result<(), FlameError> {
        trace_fn!("Storage::bind_session_completed");

        let exe_ptr = self.get_executor_ptr(id)?;
        let state = states::from(Arc::new(self.clone()), exe_ptr)?;

        state.bind_session_completed().await?;

        Ok(())
    }

    pub async fn launch_task(&self, id: ExecutorID) -> Result<Option<Task>, FlameError> {
        trace_fn!("Storage::launch_task");
        let exe_ptr = self.get_executor_ptr(id)?;
        let state = states::from(Arc::new(self.clone()), exe_ptr.clone())?;
        let (ssn_id, task_id) = {
            let exec = lock_ptr!(exe_ptr)?;
            (exec.ssn_id, exec.task_id)
        };
        let ssn_id = ssn_id.ok_or(FlameError::InvalidState(
            "no session in bound executor".to_string(),
        ))?;

        //
        if let Some(task_id) = task_id {
            log::warn!(
                "Re-launch the task <{}/{}>",
                ssn_id.clone(),
                task_id.clone()
            );
            let task_ptr = self.get_task_ptr(ssn_id, task_id)?;

            let task = lock_ptr!(task_ptr)?;
            return Ok(Some((*task).clone()));
        }

        let ssn_ptr = self.get_session_ptr(ssn_id)?;
        state.launch_task(ssn_ptr).await
    }

    pub async fn complete_task(
        &self,
        id: ExecutorID,
        task_output: Option<TaskOutput>,
    ) -> Result<(), FlameError> {
        trace_fn!("Storage::complete_task");
        let exe_ptr = self.get_executor_ptr(id)?;
        let (ssn_id, task_id) = {
            let exe = lock_ptr!(exe_ptr)?;
            (
                exe.ssn_id.ok_or(FlameError::InvalidState(
                    "no session in executor".to_string(),
                ))?,
                exe.task_id
                    .ok_or(FlameError::InvalidState("no task in executor".to_string()))?,
            )
        };

        let task_ptr = self.get_task_ptr(ssn_id, task_id)?;
        let ssn_ptr = self.get_session_ptr(ssn_id)?;

        let state = states::from(Arc::new(self.clone()), exe_ptr)?;
        state.complete_task(ssn_ptr, task_ptr, task_output).await?;

        Ok(())
    }

    pub async fn unbind_executor(&self, id: ExecutorID) -> Result<(), FlameError> {
        let exe_ptr = self.get_executor_ptr(id)?;
        let state = states::from(Arc::new(self.clone()), exe_ptr)?;
        state.unbind_executor().await?;

        Ok(())
    }

    pub async fn unbind_executor_completed(&self, id: ExecutorID) -> Result<(), FlameError> {
        let exe_ptr = self.get_executor_ptr(id)?;
        let state = states::from(Arc::new(self.clone()), exe_ptr)?;

        state.unbind_executor_completed().await?;

        Ok(())
    }
}

struct WaitForSsnFuture {
    executor: ExecutorPtr,
}

impl WaitForSsnFuture {
    pub fn new(exe_ptr: &ExecutorPtr) -> Self {
        Self {
            executor: exe_ptr.clone(),
        }
    }
}

impl Future for WaitForSsnFuture {
    type Output = Result<SessionID, FlameError>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let exe = lock_ptr!(self.executor)?;

        match exe.ssn_id {
            None => {
                // No bound session, trigger waker.
                ctx.waker().wake_by_ref();
                Poll::Pending
            }
            Some(ssn_id) => Poll::Ready(Ok(ssn_id)),
        }
    }
}

struct WatchTaskFuture {
    storage: StoragePtr,
    current_state: TaskState,
    task_gid: TaskGID,
}

impl WatchTaskFuture {
    pub fn new(storage: StoragePtr, task_ptr: &TaskPtr) -> Result<Self, FlameError> {
        let task_ptr = task_ptr.clone();
        let task = lock_ptr!(task_ptr)?;

        Ok(Self {
            storage,
            current_state: task.state,
            task_gid: TaskGID {
                ssn_id: task.ssn_id,
                task_id: task.id,
            },
        })
    }
}

impl Future for WatchTaskFuture {
    type Output = Result<(), FlameError>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let task_ptr = self
            .storage
            .get_task_ptr(self.task_gid.ssn_id, self.task_gid.task_id)?;

        let task = lock_ptr!(task_ptr)?;
        // If the state of task was updated, return ready.
        if self.current_state != task.state || task.is_completed() {
            return Poll::Ready(Ok(()));
        }

        ctx.waker().wake_by_ref();
        Poll::Pending
    }
}
