use tonic::{transport::Server, Request, Response, Status};
use tracing::{info, error};

pub mod search {
    tonic::include_proto!("edgecb.search.v1");
}

use search::search_service_server::{SearchService, SearchServiceServer};
use search::*;

#[derive(Debug, Default)]
pub struct SearchServiceImpl {}

#[tonic::async_trait]
impl SearchService for SearchServiceImpl {
    async fn create_index(
        &self,
        request: Request<CreateSearchIndexRequest>,
    ) -> Result<Response<CreateSearchIndexResponse>, Status> {
        let req = request.into_inner();
        info!(
            index_name = %req.index_name,
            bucket = %req.bucket,
            "Creating search index"
        );

        // TODO: Implement FTS index creation
        Ok(Response::new(CreateSearchIndexResponse {
            success: true,
            message: format!("Search index '{}' created", req.index_name),
            index_uuid: uuid::Uuid::new_v4().to_string(),
        }))
    }

    async fn delete_index(
        &self,
        request: Request<DeleteSearchIndexRequest>,
    ) -> Result<Response<DeleteSearchIndexResponse>, Status> {
        let req = request.into_inner();
        info!(index_name = %req.index_name, "Deleting search index");

        // TODO: Implement FTS index deletion
        Ok(Response::new(DeleteSearchIndexResponse {
            success: true,
            message: format!("Search index '{}' deleted", req.index_name),
        }))
    }

    async fn list_indexes(
        &self,
        request: Request<ListSearchIndexesRequest>,
    ) -> Result<Response<ListSearchIndexesResponse>, Status> {
        let req = request.into_inner();
        info!(bucket = ?req.bucket, "Listing search indexes");

        // TODO: Implement list search indexes
        Ok(Response::new(ListSearchIndexesResponse {
            indexes: vec![],
        }))
    }

    async fn get_index_stats(
        &self,
        request: Request<GetSearchIndexStatsRequest>,
    ) -> Result<Response<GetSearchIndexStatsResponse>, Status> {
        let req = request.into_inner();
        info!(index_name = %req.index_name, "Getting search index stats");

        // TODO: Implement stats retrieval
        Err(Status::unimplemented("Search index stats not yet implemented"))
    }

    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let req = request.into_inner();
        info!(index_name = %req.index_name, "Executing search");

        // TODO: Implement full-text search
        Ok(Response::new(SearchResponse {
            status: Some(SearchStatus {
                total_shards: 1,
                successful_shards: 1,
                failed_shards: 0,
                failures: vec![],
            }),
            hits: vec![],
            total_hits: 0,
            max_score: 0.0,
            took_ms: 0,
            facets: None,
        }))
    }

    async fn vector_search(
        &self,
        request: Request<VectorSearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let req = request.into_inner();
        info!(index_name = %req.index_name, k = req.k, "Executing vector search");

        // TODO: Implement vector search (ANN)
        Ok(Response::new(SearchResponse {
            status: Some(SearchStatus {
                total_shards: 1,
                successful_shards: 1,
                failed_shards: 0,
                failures: vec![],
            }),
            hits: vec![],
            total_hits: 0,
            max_score: 0.0,
            took_ms: 0,
            facets: None,
        }))
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
        .unwrap_or_else(|_| "0.0.0.0:8094".to_string())
        .parse()?;

    let service = SearchServiceImpl::default();

    info!("Search service starting on {}", addr);

    Server::builder()
        .add_service(SearchServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
