use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod kv {
    tonic::include_proto!("edgecb.kv.v1");
}

use kv::data_service_server::{DataService, DataServiceServer};
use kv::*;

#[derive(Debug, Default)]
pub struct DataServiceImpl {}

#[tonic::async_trait]
impl DataService for DataServiceImpl {
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let req = request.into_inner();
        info!(
            bucket = %req.bucket,
            scope = %req.scope,
            collection = %req.collection,
            key = %req.key,
            "Received GET request"
        );

        // TODO: Implement actual KV lookup
        // For now, return key not found
        let error = Error {
            code: error::ErrorCode::ErrorKeyNotFound as i32,
            message: format!("Key '{}' not found", req.key),
        };

        Ok(Response::new(GetResponse {
            result: Some(get_response::Result::Error(error)),
        }))
    }

    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
        let req = request.into_inner();
        info!(
            bucket = %req.bucket,
            scope = %req.scope,
            collection = %req.collection,
            key = %req.key,
            "Received SET request"
        );

        // TODO: Implement actual KV storage
        // For now, return success with dummy CAS
        let result = SetResult {
            cas: 1,
            seq_no: 1,
            vbucket_id: 0,
        };

        Ok(Response::new(SetResponse {
            result: Some(set_response::Result::Success(result)),
        }))
    }

    async fn delete(
        &self,
        request: Request<DeleteRequest>,
    ) -> Result<Response<DeleteResponse>, Status> {
        let req = request.into_inner();
        info!(
            bucket = %req.bucket,
            scope = %req.scope,
            collection = %req.collection,
            key = %req.key,
            "Received DELETE request"
        );

        // TODO: Implement actual deletion
        let error = Error {
            code: error::ErrorCode::ErrorKeyNotFound as i32,
            message: format!("Key '{}' not found", req.key),
        };

        Ok(Response::new(DeleteResponse {
            result: Some(delete_response::Result::Error(error)),
        }))
    }

    async fn get_multi(
        &self,
        request: Request<GetMultiRequest>,
    ) -> Result<Response<GetMultiResponse>, Status> {
        let req = request.into_inner();
        info!(
            bucket = %req.bucket,
            keys_count = req.keys.len(),
            "Received GET_MULTI request"
        );

        // TODO: Implement batch get
        Ok(Response::new(GetMultiResponse {
            documents: std::collections::HashMap::new(),
            missing_keys: req.keys,
        }))
    }

    async fn touch(
        &self,
        request: Request<TouchRequest>,
    ) -> Result<Response<TouchResponse>, Status> {
        let req = request.into_inner();
        info!(key = %req.key, "Received TOUCH request");

        // TODO: Implement touch
        let error = Error {
            code: error::ErrorCode::ErrorKeyNotFound as i32,
            message: "Not implemented".to_string(),
        };

        Ok(Response::new(TouchResponse {
            result: Some(touch_response::Result::Error(error)),
        }))
    }

    async fn increment(
        &self,
        request: Request<IncrementRequest>,
    ) -> Result<Response<IncrementResponse>, Status> {
        let req = request.into_inner();
        info!(key = %req.key, delta = req.delta, "Received INCREMENT request");

        // TODO: Implement atomic increment
        let result = CounterResult {
            value: req.initial,
            cas: 1,
        };

        Ok(Response::new(IncrementResponse {
            result: Some(increment_response::Result::Success(result)),
        }))
    }

    async fn decrement(
        &self,
        request: Request<DecrementRequest>,
    ) -> Result<Response<DecrementResponse>, Status> {
        let req = request.into_inner();
        info!(key = %req.key, delta = req.delta, "Received DECREMENT request");

        // TODO: Implement atomic decrement
        let result = CounterResult {
            value: req.initial,
            cas: 1,
        };

        Ok(Response::new(DecrementResponse {
            result: Some(decrement_response::Result::Success(result)),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    let addr = "0.0.0.0:11210".parse()?;
    let data_service = DataServiceImpl::default();

    info!("EdgeCouchbase Data Service starting on {}", addr);

    Server::builder()
        .add_service(DataServiceServer::new(data_service))
        .serve(addr)
        .await?;

    Ok(())
}
