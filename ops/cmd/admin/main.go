package main

import (
	"context"
	"fmt"
	"log"
	"net"

	"google.golang.org/grpc"
	"google.golang.org/grpc/reflection"
	"google.golang.org/protobuf/types/known/timestamppb"

	pb "github.com/edgecouchbase/edgecouchbase/proto/admin/v1"
)

type adminServer struct {
	pb.UnimplementedAdminServiceServer
}

func (s *adminServer) GetClusterInfo(ctx context.Context, req *pb.GetClusterInfoRequest) (*pb.GetClusterInfoResponse, error) {
	log.Println("Received GetClusterInfo request")

	// TODO: Implement cluster info retrieval
	clusterInfo := &pb.ClusterInfo{
		ClusterId:   "cluster-1",
		ClusterName: "EdgeCouchbase Dev Cluster",
		Nodes: []*pb.NodeInfo{
			{
				NodeId:   "node-1",
				Hostname: "localhost",
				Status:   pb.NodeInfo_NODE_STATUS_ACTIVE,
				JoinedAt: timestamppb.Now(),
				Services: []*pb.ServiceInfo{
					{
						Type:    pb.ServiceInfo_SERVICE_DATA,
						Port:    11210,
						Enabled: true,
					},
					{
						Type:    pb.ServiceInfo_SERVICE_QUERY,
						Port:    8093,
						Enabled: true,
					},
				},
			},
		},
	}

	return &pb.GetClusterInfoResponse{
		Cluster: clusterInfo,
	}, nil
}

func (s *adminServer) InitCluster(ctx context.Context, req *pb.InitClusterRequest) (*pb.InitClusterResponse, error) {
	log.Printf("Received InitCluster request: cluster_name=%s, memory_quota=%d MB\n",
		req.ClusterName, req.MemoryQuotaMb)

	// TODO: Implement cluster initialization
	return &pb.InitClusterResponse{
		ClusterId: "cluster-" + req.ClusterName,
		Success:   true,
	}, nil
}

func (s *adminServer) CreateBucket(ctx context.Context, req *pb.CreateBucketRequest) (*pb.CreateBucketResponse, error) {
	log.Printf("Received CreateBucket request: name=%s, memory_quota=%d MB, replicas=%d\n",
		req.Name, req.MemoryQuotaMb, req.ReplicaCount)

	// TODO: Implement bucket creation
	return &pb.CreateBucketResponse{
		Success: true,
	}, nil
}

func (s *adminServer) ListBuckets(ctx context.Context, req *pb.ListBucketsRequest) (*pb.ListBucketsResponse, error) {
	log.Println("Received ListBuckets request")

	// TODO: Implement bucket listing
	return &pb.ListBucketsResponse{
		Buckets: []*pb.BucketInfo{},
	}, nil
}

func (s *adminServer) GetNodeHealth(ctx context.Context, req *pb.GetNodeHealthRequest) (*pb.GetNodeHealthResponse, error) {
	log.Println("Received GetNodeHealth request")

	// TODO: Implement actual health checks
	return &pb.GetNodeHealthResponse{
		Status: pb.GetNodeHealthResponse_HEALTH_HEALTHY,
		Services: []*pb.ServiceHealth{
			{
				Type:    pb.ServiceInfo_SERVICE_DATA,
				Status:  pb.GetNodeHealthResponse_HEALTH_HEALTHY,
				Message: "OK",
			},
			{
				Type:    pb.ServiceInfo_SERVICE_QUERY,
				Status:  pb.GetNodeHealthResponse_HEALTH_HEALTHY,
				Message: "OK",
			},
		},
	}, nil
}

func (s *adminServer) Rebalance(req *pb.RebalanceRequest, stream pb.AdminService_RebalanceServer) error {
	log.Println("Received Rebalance request")

	// TODO: Implement rebalancing logic
	// Send progress updates
	if err := stream.Send(&pb.RebalanceResponse{
		Event: &pb.RebalanceResponse_Started{
			Started: &pb.RebalanceStarted{
				TotalVbuckets: 1024,
				StartedAt:     timestamppb.Now(),
			},
		},
	}); err != nil {
		return err
	}

	// Simulate progress
	if err := stream.Send(&pb.RebalanceResponse{
		Event: &pb.RebalanceResponse_Progress{
			Progress: &pb.RebalanceProgress{
				VbucketsMoved:   1024,
				TotalVbuckets:   1024,
				PercentComplete: 100,
			},
		},
	}); err != nil {
		return err
	}

	// Complete
	if err := stream.Send(&pb.RebalanceResponse{
		Event: &pb.RebalanceResponse_Completed{
			Completed: &pb.RebalanceCompleted{
				CompletedAt:   timestamppb.Now(),
				VbucketsMoved: 1024,
			},
		},
	}); err != nil {
		return err
	}

	return nil
}

// Stub implementations for other methods
func (s *adminServer) AddNode(ctx context.Context, req *pb.AddNodeRequest) (*pb.AddNodeResponse, error) {
	return &pb.AddNodeResponse{Success: false}, fmt.Errorf("not implemented")
}

func (s *adminServer) RemoveNode(ctx context.Context, req *pb.RemoveNodeRequest) (*pb.RemoveNodeResponse, error) {
	return &pb.RemoveNodeResponse{Success: false}, fmt.Errorf("not implemented")
}

func (s *adminServer) Failover(ctx context.Context, req *pb.FailoverRequest) (*pb.FailoverResponse, error) {
	return &pb.FailoverResponse{Success: false}, fmt.Errorf("not implemented")
}

func (s *adminServer) DeleteBucket(ctx context.Context, req *pb.DeleteBucketRequest) (*pb.DeleteBucketResponse, error) {
	return &pb.DeleteBucketResponse{Success: false}, fmt.Errorf("not implemented")
}

func (s *adminServer) GetBucketInfo(ctx context.Context, req *pb.GetBucketInfoRequest) (*pb.GetBucketInfoResponse, error) {
	return nil, fmt.Errorf("not implemented")
}

func (s *adminServer) CreateScope(ctx context.Context, req *pb.CreateScopeRequest) (*pb.CreateScopeResponse, error) {
	return &pb.CreateScopeResponse{Success: false}, fmt.Errorf("not implemented")
}

func (s *adminServer) DeleteScope(ctx context.Context, req *pb.DeleteScopeRequest) (*pb.DeleteScopeResponse, error) {
	return &pb.DeleteScopeResponse{Success: false}, fmt.Errorf("not implemented")
}

func (s *adminServer) CreateCollection(ctx context.Context, req *pb.CreateCollectionRequest) (*pb.CreateCollectionResponse, error) {
	return &pb.CreateCollectionResponse{Success: false}, fmt.Errorf("not implemented")
}

func (s *adminServer) DeleteCollection(ctx context.Context, req *pb.DeleteCollectionRequest) (*pb.DeleteCollectionResponse, error) {
	return &pb.DeleteCollectionResponse{Success: false}, fmt.Errorf("not implemented")
}

func (s *adminServer) GetClusterMetrics(ctx context.Context, req *pb.GetClusterMetricsRequest) (*pb.GetClusterMetricsResponse, error) {
	return &pb.GetClusterMetricsResponse{
		Metrics: &pb.ClusterMetrics{
			TotalOpsPerSec:  1000,
			TotalGetsPerSec: 600,
			TotalSetsPerSec: 400,
		},
	}, nil
}

func main() {
	lis, err := net.Listen("tcp", ":8091")
	if err != nil {
		log.Fatalf("Failed to listen: %v", err)
	}

	grpcServer := grpc.NewServer()
	pb.RegisterAdminServiceServer(grpcServer, &adminServer{})
	
	// Register reflection service for grpcurl/grpcui
	reflection.Register(grpcServer)

	log.Println("EdgeCouchbase Admin Service starting on :8091")
	if err := grpcServer.Serve(lis); err != nil {
		log.Fatalf("Failed to serve: %v", err)
	}
}
