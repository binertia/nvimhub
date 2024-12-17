SCRIPT_DIR="$(dirname "$0")"

echo "Starting Go program..."
cd "$SCRIPT_DIR/gateway" || exit 1
go run . &
GATEWAY_PID=$!
echo "gateway started with PID $GATEWAY_PID"
pwd

echo "Starting Rust program..."
cd "../fetcher" || exit 1
pwd
cargo run &
SERVICE_PID=$!
echo "fetch service program started with PID $SERVICE_PID"

wait $GATEWAY_PID
wait $SERVICE_PID
