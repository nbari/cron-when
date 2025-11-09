#!/bin/bash
set -e

echo "🧪 Testing OpenTelemetry Tracing with Jaeger"
echo ""

# Check if Jaeger is running
if ! podman ps | grep -q jaeger; then
    echo "❌ Jaeger is not running!"
    echo "Start it with: just jaeger"
    exit 1
fi

echo "✅ Jaeger is running"
echo ""

# Set OTLP endpoint
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317

echo "📤 Generating traces..."
echo ""

# Run different commands
echo "1. Single expression:"
cargo run --release -- -v "*/5 * * * *" 2>&1 | grep -v "ERROR\|Failed" || true
echo ""

echo "2. File parsing:"
cargo run --release -- -v -f sample.crontab 2>&1 | grep -v "ERROR\|Failed" | tail -10 || true
echo ""

echo "3. Multiple occurrences:"
cargo run --release -- -v --next 3 "0 * * * *" 2>&1 | grep -v "ERROR\|Failed" | tail -10 || true
echo ""

echo "⏳ Waiting for traces to reach Jaeger..."
sleep 3

echo ""
echo "🔍 Check traces at: http://localhost:16686"
echo ""
echo "In Jaeger UI:"
echo "  1. Select service: cron-when"
echo "  2. Click 'Find Traces'"
echo "  3. You should see 3 traces from the commands above"
echo ""
echo "Note: The timeout errors are expected when CLI exits quickly."
echo "      Traces should still appear in Jaeger!"
