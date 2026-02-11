package main

import (
	"fmt"
	"log"
	"os"

	"github.com/spf13/cobra"
)

var (
	serverAddr string
)

var rootCmd = &cobra.Command{
	Use:   "edgecb-cli",
	Short: "EdgeCouchbase cluster management CLI",
	Long:  `CLI tool for managing EdgeCouchbase clusters, buckets, and data.`,
}

var clusterCmd = &cobra.Command{
	Use:   "cluster",
	Short: "Cluster management commands",
}

var clusterInfoCmd = &cobra.Command{
	Use:   "info",
	Short: "Get cluster information",
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("Cluster info not yet implemented")
		// TODO: Call AdminService.GetClusterInfo
	},
}

var bucketCmd = &cobra.Command{
	Use:   "bucket",
	Short: "Bucket management commands",
}

var bucketCreateCmd = &cobra.Command{
	Use:   "create [name]",
	Short: "Create a new bucket",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		bucketName := args[0]
		fmt.Printf("Creating bucket: %s\n", bucketName)
		// TODO: Call AdminService.CreateBucket
	},
}

var bucketListCmd = &cobra.Command{
	Use:   "list",
	Short: "List all buckets",
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("Listing buckets...")
		// TODO: Call AdminService.ListBuckets
	},
}

func init() {
	rootCmd.PersistentFlags().StringVarP(&serverAddr, "server", "s", "localhost:8091", "EdgeCouchbase server address")

	clusterCmd.AddCommand(clusterInfoCmd)
	bucketCmd.AddCommand(bucketCreateCmd, bucketListCmd)
	
	rootCmd.AddCommand(clusterCmd, bucketCmd)
}

func main() {
	if err := rootCmd.Execute(); err != nil {
		log.Fatal(err)
		os.Exit(1)
	}
}
