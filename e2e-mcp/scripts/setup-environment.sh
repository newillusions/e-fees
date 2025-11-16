#!/bin/bash

# E2E MCP Testing Environment Setup
# Sets up the proper environment for Tauri MCP-based E2E testing

set -e

echo "🚀 Setting up E2E MCP testing environment..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[SETUP]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [[ ! -f "package.json" ]] || [[ ! -d "src-tauri" ]]; then
    print_error "Must be run from the e-fees project root directory"
    exit 1
fi

print_status "Checking project structure..."

# Verify MCP plugin exists
if [[ ! -d "tauri-plugin-mcp" ]]; then
    print_error "Tauri MCP plugin not found. Expected: tauri-plugin-mcp/"
    exit 1
fi

print_success "✅ Project structure validated"

# Build MCP server
print_status "Building Tauri MCP server..."
cd tauri-plugin-mcp/mcp-server-ts

if [[ ! -f "package.json" ]]; then
    print_error "MCP server package.json not found"
    exit 1
fi

# Install dependencies
print_status "Installing MCP server dependencies..."
npm install

# Build the server
print_status "Building MCP server..."
npm run build

if [[ ! -f "build/index.js" ]]; then
    print_error "MCP server build failed - build/index.js not found"
    exit 1
fi

print_success "✅ MCP server built successfully"

# Return to project root
cd ../..

# Create necessary directories
print_status "Creating test result directories..."
mkdir -p e2e-mcp/results/{screenshots,reports,logs}
mkdir -p e2e-mcp/results/artifacts

print_success "✅ Test directories created"

# Verify SurrealDB connection
print_status "Verifying SurrealDB connection..."
if command -v curl &> /dev/null; then
    if curl -s "http://10.0.1.17:8000/health" > /dev/null 2>&1; then
        print_success "✅ SurrealDB is accessible"
    else
        print_warning "⚠️ SurrealDB connection test failed (may not be critical)"
        print_warning "   Database: ws://10.0.1.17:8000"
        print_warning "   Namespace: emittiv"
        print_warning "   Database: projects"
    fi
else
    print_warning "⚠️ curl not available - skipping SurrealDB connection test"
fi

# Check if Tauri app can be built
print_status "Checking Tauri build capability..."
if command -v cargo &> /dev/null; then
    print_success "✅ Rust/Cargo available"
else
    print_error "Rust/Cargo not found - required for Tauri"
    print_error "Install from: https://rustup.rs/"
    exit 1
fi

# Verify Node.js version
print_status "Checking Node.js version..."
NODE_VERSION=$(node --version)
print_success "✅ Node.js version: $NODE_VERSION"

# Check if MCP tools are available globally (for validation)
print_status "Checking MCP development tools..."
if npm list -g @modelcontextprotocol/inspector &> /dev/null; then
    print_success "✅ MCP Inspector available"
else
    print_warning "⚠️ MCP Inspector not installed globally"
    print_warning "   Install with: npm install -g @modelcontextprotocol/inspector"
    print_warning "   Useful for debugging MCP server issues"
fi

# Test MCP server can start
print_status "Testing MCP server startup..."
cd tauri-plugin-mcp/mcp-server-ts

# Start server in background for test
node build/index.js &
SERVER_PID=$!

# Give it time to start
sleep 2

# Check if it's still running
if kill -0 $SERVER_PID 2> /dev/null; then
    print_success "✅ MCP server can start successfully"
    kill $SERVER_PID
    wait $SERVER_PID 2>/dev/null || true
else
    print_error "MCP server failed to start"
    exit 1
fi

cd ../..

# Create environment-specific MCP config
print_status "Creating E2E MCP configuration..."
if [[ -f "claude_desktop_config_e2e.json" ]]; then
    # Verify the config has the right socket path
    if grep -q "/tmp/tauri-mcp-e2e.sock" claude_desktop_config_e2e.json; then
        print_success "✅ E2E MCP configuration ready"
    else
        print_warning "⚠️ E2E MCP config may need socket path update"
    fi
else
    print_error "E2E MCP configuration not found: claude_desktop_config_e2e.json"
    exit 1
fi

# Set up test data safety validation
print_status "Setting up test data safety checks..."

# Create a simple test data validation script
cat > e2e-mcp/scripts/validate-test-data.js << 'EOF'
// Validate that test data has proper "DELETE ME" identification
const testDataPatterns = [
  /DELETE ME/i,
  /test.*\d{4}-\d{2}-\d{2}/i,
  /@delete-me-testing\.com/i
];

export function validateTestRecord(record) {
  const fields = [record.name, record.description, record.email, record.first_name, record.last_name].filter(Boolean);
  
  return fields.some(field => 
    testDataPatterns.some(pattern => pattern.test(field))
  );
}

export function isProductionData(record) {
  return !validateTestRecord(record);
}
EOF

print_success "✅ Test data validation utilities created"

# Create a pre-test cleanup script
print_status "Creating pre-test cleanup script..."
cat > e2e-mcp/scripts/pre-test-cleanup.sh << 'EOF'
#!/bin/bash
echo "🧹 Running pre-test cleanup..."

# Use Node.js to run cleanup
node -e "
import('./e2e-mcp/fixtures/cleanup-utilities.js').then(async (m) => {
  try {
    const report = await m.cleanupAllTestData();
    console.log('Pre-test cleanup completed:', report.totalDeleted, 'records removed');
    
    const verification = await m.verifyCleanup();
    if (verification.isClean) {
      console.log('✅ Database is clean and ready for testing');
    } else {
      console.log('⚠️ Some test data may remain');
    }
  } catch (error) {
    console.error('Pre-test cleanup failed:', error.message);
    process.exit(1);
  }
});
" || {
    echo "❌ Pre-test cleanup failed"
    exit 1
}
EOF

chmod +x e2e-mcp/scripts/pre-test-cleanup.sh

print_success "✅ Pre-test cleanup script created"

# Final validation
print_status "Running final environment validation..."

# Check that all required files exist
REQUIRED_FILES=(
    "e2e-mcp/fixtures/test-data-safe.ts"
    "e2e-mcp/fixtures/cleanup-utilities.ts"
    "e2e-mcp/helpers/mcp-client.ts"
    "e2e-mcp/tests/project-crud.mcp.ts"
    "tauri-plugin-mcp/mcp-server-ts/build/index.js"
    "claude_desktop_config_e2e.json"
)

MISSING_FILES=()

for file in "${REQUIRED_FILES[@]}"; do
    if [[ ! -f "$file" ]]; then
        MISSING_FILES+=("$file")
    fi
done

if [[ ${#MISSING_FILES[@]} -eq 0 ]]; then
    print_success "✅ All required files present"
else
    print_error "Missing required files:"
    for file in "${MISSING_FILES[@]}"; do
        print_error "  - $file"
    done
    exit 1
fi

# Create quick test script
print_status "Creating quick test runner..."
cat > e2e-mcp/scripts/run-tests.sh << 'EOF'
#!/bin/bash

# Quick test runner for MCP-based E2E tests
set -e

echo "🚀 Starting MCP-based E2E tests..."

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Parse command line arguments
UI_MODE=false
CLEANUP=true
COVERAGE=false

for arg in "$@"; do
    case $arg in
        --ui)
            UI_MODE=true
            shift
            ;;
        --no-cleanup)
            CLEANUP=false
            shift
            ;;
        --coverage)
            COVERAGE=true
            shift
            ;;
        *)
            # Unknown argument
            ;;
    esac
done

# Pre-test cleanup
if [[ "$CLEANUP" == "true" ]]; then
    print_status "Running pre-test cleanup..."
    ./e2e-mcp/scripts/pre-test-cleanup.sh
fi

# Start Tauri app in development mode
print_status "Starting Tauri application..."
npm run tauri:dev &
TAURI_PID=$!

# Give Tauri time to start
print_status "Waiting for Tauri application to be ready..."
sleep 10

# Function to cleanup on exit
cleanup() {
    print_status "Cleaning up processes..."
    if [[ -n "$TAURI_PID" ]]; then
        kill $TAURI_PID 2>/dev/null || true
        wait $TAURI_PID 2>/dev/null || true
    fi
}

# Set trap to cleanup on script exit
trap cleanup EXIT

# Run tests
if [[ "$UI_MODE" == "true" ]]; then
    print_status "Running tests in UI mode..."
    npm run test:e2e:mcp:ui
elif [[ "$COVERAGE" == "true" ]]; then
    print_status "Running tests with coverage..."
    npm run test:e2e:mcp:coverage
else
    print_status "Running tests..."
    npm run test:e2e:mcp
fi

# Post-test cleanup
if [[ "$CLEANUP" == "true" ]]; then
    print_status "Running post-test cleanup..."
    npm run test:e2e:cleanup
    npm run test:e2e:verify-clean
fi

print_success "🎉 MCP E2E tests completed successfully!"
EOF

chmod +x e2e-mcp/scripts/run-tests.sh

print_success "✅ Test runner created"

# Summary
echo ""
echo "🎉 E2E MCP Testing Environment Setup Complete!"
echo ""
echo "📋 Summary:"
echo "  ✅ Tauri MCP server built and tested"
echo "  ✅ Test directories created"
echo "  ✅ Safety utilities configured"
echo "  ✅ Cleanup scripts ready"
echo "  ✅ Test runner prepared"
echo ""
echo "🚀 Ready to run tests:"
echo "  npm run test:e2e:mcp        # Run all MCP-based E2E tests"
echo "  npm run test:e2e:mcp:ui     # Interactive test UI"
echo "  npm run test:e2e:safe       # Run tests with automatic cleanup"
echo "  ./e2e-mcp/scripts/run-tests.sh --ui  # Custom test runner"
echo ""
echo "🧹 Cleanup commands:"
echo "  npm run test:e2e:cleanup              # Clean all test data"
echo "  npm run test:e2e:verify-clean         # Verify database is clean"
echo "  npm run test:e2e:emergency-cleanup    # Emergency cleanup"
echo ""
echo "⚠️  IMPORTANT: All test data includes 'DELETE ME' identification"
echo "   This ensures safe testing on the live database"
echo ""
print_success "Environment setup completed successfully! 🎯"