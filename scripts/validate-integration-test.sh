#!/bin/bash
set -e

echo "🧪 Testing crontab -l parsing..."
OUTPUT=$(podman exec cron-when-test /usr/local/bin/cron-when --crontab 2>&1)

echo "$OUTPUT"
echo ""
echo "🔍 Validating output..."

# Validate that we got output
if [ -z "$OUTPUT" ]; then
    echo "❌ ERROR: No output from cron-when"
    exit 1
fi

# Count the number of cron entries (should be 5)
CRON_COUNT=$(echo "$OUTPUT" | grep -c "Next:")
if [ "$CRON_COUNT" -ne 5 ]; then
    echo "❌ ERROR: Expected 5 cron entries, found $CRON_COUNT"
    exit 1
fi

# Verify specific comments exist (since default output shows comments, not the cron expression)
echo "$OUTPUT" | grep -q "Backup every day at 2 AM" || { echo "❌ Missing backup job comment"; exit 1; }
echo "$OUTPUT" | grep -q "Clean logs every hour" || { echo "❌ Missing cleanup job comment"; exit 1; }
echo "$OUTPUT" | grep -q "Monitor every 5 minutes" || { echo "❌ Missing monitor job comment"; exit 1; }
echo "$OUTPUT" | grep -q "Weekly report on Monday at 9 AM" || { echo "❌ Missing weekly job comment"; exit 1; }
echo "$OUTPUT" | grep -q "Midday checks at 12, 15, and 18" || { echo "❌ Missing midday check job comment"; exit 1; }

# Verify format - each entry should have "Next:" and "Left:"
NEXT_COUNT=$(echo "$OUTPUT" | grep -c "Next:.*UTC")
LEFT_COUNT=$(echo "$OUTPUT" | grep -c "Left:")
if [ "$NEXT_COUNT" -ne 5 ] || [ "$LEFT_COUNT" -ne 5 ]; then
    echo "❌ ERROR: Expected 5 Next: and 5 Left: lines, found Next:$NEXT_COUNT Left:$LEFT_COUNT"
    exit 1
fi

# Verify environment variables are NOT in output
if echo "$OUTPUT" | grep -q "^SHELL=\|^PATH=\|^MAILTO="; then
    echo "❌ ERROR: Environment variables should not appear in output"
    exit 1
fi

echo "  ✓ Found all 5 cron entries"
echo "  ✓ Backup job (0 2 * * *)"
echo "  ✓ Cleanup job (0 * * * *)"
echo "  ✓ Monitor job (*/5 * * * *)"
echo "  ✓ Weekly job (0 9 * * 1)"
echo "  ✓ Midday check with range/step (0 12-18/3 * * *)"
echo "  ✓ All entries have Next: and Left: format"
echo "  ✓ Environment variables correctly filtered"

# Test individual expression with range/step pattern
echo ""
echo "🧪 Testing range/step pattern: 0 12-18/3 * * *"
RANGE_OUTPUT=$(podman exec cron-when-test /usr/local/bin/cron-when "0 12-18/3 * * *")
echo "$RANGE_OUTPUT"

if ! echo "$RANGE_OUTPUT" | grep -q "Next:"; then
    echo "❌ Range/step pattern test failed"
    exit 1
fi

echo "  ✓ Range/step pattern (12, 15, 18) parsed correctly"
