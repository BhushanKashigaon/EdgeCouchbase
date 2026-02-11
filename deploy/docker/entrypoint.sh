#!/bin/bash
set -e

echo "Starting EdgeCouchbase Node"
echo "Node ID: ${NODE_ID:-unknown}"
echo "Services: ${SERVICES:-data}"
echo "Cluster: ${CLUSTER_NAME:-default}"

# Parse services to start
IFS=',' read -ra SERVICE_ARRAY <<< "${SERVICES:-data}"

# Start admin service in background
if [[ " ${SERVICE_ARRAY[@]} " =~ " admin " ]]; then
    echo "Starting Admin service on :8091"
    edgecb-admin &
    ADMIN_PID=$!
fi

# Start data service
if [[ " ${SERVICE_ARRAY[@]} " =~ " data " ]]; then
    echo "Starting Data service on :11210"
    edgecb-server &
    DATA_PID=$!
fi

# Start query service
if [[ " ${SERVICE_ARRAY[@]} " =~ " query " ]]; then
    echo "Starting Query service on :8093"
    edgecb-query &
    QUERY_PID=$!
fi

# TODO: Start index, search, analytics services

# Wait for all background processes
wait
