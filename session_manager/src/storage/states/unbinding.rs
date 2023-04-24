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

use crate::storage::states::States;
use common::apis::{ExecutorPtr, ExecutorState, SessionPtr, Task, TaskOutput, TaskPtr, TaskState};
use common::{lock_cond_ptr, trace::TraceFn, trace_fn, FlameError};

pub struct UnbindingState {
    pub executor: ExecutorPtr,
}

impl States for UnbindingState {
    fn bind_session(&self, _ssn_ptr: SessionPtr) -> Result<(), FlameError> {
        todo!()
    }

    fn bind_session_completed(&self) -> Result<(), FlameError> {
        todo!()
    }

    fn unbind_executor(&self) -> Result<(), FlameError> {
        trace_fn!("UnbindingState::unbind_session");

        let mut e = lock_cond_ptr!(self.executor)?;
        e.state = ExecutorState::Unbinding;

        Ok(())
    }

    fn unbind_executor_completed(&self) -> Result<(), FlameError> {
        trace_fn!("UnbindingState::unbind_session_completed");

        let mut e = lock_cond_ptr!(self.executor)?;
        e.state = ExecutorState::Idle;
        e.ssn_id = None;
        e.task_id = None;

        Ok(())
    }

    fn launch_task(&self, _ssn: SessionPtr) -> Result<Option<Task>, FlameError> {
        Ok(None)
    }

    fn complete_task(
        &self,
        ssn_ptr: SessionPtr,
        task_ptr: TaskPtr,
        task_output: Option<TaskOutput>,
    ) -> Result<(), FlameError> {
        trace_fn!("UnbindingState::complete_task");

        {
            let mut e = lock_cond_ptr!(self.executor)?;
            e.task_id = None;
        };

        {
            let mut task = lock_cond_ptr!(task_ptr)?;
            task.output = task_output;
        }

        {
            let mut ssn = lock_cond_ptr!(ssn_ptr)?;
            ssn.update_task_state(task_ptr, TaskState::Succeed)?;
        }

        Ok(())
    }
}
