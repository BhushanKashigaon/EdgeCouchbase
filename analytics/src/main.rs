use tonic::{transport::Server, Request, Response, Status};
use tracing::{info, error};

pub mod analytics {
    tonic::include_proto!("edgecb.analytics.v1");
}

use analytics::analytics_service_server::{AnalyticsService, AnalyticsServiceServer};
use analytics::*;

#[derive(Debug, Default)]
pub struct AnalyticsServiceImpl {}

#[tonic::async_trait]
impl AnalyticsService for AnalyticsServiceImpl {
    async fn create_dataverse(
        &self,
        request: Request<CreateDataverseRequest>,
    ) -> Result<Response<CreateDataverseResponse>, Status> {
        let req = request.into_inner();
        info!(dataverse = %req.dataverse, "Creating dataverse");

        // TODO: Implement dataverse creation
        Ok(Response::new(CreateDataverseResponse {
            success: true,
            message: format!("Dataverse '{}' created", req.dataverse),
        }))
    }

    async fn drop_dataverse(
        &self,
        request: Request<DropDataverseRequest>,
    ) -> Result<Response<DropDataverseResponse>, Status> {
        let req = request.into_inner();
        info!(dataverse = %req.dataverse, "Dropping dataverse");

        // TODO: Implement dataverse drop
        Ok(Response::new(DropDataverseResponse {
            success: true,
            message: format!("Dataverse '{}' dropped", req.dataverse),
        }))
    }

    async fn create_dataset(
        &self,
        request: Request<CreateDatasetRequest>,
    ) -> Result<Response<CreateDatasetResponse>, Status> {
        let req = request.into_inner();
        info!(
            dataverse = %req.dataverse,
            dataset = %req.dataset_name,
            bucket = %req.bucket,
            "Creating dataset"
        );

        // TODO: Implement dataset creation with DCP ingestion
        Ok(Response::new(CreateDatasetResponse {
            success: true,
            message: format!("Dataset '{}.{}' created", req.dataverse, req.dataset_name),
        }))
    }

    async fn drop_dataset(
        &self,
        request: Request<DropDatasetRequest>,
    ) -> Result<Response<DropDatasetResponse>, Status> {
        let req = request.into_inner();
        info!(
            dataverse = %req.dataverse,
            dataset = %req.dataset_name,
            "Dropping dataset"
        );

        // TODO: Implement dataset drop
        Ok(Response::new(DropDatasetResponse {
            success: true,
            message: format!("Dataset '{}.{}' dropped", req.dataverse, req.dataset_name),
        }))
    }

    async fn connect_link(
        &self,
        request: Request<ConnectLinkRequest>,
    ) -> Result<Response<ConnectLinkResponse>, Status> {
        let req = request.into_inner();
        info!(
            dataverse = %req.dataverse,
            link = %req.link_name,
            "Connecting link"
        );

        // TODO: Start DCP ingestion
        Ok(Response::new(ConnectLinkResponse {
            success: true,
            message: format!("Link '{}.{}' connected", req.dataverse, req.link_name),
        }))
    }

    async fn disconnect_link(
        &self,
        request: Request<DisconnectLinkRequest>,
    ) -> Result<Response<DisconnectLinkResponse>, Status> {
        let req = request.into_inner();
        info!(
            dataverse = %req.dataverse,
            link = %req.link_name,
            "Disconnecting link"
        );

        // TODO: Stop DCP ingestion
        Ok(Response::new(DisconnectLinkResponse {
            success: true,
            message: format!("Link '{}.{}' disconnected", req.dataverse, req.link_name),
        }))
    }

    type QueryStream = tokio_stream::wrappers::ReceiverStream<Result<AnalyticsQueryResponse, Status>>;

    async fn query(
        &self,
        request: Request<AnalyticsQueryRequest>,
    ) -> Result<Response<Self::QueryStream>, Status> {
        let req = request.into_inner();
        info!(statement = %req.statement, "Executing analytics query");

        let (tx, rx) = tokio::sync::mpsc::channel(128);
        
        tokio::spawn(async move {
            // TODO: Implement MPP query execution
            use analytics::analytics_query_response::Message;
            let _ = tx.send(Ok(AnalyticsQueryResponse {
                message: Some(Message::Status(QueryStatus {
                    status: "success".to_string(),
                })),
            })).await;
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    async fn get_dataset_stats(
        &self,
        request: Request<GetDatasetStatsRequest>,
    ) -> Result<Response<GetDatasetStatsResponse>, Status> {
        let req = request.into_inner();
        info!(
            dataverse = %req.dataverse,
            dataset = %req.dataset_name,
            "Getting dataset stats"
        );

        // TODO: Implement stats retrieval
        Err(Status::unimplemented("Dataset stats not yet implemented"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .json()
        .init();

    let addr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8095".to_string())
        .parse()?;

    let service = AnalyticsServiceImpl::default();

    info!("Analytics service starting on {}", addr);

    Server::builder()
        .add_service(AnalyticsServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
