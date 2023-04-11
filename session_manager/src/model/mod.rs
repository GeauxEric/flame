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

use chrono::{DateTime, Utc};

use std::sync::Arc;

mod errors;
pub use crate::model::errors::FlameError;

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
    pub application: String,
    pub slots: i32,
    pub tasks: Vec<Arc<Task>>,

    pub creation_time: DateTime<Utc>,
    pub completion_time: Option<DateTime<Utc>>,

    pub state: SessionState,

    pub desired: f64,
    pub allocated: f64,
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

    pub creation_time: DateTime<Utc>,
    pub completion_time: Option<DateTime<Utc>>,

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
pub struct Application {
    pub name: String,
    pub command: String,
    pub arguments: Vec<String>,
    pub environments: Vec<String>,
    pub working_directory: String,
}

#[derive(Clone, Debug)]
pub struct Executor {
    pub id: ExecutorID,
    pub application: Application,
    pub task_id: Option<TaskID>,
    pub ssn_id: Option<SessionID>,

    pub creation_time: DateTime<Utc>,
    pub state: ExecutorState,
}
