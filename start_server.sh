#!/bin/bash

# Default configuration
DEFAULT_PORT=9000
DEFAULT_PRIMARY_TOKEN="test-token-1"
DEFAULT_SECONDARY_TOKEN="test-token-2"

# Parse command line arguments
PORT=${PORT:-$DEFAULT_PORT}
PRIMARY_TOKEN=${PRIMARY_TOKEN:-$DEFAULT_PRIMARY_TOKEN}
SECONDARY_TOKEN=${SECONDARY_TOKEN:-$DEFAULT_SECONDARY_TOKEN}
RUST_LOG=${RUST_LOG:-info}

# Help function
show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Start the aipriceaction-proxy server"
    echo ""
    echo "Options:"
    echo "  -p, --port PORT          Server port (default: $DEFAULT_PORT)"
    echo "  -t, --primary-token TOKEN Primary authentication token"
    echo "  -s, --secondary-token TOKEN Secondary authentication token"
    echo "  -l, --log-level LEVEL    Log level (default: info)"
    echo "  -h, --help               Show this help message"
    echo ""
    echo "Environment variables:"
    echo "  PORT                     Server port"
    echo "  PRIMARY_TOKEN            Primary authentication token"
    echo "  SECONDARY_TOKEN          Secondary authentication token"
    echo "  RUST_LOG                 Log level"
    echo ""
    echo "Examples:"
    echo "  $0                       # Start with defaults"
    echo "  $0 -p 8080               # Start on port 8080"
    echo "  PORT=9999 $0             # Start on port 9999 via env var"
    echo "  $0 -l debug              # Start with debug logging"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--port)
            PORT="$2"
            shift 2
            ;;
        -t|--primary-token)
            PRIMARY_TOKEN="$2"
            shift 2
            ;;
        -s|--secondary-token)
            SECONDARY_TOKEN="$2"
            shift 2
            ;;
        -l|--log-level)
            RUST_LOG="$2"
            shift 2
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

echo "Starting aipriceaction-proxy server..."
echo "Port: $PORT"
echo "Log level: $RUST_LOG"
echo "Primary token: ${PRIMARY_TOKEN:0:8}..."
echo "Secondary token: ${SECONDARY_TOKEN:0:8}..."
echo ""

# Set environment variables and start the server
export PRIMARY_TOKEN="$PRIMARY_TOKEN"
export SECONDARY_TOKEN="$SECONDARY_TOKEN"
export INTERNAL_PEER_URLS=""
export PORT="$PORT"
export RUST_LOG="$RUST_LOG"

# Start the server in background
cargo run &

# Store the PID
PID=$!
echo "Server started with PID: $PID"
echo "Server running at http://localhost:$PORT"
echo ""
echo "To stop the server, run: kill $PID"
echo "To view logs, run: tail -f [log file]"
echo ""

# Wait a moment and check if server started successfully
sleep 2
if ps -p $PID > /dev/null; then
    echo "Server is running successfully!"
else
    echo "Server failed to start. Check the logs for errors."
    exit 1
fi