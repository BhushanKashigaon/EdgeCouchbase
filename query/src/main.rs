use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod query {
    tonic::include_proto!("edgecb.query.v1");
}

use query::query_service_server::{QueryService, QueryServiceServer};
use query::*;

#[derive(Debug, Default)]
pub struct QueryServiceImpl {}

#[tonic::async_trait]
impl QueryService for QueryServiceImpl {
    type ExecuteStream = tokio_stream::wrappers::ReceiverStream<Result<ExecuteResponse, Status>>;
    
    async fn execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<Self::ExecuteStream>, Status> {
        let req = request.into_inner();
        info!(statement = %req.statement, "Received EXECUTE request");

        let (tx, rx) = tokio::sync::mpsc::channel(128);

        // Spawn task to send responses
        tokio::spawn(async move {
            // TODO: Parse and execute SQL++ query
            // For now, return empty result set

            let status = QueryStatus {
                status: "success".to_string(),
                request_id: uuid::Uuid::new_v4().to_string(),
                client_context_id: req
                    .options
                    .and_then(|o| Some(o.client_context_id))
                    .unwrap_or_default(),
            };

            let _ = tx
                .send(Ok(ExecuteResponse {
                    message: Some(execute_response::Message::Status(status)),
                }))
                .await;

            let metrics = QueryMetrics {
                elapsed_time_ms: 0,
                execution_time_ms: 0,
                result_count: 0,
                result_size_bytes: 0,
                mutation_count: 0,
                sort_count: 0,
                error_count: 0,
                warning_count: 0,
            };

            let _ = tx
                .send(Ok(ExecuteResponse {
                    message: Some(execute_response::Message::Metrics(metrics)),
                }))
                .await;
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    async fn prepare(
        &self,
        request: Request<PrepareRequest>,
    ) -> Result<Response<PrepareResponse>, Status> {
        let req = request.into_inner();
        info!(statement = %req.statement, "Received PREPARE request");

        // TODO: Parse and plan query
        Ok(Response::new(PrepareResponse {
            prepared_name: uuid::Uuid::new_v4().to_string(),
            encoded_plan: vec![],
            plan_text: None,
        }))
    }

    type ExecutePreparedStream = tokio_stream::wrappers::ReceiverStream<Result<ExecuteResponse, Status>>;

    async fn execute_prepared(
        &self,
        request: Request<ExecutePreparedRequest>,
    ) -> Result<Response<Self::ExecutePreparedStream>, Status> {
        let req = request.into_inner();
        info!(prepared_name = %req.prepared_name, "Received EXECUTE_PREPARED request");

        let (tx, rx) = tokio::sync::mpsc::channel(128);

        // TODO: Execute prepared plan
        tokio::spawn(async move {
            let status = QueryStatus {
                status: "success".to_string(),
                request_id: uuid::Uuid::new_v4().to_string(),
                client_context_id: String::new(),
            };

            let _ = tx
                .send(Ok(ExecuteResponse {
                    message: Some(execute_response::Message::Status(status)),
                }))
                .await;
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    async fn explain(
        &self,
        request: Request<ExplainRequest>,
    ) -> Result<Response<ExplainResponse>, Status> {
        let req = request.into_inner();
        info!(statement = %req.statement, "Received EXPLAIN request");

        // TODO: Generate query plan
        Ok(Response::new(ExplainResponse {
            plan: None,
            plan_text: "Query plan not yet implemented".to_string(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    let addr = "0.0.0.0:8093".parse()?;
    let query_service = QueryServiceImpl::default();

    info!("EdgeCouchbase Query Service starting on {}", addr);

    Server::builder()
        .add_service(QueryServiceServer::new(query_service))
        .serve(addr)
        .await?;

    Ok(())
}
