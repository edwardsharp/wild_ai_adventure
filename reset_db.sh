#!/bin/bash

# Simple PostgreSQL Database Reset Script
# Resets Docker volumes, SQLx state, and runs migrations

set -e  # Exit on any error

echo "🗑️  Stopping containers and removing volumes..."
docker-compose down --volumes

echo "🔄 Resetting SQLx state..."
rm -rf .sqlx

echo "🚀 Starting fresh database..."
docker-compose up -d postgres

echo "⏳ Waiting for PostgreSQL to be ready..."
sleep 5

echo "📋 Running migrations..."
./run_migrations.sh

echo "✅ Database reset complete!"
