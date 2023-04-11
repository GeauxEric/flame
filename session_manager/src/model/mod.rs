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

pub use crate::model::errors::FlameError;

mod errors;

pub type SessionID = i64;
pub type TaskID = i64;
pub type ExecutorID = String;


#[derive(Clone, Copy, Debug)]
pub enum SessionState {
    Open = 0,
    Closed = 1,
}

#[derive(Clone, Debug)]
pub struct Session {
    pub id: SessionID,
    pub service_type: String,
    pub slots: i32,
    pub tasks: Vec<Box<Task>>,
    pub state: SessionState,
}

#[derive(Clone, Copy, Debug)]
pub enum TaskState {
    Pending = 0,
    Running = 1,
    Completed = 2,
    Failed = 3,
    Aborting = 4,
    Aborted = 5,
}

#[derive(Clone, Debug)]
pub struct Task {
    pub id: TaskID,
    pub ssn_id: SessionID,
    pub input: String,
    pub output: String,
    pub state: TaskState,
}

#[derive(Clone, Copy, Debug)]
pub enum ExecutorState {
    Idle = 0,
    Binding = 1,
    Bound = 2,
    Unbinding = 3,
    Unknown = 4,
}

#[derive(Clone, Debug)]
pub struct Executor {
    pub id: ExecutorID,
    pub service_type: String,
    pub task_id: Option<TaskID>,
    pub ssn_id: Option<SessionID>,

    pub state: ExecutorState,
}
