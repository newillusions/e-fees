#!/bin/bash

# E2E Test Runner Script
# Provides convenient wrapper for running E2E tests with proper setup

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SURREALDB_PORT=8001
PROJECT_ROOT=$(dirname "$(dirname "$(realpath "$0")")")

echo -e "${BLUE}🧪 E-Fees E2E Test Runner${NC}"
echo "=================================="

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check if port is in use
port_in_use() {
    lsof -Pi :$1 -sTCP:LISTEN -t >/dev/null 2>&1
}

# Function to kill process on port
kill_port() {
    if port_in_use $1; then
        echo -e "${YELLOW}⚠️  Port $1 is in use, attempting to free it...${NC}"
        lsof -ti:$1 | xargs kill -9 2>/dev/null || true
        sleep 2
    fi
}

# Check prerequisites
echo -e "${BLUE}📋 Checking prerequisites...${NC}"

if ! command_exists node; then
    echo -e "${RED}❌ Node.js is required but not installed${NC}"
    exit 1
fi

if ! command_exists npm; then
    echo -e "${RED}❌ npm is required but not installed${NC}"
    exit 1
fi

if ! command_exists surreal; then
    echo -e "${RED}❌ SurrealDB is required but not installed${NC}"
    echo "   Install from: https://surrealdb.com/install"
    exit 1
fi

echo -e "${GREEN}✅ All prerequisites found${NC}"

# Parse command line arguments
TEST_MODE="full"
HEADED=""
DEBUG=""
PATTERN=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --dev)
            TEST_MODE="dev"
            shift
            ;;
        --headed)
            HEADED="--headed"
            shift
            ;;
        --debug)
            DEBUG="--debug"
            shift
            ;;
        --ui)
            TEST_MODE="ui"
            shift
            ;;
        --pattern)
            PATTERN="--grep $2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --dev      Use development mode (with running dev server)"
            echo "  --headed   Run tests with visible browser"
            echo "  --debug    Run tests in debug mode"
            echo "  --ui       Run tests with interactive UI"
            echo "  --pattern  Run tests matching pattern"
            echo "  --help     Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}🔧 Test Configuration:${NC}"
echo "   Mode: $TEST_MODE"
echo "   Headed: ${HEADED:-"false"}"
echo "   Debug: ${DEBUG:-"false"}"
echo "   Pattern: ${PATTERN:-"all tests"}"

# Setup environment
echo -e "${BLUE}🌍 Setting up environment...${NC}"

export SURREALDB_URL="ws://localhost:$SURREALDB_PORT"
export SURREALDB_NS="test"
export SURREALDB_DB="e2e"
export SURREALDB_USER="root"
export SURREALDB_PASS="test"
export PROJECT_FOLDER_PATH="$PROJECT_ROOT/e2e/test-projects"
export NODE_ENV="test"

if [ "$TEST_MODE" = "dev" ]; then
    export E2E_DEV_MODE="true"
fi

# Create test directories
mkdir -p "$PROJECT_FOLDER_PATH"
mkdir -p "$PROJECT_ROOT/e2e/reports"
mkdir -p "$PROJECT_ROOT/e2e/test-results"

# Check if we need to start SurrealDB
if [ "$TEST_MODE" != "dev" ] || ! port_in_use $SURREALDB_PORT; then
    echo -e "${BLUE}🗄️  Starting SurrealDB test instance...${NC}"
    kill_port $SURREALDB_PORT
    
    # Start SurrealDB in background
    surreal start --log warn --user root --pass test --bind "0.0.0.0:$SURREALDB_PORT" memory &
    SURREALDB_PID=$!
    
    # Wait for SurrealDB to be ready
    echo -e "${YELLOW}⏳ Waiting for SurrealDB to start...${NC}"
    for i in {1..10}; do
        if curl -s "http://localhost:$SURREALDB_PORT/version" >/dev/null 2>&1; then
            echo -e "${GREEN}✅ SurrealDB is ready${NC}"
            break
        fi
        if [ $i -eq 10 ]; then
            echo -e "${RED}❌ SurrealDB failed to start${NC}"
            exit 1
        fi
        sleep 1
    done
fi

# Cleanup function
cleanup() {
    echo -e "${BLUE}🧹 Cleaning up...${NC}"
    if [ ! -z "$SURREALDB_PID" ]; then
        kill $SURREALDB_PID 2>/dev/null || true
    fi
    kill_port $SURREALDB_PORT
}

# Set trap for cleanup
trap cleanup EXIT

# Install dependencies if needed
if [ ! -d "$PROJECT_ROOT/node_modules" ]; then
    echo -e "${BLUE}📦 Installing dependencies...${NC}"
    cd "$PROJECT_ROOT"
    npm ci
fi

# Install Playwright browsers if needed
if [ ! -d "$PROJECT_ROOT/playwright-browsers" ] && [ ! -d "$HOME/.cache/ms-playwright" ]; then
    echo -e "${BLUE}🌐 Installing Playwright browsers...${NC}"
    cd "$PROJECT_ROOT"
    npx playwright install chromium
fi

# Run tests based on mode
cd "$PROJECT_ROOT"

case $TEST_MODE in
    "dev")
        echo -e "${BLUE}🚀 Running E2E tests in development mode...${NC}"
        npm run test:e2e:dev -- $HEADED $DEBUG $PATTERN
        ;;
    "ui")
        echo -e "${BLUE}🖥️  Running E2E tests with interactive UI...${NC}"
        npm run test:e2e:ui -- $PATTERN
        ;;
    "full")
        echo -e "${BLUE}🧪 Running full E2E test suite...${NC}"
        npm run test:e2e -- $HEADED $DEBUG $PATTERN
        ;;
esac

TEST_EXIT_CODE=$?

# Report results
if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed!${NC}"
else
    echo -e "${RED}❌ Some tests failed (exit code: $TEST_EXIT_CODE)${NC}"
fi

# Show report location
if [ -f "$PROJECT_ROOT/e2e/reports/html/index.html" ]; then
    echo -e "${BLUE}📊 Test report available at:${NC}"
    echo "   file://$PROJECT_ROOT/e2e/reports/html/index.html"
fi

exit $TEST_EXIT_CODE