#!/bin/bash

echo "Testing Multi-Protocol Server on port 9876"
echo "=========================================="
echo ""

# Test 1: HTTP
echo "1. Testing HTTP protocol:"
echo "   curl http://127.0.0.1:9876"
curl -s http://127.0.0.1:9876 | head -n 5
echo ""
echo ""

# Test 2: WebSocket (using websocat if available)
echo "2. Testing WebSocket protocol:"
if command -v websocat &> /dev/null; then
    echo "   echo 'Hello WebSocket' | websocat ws://127.0.0.1:9876"
    echo "Hello WebSocket" | timeout 2 websocat ws://127.0.0.1:9876 || true
else
    echo "   (websocat not installed, skipping)"
fi
echo ""
echo ""

# Test 3: Raw TCP
echo "3. Testing Raw TCP protocol:"
echo "   echo 'Hello TCP' | nc 127.0.0.1 9876"
echo "Hello TCP" | timeout 2 nc 127.0.0.1 9876 || true
echo ""
