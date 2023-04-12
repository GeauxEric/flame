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

use async_trait::async_trait;
use tonic::{Request, Response, Status};

use rpc::flame::frontend_server::Frontend;
use rpc::flame::{
    CreateSessionRequest, CreateTaskRequest, DeleteSessionRequest, DeleteTaskRequest,
    GetSessionRequest, GetTaskRequest, ListSessionRequest, Session, SessionList, Task,
};

use crate::apiserver::Flame;
use crate::model::SessionID;

#[async_trait]
impl Frontend for Flame {
    async fn create_session(
        &self,
        req: Request<CreateSessionRequest>,
    ) -> Result<Response<Session>, Status> {
        let ssn_spec = req
            .into_inner()
            .session
            .ok_or(Status::invalid_argument("session spec"))?;

        let ssn = self
            .storage
            .create_session(ssn_spec.application, ssn_spec.slots)
            .map_err(Status::from)?;

        Ok(Response::new(Session::from(&ssn)))
    }

    async fn delete_session(
        &self,
        _: Request<DeleteSessionRequest>,
    ) -> Result<Response<rpc::flame::Result>, Status> {
        todo!()
    }
    async fn get_session(
        &self,
        req: Request<GetSessionRequest>,
    ) -> Result<Response<Session>, Status> {
        let ssn_id = req
            .into_inner()
            .session_id
            .parse::<SessionID>()
            .map_err(|_| Status::invalid_argument("invalid session id"))?;

        let ssn = self.storage.get_session(ssn_id).map_err(Status::from)?;

        Ok(Response::new(Session::from(&ssn)))
    }
    async fn list_session(
        &self,
        _: Request<ListSessionRequest>,
    ) -> Result<Response<SessionList>, Status> {
        todo!()
    }

    async fn create_task(&self, _: Request<CreateTaskRequest>) -> Result<Response<Task>, Status> {
        todo!()
    }
    async fn delete_task(
        &self,
        _: Request<DeleteTaskRequest>,
    ) -> Result<Response<rpc::flame::Result>, Status> {
        todo!()
    }
    async fn get_task(&self, _: Request<GetTaskRequest>) -> Result<Response<Task>, Status> {
        todo!()
    }
}
