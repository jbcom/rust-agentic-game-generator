#!/bin/bash

echo "Testing Vintage Game Generator..."
echo "================================"

# Build the app
echo "Building app..."
cargo build --release 2>&1 | tail -5

# Run the app for 3 seconds and capture logs
echo -e "\nRunning app and checking for errors..."
timeout 3 cargo run --release 2>&1 > app_output.log

# Check for the grey screen error
if grep -q "Failed to get EguiContext in draw_generate_ui" app_output.log; then
    echo "❌ ERROR: Grey screen issue detected!"
    echo "The app is failing to get EguiContext"
    grep -A2 -B2 "Failed to get EguiContext" app_output.log
else
    echo "✅ No grey screen errors detected"
fi

# Check if the welcome step is being drawn
if grep -q "Drawing welcome step" app_output.log; then
    echo "✅ Welcome screen is rendering"
else
    echo "⚠️  Welcome screen may not be rendering"
fi

# Check for successful EguiContext
if grep -q "Successfully got EguiContext" app_output.log; then
    echo "✅ EguiContext is available"
else
    echo "⚠️  EguiContext may not be initializing properly"
fi

# Show any errors
echo -e "\nErrors found:"
grep -i "error" app_output.log | grep -v "EguiContext not ready" | head -5 || echo "No critical errors found"

echo -e "\nTest complete. Check app_output.log for full details."
