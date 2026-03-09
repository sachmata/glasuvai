#!/usr/bin/env bash
# ── Glasuvai Docker Dev Environment Bootstrap ──────────────────────
# Quick start script for Docker-based development.
#
# Usage:
#   ./scripts/docker-dev.sh         # Build & start dev shell
#   ./scripts/docker-dev.sh build   # Rebuild image
#   ./scripts/docker-dev.sh shell   # Attach to running dev container
#   ./scripts/docker-dev.sh down    # Stop everything
#   ./scripts/docker-dev.sh clean   # Stop and remove volumes

set -euo pipefail

COMPOSE_PROJECT="glasuvai"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

# Create .env from example if missing
if [[ ! -f .env ]]; then
    echo "Creating .env from .env.example..."
    cp .env.example .env
fi

case "${1:-shell}" in
    build)
        echo "Building glasuvai dev image..."
        docker compose build dev
        ;;
    shell)
        echo "Starting glasuvai dev shell..."
        docker compose up -d dev
        docker compose exec dev bash
        ;;
    up)
        echo "Starting all services..."
        docker compose up -d
        ;;
    down)
        echo "Stopping all services..."
        docker compose down
        ;;
    clean)
        echo "Stopping all services and removing volumes..."
        docker compose down -v
        ;;
    logs)
        docker compose logs -f "${@:2}"
        ;;
    test)
        echo "Running tests in dev container..."
        docker compose exec dev cargo test --workspace
        ;;
    fmt)
        echo "Running formatter in dev container..."
        docker compose exec dev cargo fmt --all
        ;;
    clippy)
        echo "Running clippy in dev container..."
        docker compose exec dev cargo clippy --workspace --all-targets -- -D warnings
        ;;
    wasm)
        echo "Building crypto-wasm package..."
        docker compose exec dev bash -c "cd packages/crypto-wasm && wasm-pack build --target web"
        ;;
    *)
        echo "Usage: $0 {build|shell|up|down|clean|logs|test|fmt|clippy|wasm}"
        echo ""
        echo "Commands:"
        echo "  build   - Build the dev Docker image"
        echo "  shell   - Start and attach to the dev container (default)"
        echo "  up      - Start all services in background"
        echo "  down    - Stop all services"
        echo "  clean   - Stop all services and remove volumes"
        echo "  logs    - Follow service logs (optionally specify service name)"
        echo "  test    - Run cargo test in the dev container"
        echo "  fmt     - Run cargo fmt in the dev container"
        echo "  clippy  - Run clippy in the dev container"
        echo "  wasm    - Build the crypto-wasm package"
        exit 1
        ;;
esac
