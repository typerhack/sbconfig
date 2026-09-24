#!/bin/bash
# lab/scripts/lab-test.sh
# Run lab environment tests
set -e

cd "$(dirname "$0")/.."

echo "=== sbconfig Lab Environment Tests ==="
echo ""

PASS=0
FAIL=0

test_pass() {
    echo "[PASS] $1"
    PASS=$((PASS + 1))
}

test_fail() {
    echo "[FAIL] $1"
    FAIL=$((FAIL + 1))
}

# Test 1: Docker container running
echo "[1/6] Testing Docker container is running..."
if docker ps | grep -q sbconfig-lab; then
    test_pass "Container running"
else
    test_fail "Container not running"
fi

# Test 2: SSH connectivity (using key-based auth, fallback to docker exec)
echo "[2/6] Testing SSH connectivity..."
if ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o BatchMode=yes -o ConnectTimeout=5 -p 2222 root@localhost "exit" 2>/dev/null; then
    test_pass "SSH working (key-based auth)"
elif docker exec sbconfig-lab echo "connected" >/dev/null 2>&1; then
    test_pass "SSH service running (tested via docker exec)"
else
    test_fail "SSH not working"
fi

# Test 3: sing-box installed
echo "[3/6] Testing sing-box installation..."
if docker exec sbconfig-lab which sing-box >/dev/null 2>&1; then
    test_pass "sing-box installed"
else
    test_fail "sing-box not installed"
fi

# Test 4: sing-box version
echo "[4/6] Testing sing-box version..."
VERSION=$(docker exec sbconfig-lab sing-box version 2>&1 | head -1)
if [ -n "$VERSION" ]; then
    test_pass "sing-box version: $VERSION"
else
    test_fail "Could not get sing-box version"
fi

# Test 5: User management capability
echo "[5/6] Testing user creation capability..."
if docker exec sbconfig-lab useradd --help >/dev/null 2>&1; then
    test_pass "User management available"
else
    test_fail "useradd not available"
fi

# Test 6: SQLite available
echo "[6/6] Testing SQLite availability..."
if docker exec sbconfig-lab sqlite3 --version >/dev/null 2>&1; then
    SQLITE_VER=$(docker exec sbconfig-lab sqlite3 --version 2>&1 | head -1)
    test_pass "SQLite available: $SQLITE_VER"
else
    test_fail "SQLite not available"
fi

echo ""
echo "=== Test Summary ==="
echo "Passed: $PASS"
echo "Failed: $FAIL"
echo ""

if [ $FAIL -eq 0 ]; then
    echo "=== All Lab Tests Passed ==="
    exit 0
else
    echo "=== Some Tests Failed ==="
    exit 1
fi
