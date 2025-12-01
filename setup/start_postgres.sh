#!/bin/bash

# Start PostgreSQL and Redis using Docker Compose

echo "Starting PostgreSQL and Redis..."
docker-compose up -d postgres redis

if [ $? -ne 0 ]; then
    echo "Error: Failed to start Docker containers"
    echo "Make sure Docker is installed and running"
    exit 1
fi

echo ""
echo "Waiting for database to be ready..."
sleep 5

echo ""
echo "Running migrations..."
psql -U intent_user -d intent_segregation -h localhost -f core/ledger/migrations/20250101000001_init.sql

if [ $? -ne 0 ]; then
    echo "Warning: Migration may have failed. Check that PostgreSQL is accessible."
fi

echo ""
echo "PostgreSQL is ready!"
echo "Credentials: intent_user / intent_pass @ localhost:5432/intent_segregation"
echo ""
echo "Next steps:"
echo "  1. Open another terminal and run: cargo run --bin intent-api"
echo "  2. After API starts, run: bash run_tests.sh"
