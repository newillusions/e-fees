#!/bin/bash
# E-Fees Performance Testing Infrastructure Setup Script
# 
# Installs and configures all dependencies for comprehensive
# performance testing of the E-Fees desktop application.

set -e  # Exit on any error

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PERF_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$PERF_DIR")"
NODE_VERSION="18"

echo -e "${BLUE}🚀 E-Fees Performance Testing Setup${NC}"
echo -e "${BLUE}====================================${NC}"
echo ""

# Function to print status messages
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}📊 $1${NC}"
}

# Check system requirements
check_system_requirements() {
    print_info "Checking system requirements..."
    
    # Check Node.js version
    if command -v node &> /dev/null; then
        NODE_CURRENT=$(node -v | cut -d'v' -f2 | cut -d'.' -f1)
        if [ "$NODE_CURRENT" -ge "$NODE_VERSION" ]; then
            print_status "Node.js $NODE_CURRENT detected (required: $NODE_VERSION+)"
        else
            print_error "Node.js $NODE_VERSION+ required, found $NODE_CURRENT"
            echo "Please install Node.js $NODE_VERSION+ from https://nodejs.org/"
            exit 1
        fi
    else
        print_error "Node.js not found"
        echo "Please install Node.js $NODE_VERSION+ from https://nodejs.org/"
        exit 1
    fi
    
    # Check npm
    if ! command -v npm &> /dev/null; then
        print_error "npm not found"
        exit 1
    fi
    
    # Check if we're on macOS for some specific tools
    OS="$(uname -s)"
    case "${OS}" in
        Linux*)     MACHINE=Linux;;
        Darwin*)    MACHINE=Mac;;
        CYGWIN*)    MACHINE=Cygwin;;
        MINGW*)     MACHINE=MinGw;;
        *)          MACHINE="UNKNOWN:${OS}"
    esac
    print_status "Operating System: $MACHINE"
    
    # Check available memory
    if [ "$MACHINE" = "Mac" ]; then
        TOTAL_MEM_GB=$(( $(sysctl -n hw.memsize) / 1024 / 1024 / 1024 ))
    elif [ "$MACHINE" = "Linux" ]; then
        TOTAL_MEM_GB=$(( $(grep MemTotal /proc/meminfo | awk '{print $2}') / 1024 / 1024 ))
    else
        TOTAL_MEM_GB=8  # Default assumption
    fi
    
    if [ "$TOTAL_MEM_GB" -lt 4 ]; then
        print_warning "Low memory detected (${TOTAL_MEM_GB}GB). Performance testing may be limited."
    else
        print_status "Memory: ${TOTAL_MEM_GB}GB (sufficient for performance testing)"
    fi
}

# Install k6 for load testing
install_k6() {
    print_info "Installing k6 for load testing..."
    
    if command -v k6 &> /dev/null; then
        K6_VERSION=$(k6 version | head -n1 | cut -d' ' -f2)
        print_status "k6 $K6_VERSION already installed"
        return
    fi
    
    case "${MACHINE}" in
        Mac)
            if command -v brew &> /dev/null; then
                print_info "Installing k6 via Homebrew..."
                brew install k6
            else
                print_warning "Homebrew not found, installing k6 manually..."
                curl https://github.com/grafana/k6/releases/download/v0.52.0/k6-v0.52.0-macos-amd64.zip -L -o k6.zip
                unzip k6.zip
                sudo mv k6-v0.52.0-macos-amd64/k6 /usr/local/bin/
                rm -rf k6.zip k6-v0.52.0-macos-amd64
            fi
            ;;
        Linux)
            print_info "Installing k6 on Linux..."
            sudo gpg -k
            sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
            echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
            sudo apt-get update
            sudo apt-get install k6
            ;;
        *)
            print_warning "Unsupported OS for automatic k6 installation. Please install k6 manually."
            echo "Visit: https://k6.io/docs/get-started/installation/"
            ;;
    esac
    
    if command -v k6 &> /dev/null; then
        print_status "k6 successfully installed"
    else
        print_error "k6 installation failed"
        exit 1
    fi
}

# Install performance testing dependencies
install_performance_dependencies() {
    print_info "Installing performance testing dependencies..."
    
    cd "$PERF_DIR"
    
    # Install dependencies from performance/package.json
    if [ -f "package.json" ]; then
        print_info "Installing npm dependencies..."
        npm install
        print_status "Performance testing dependencies installed"
    else
        print_error "performance/package.json not found"
        exit 1
    fi
    
    # Install global tools if needed
    print_info "Checking global npm tools..."
    
    # Check for Playwright
    if ! npm list -g @playwright/test &> /dev/null; then
        print_info "Installing Playwright globally..."
        npm install -g @playwright/test
    fi
    
    # Install Playwright browsers
    print_info "Installing Playwright browsers..."
    npx playwright install
    
    print_status "All performance testing dependencies installed"
}

# Setup SurrealDB for testing
setup_test_database() {
    print_info "Setting up test database environment..."
    
    # Check if SurrealDB is running
    if ! curl -s "http://10.0.1.17:8000/status" &> /dev/null; then
        print_warning "Main SurrealDB instance not accessible at 10.0.1.17:8000"
        print_info "Setting up local SurrealDB for testing..."
        
        # Install SurrealDB if not present
        if ! command -v surreal &> /dev/null; then
            case "${MACHINE}" in
                Mac)
                    print_info "Installing SurrealDB on macOS..."
                    if command -v brew &> /dev/null; then
                        brew install surrealdb/tap/surreal
                    else
                        curl --proto '=https' --tlsv1.2 -sSf https://install.surrealdb.com | sh
                    fi
                    ;;
                Linux)
                    print_info "Installing SurrealDB on Linux..."
                    curl --proto '=https' --tlsv1.2 -sSf https://install.surrealdb.com | sh
                    ;;
                *)
                    print_warning "Please install SurrealDB manually: https://surrealdb.com/install"
                    ;;
            esac
        fi
        
        # Create test database startup script
        cat > "$PERF_DIR/scripts/start-test-db.sh" << 'EOF'
#!/bin/bash
echo "🗄️  Starting test SurrealDB instance..."
surreal start --log trace --user root --pass root memory &
SURREAL_PID=$!
echo "SurrealDB started with PID: $SURREAL_PID"
echo $SURREAL_PID > /tmp/test-surrealdb.pid
echo "✅ Test database ready at ws://localhost:8000"
EOF
        chmod +x "$PERF_DIR/scripts/start-test-db.sh"
        
        cat > "$PERF_DIR/scripts/stop-test-db.sh" << 'EOF'
#!/bin/bash
if [ -f /tmp/test-surrealdb.pid ]; then
    PID=$(cat /tmp/test-surrealdb.pid)
    echo "🛑 Stopping test SurrealDB (PID: $PID)..."
    kill $PID
    rm /tmp/test-surrealdb.pid
    echo "✅ Test database stopped"
else
    echo "⚠️  Test database PID file not found"
fi
EOF
        chmod +x "$PERF_DIR/scripts/stop-test-db.sh"
        
        print_status "Test database scripts created"
    else
        print_status "SurrealDB accessible at 10.0.1.17:8000"
    fi
}

# Create directory structure
setup_directory_structure() {
    print_info "Setting up performance testing directory structure..."
    
    cd "$PERF_DIR"
    
    # Create necessary directories
    mkdir -p data/generated
    mkdir -p reports/html
    mkdir -p reports/json
    mkdir -p reports/trends
    mkdir -p utils/monitoring
    mkdir -p utils/helpers
    mkdir -p docker
    mkdir -p logs
    
    print_status "Directory structure created"
}

# Setup Docker environment (optional)
setup_docker_environment() {
    print_info "Setting up Docker environment for isolated testing..."
    
    if ! command -v docker &> /dev/null; then
        print_warning "Docker not found. Skipping Docker setup."
        return
    fi
    
    # Create Docker Compose file for test environment
    cat > "$PERF_DIR/docker/docker-compose.yml" << 'EOF'
version: '3.8'

services:
  surrealdb-test:
    image: surrealdb/surrealdb:latest
    ports:
      - "8000:8000"
    command: start --log trace --user root --pass root memory
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
      interval: 10s
      timeout: 5s
      retries: 5
    
  k6-runner:
    image: grafana/k6:latest
    volumes:
      - "../tests:/scripts"
      - "../reports:/reports"
    depends_on:
      - surrealdb-test
    profiles:
      - testing
      
  performance-monitor:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - "./prometheus.yml:/etc/prometheus/prometheus.yml"
    profiles:
      - monitoring
EOF
    
    # Create Prometheus config for monitoring
    cat > "$PERF_DIR/docker/prometheus.yml" << 'EOF'
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files: []

scrape_configs:
  - job_name: 'performance-tests'
    static_configs:
      - targets: ['host.docker.internal:3000']
    scrape_interval: 5s
    metrics_path: '/metrics'
EOF
    
    print_status "Docker environment configured"
}

# Generate test configuration files
generate_config_files() {
    print_info "Generating test configuration files..."
    
    # Create environment configuration
    cat > "$PERF_DIR/.env.example" << 'EOF'
# E-Fees Performance Testing Configuration

# Database Configuration
SURREALDB_URL=ws://10.0.1.17:8000
SURREALDB_NS=emittiv
SURREALDB_DB=projects
SURREALDB_USER=martin
SURREALDB_PASS=your_password

# Test Database Configuration  
SURREALDB_TEST_URL=ws://localhost:8000
SURREALDB_TEST_NS=emittiv_test
SURREALDB_TEST_DB=projects_test
SURREALDB_TEST_USER=root
SURREALDB_TEST_PASS=root

# Application Configuration
APP_URL=http://localhost:1420
APP_BUILD_CMD=npm run tauri:dev

# Performance Test Configuration
TEST_ENV=development
MAX_BATCH_SIZE=1000
TEST_ITERATIONS=10
WARMUP_ITERATIONS=2
MEMORY_DEBUG=false

# Reporting Configuration
REPORT_OUTPUT_DIR=./reports
HISTORICAL_DATA_RETENTION_DAYS=30
AUTO_CLEANUP_REPORTS=true

# k6 Cloud (optional)
K6_PROJECT_ID=
K6_TOKEN=
EOF
    
    # Create run configuration
    cat > "$PERF_DIR/performance.config.json" << 'EOF'
{
  "name": "E-Fees Performance Testing Suite",
  "version": "1.0.0",
  "testSuites": {
    "smoke": {
      "description": "Quick smoke tests for basic functionality",
      "tests": [
        "tests/database/basic-operations.js",
        "tests/ui/component-rendering.js"
      ],
      "duration": "2m",
      "vus": 1
    },
    "load": {
      "description": "Normal load testing scenarios", 
      "tests": [
        "tests/database/bulk-operations.js",
        "tests/ui/large-lists.spec.js",
        "tests/workflows/complete-workflows.js"
      ],
      "duration": "10m",
      "vus": 10
    },
    "stress": {
      "description": "High load stress testing",
      "tests": [
        "tests/database/bulk-operations.js",
        "tests/workflows/complete-workflows.js"
      ],
      "duration": "20m",
      "vus": 50
    },
    "endurance": {
      "description": "Long-running stability testing",
      "tests": [
        "tests/memory/memory-leak-detection.js",
        "tests/workflows/complete-workflows.js"
      ],
      "duration": "60m",
      "vus": 5
    }
  },
  "thresholds": {
    "response_time_p95": 500,
    "error_rate": 0.01,
    "memory_growth_mb_per_hour": 10
  },
  "environments": {
    "development": {
      "baseUrl": "http://localhost:1420",
      "databaseUrl": "ws://localhost:8000"
    },
    "testing": {
      "baseUrl": "http://localhost:3000", 
      "databaseUrl": "ws://localhost:8000"
    }
  }
}
EOF
    
    print_status "Configuration files generated"
}

# Install additional tools
install_additional_tools() {
    print_info "Installing additional performance monitoring tools..."
    
    # Check for Python (needed for some monitoring scripts)
    if command -v python3 &> /dev/null; then
        print_status "Python 3 detected"
        
        # Install Python packages for advanced analysis
        if command -v pip3 &> /dev/null; then
            pip3 install --user psutil matplotlib numpy pandas 2>/dev/null || true
        fi
    else
        print_warning "Python 3 not found. Some advanced analysis features may not work."
    fi
    
    # Install additional Node.js tools
    print_info "Installing additional Node.js monitoring tools..."
    npm install -g clinic@latest 2>/dev/null || print_warning "Failed to install clinic.js"
    
    print_status "Additional tools installation complete"
}

# Validate installation
validate_installation() {
    print_info "Validating performance testing installation..."
    
    local errors=0
    
    # Check k6
    if ! command -v k6 &> /dev/null; then
        print_error "k6 not found in PATH"
        ((errors++))
    fi
    
    # Check Node.js dependencies
    cd "$PERF_DIR"
    if ! npm list k6 &> /dev/null; then
        print_error "k6 npm package not found"
        ((errors++))
    fi
    
    if ! npm list playwright &> /dev/null; then
        print_error "Playwright not found"
        ((errors++))
    fi
    
    # Check directory structure
    for dir in "data/generated" "reports/html" "reports/json" "tests" "utils"; do
        if [ ! -d "$dir" ]; then
            print_error "Directory $dir not found"
            ((errors++))
        fi
    done
    
    # Check configuration files
    for file in "package.json" "performance.config.json" ".env.example"; do
        if [ ! -f "$file" ]; then
            print_error "Configuration file $file not found"
            ((errors++))
        fi
    done
    
    if [ $errors -eq 0 ]; then
        print_status "All validation checks passed"
        return 0
    else
        print_error "$errors validation errors found"
        return 1
    fi
}

# Generate sample test data
generate_sample_data() {
    print_info "Generating sample test data..."
    
    cd "$PERF_DIR"
    
    # Run data generator for small dataset
    if [ -f "data/generators/generate-large-dataset.js" ]; then
        node data/generators/generate-large-dataset.js small 2>/dev/null || print_warning "Sample data generation failed"
        print_status "Sample test data generated"
    else
        print_warning "Data generator not found, skipping sample data generation"
    fi
}

# Create convenience scripts
create_convenience_scripts() {
    print_info "Creating convenience scripts..."
    
    # Create run-all-tests script
    cat > "$PERF_DIR/scripts/run-all-tests.sh" << 'EOF'
#!/bin/bash
set -e

echo "🚀 Running complete E-Fees performance test suite..."

# Start test database if needed
if ! curl -s "http://localhost:8000/status" &> /dev/null; then
    echo "Starting test database..."
    ./scripts/start-test-db.sh
    sleep 3
fi

# Run database tests
echo "📊 Running database performance tests..."
npm run perf:test:database

# Run UI tests  
echo "🎨 Running UI performance tests..."
npm run perf:test:ui

# Run workflow tests
echo "🔄 Running workflow tests..."
npm run perf:test:workflows

# Run memory tests
echo "💾 Running memory leak detection..."
npm run perf:test:memory

# Generate comprehensive report
echo "📊 Generating performance report..."
npm run perf:report

echo "✅ Complete performance test suite finished!"
echo "📄 Check the reports directory for detailed results"
EOF
    chmod +x "$PERF_DIR/scripts/run-all-tests.sh"
    
    # Create quick smoke test script
    cat > "$PERF_DIR/scripts/run-smoke-tests.sh" << 'EOF'
#!/bin/bash
set -e

echo "💨 Running smoke tests..."

# Quick database test
k6 run --duration 30s --vus 1 tests/database/bulk-operations.js

# Quick UI test
npm run test:ui -- --timeout 30000 tests/ui/large-lists.spec.js

echo "✅ Smoke tests complete!"
EOF
    chmod +x "$PERF_DIR/scripts/run-smoke-tests.sh"
    
    # Create cleanup script
    cat > "$PERF_DIR/scripts/cleanup.sh" << 'EOF'
#!/bin/bash

echo "🧹 Cleaning up performance test artifacts..."

# Stop test database
./scripts/stop-test-db.sh 2>/dev/null || true

# Clean generated data
rm -rf data/generated/*

# Clean old reports (keep last 5)
cd reports/html
ls -t *.html | tail -n +6 | xargs rm -f 2>/dev/null || true

cd ../json  
ls -t *.json | tail -n +6 | xargs rm -f 2>/dev/null || true

echo "✅ Cleanup complete!"
EOF
    chmod +x "$PERF_DIR/scripts/cleanup.sh"
    
    print_status "Convenience scripts created"
}

# Main installation process
main() {
    echo -e "${BLUE}Starting E-Fees Performance Testing Infrastructure Setup...${NC}"
    echo ""
    
    check_system_requirements
    echo ""
    
    install_k6
    echo ""
    
    setup_directory_structure
    echo ""
    
    install_performance_dependencies  
    echo ""
    
    setup_test_database
    echo ""
    
    setup_docker_environment
    echo ""
    
    generate_config_files
    echo ""
    
    install_additional_tools
    echo ""
    
    create_convenience_scripts
    echo ""
    
    generate_sample_data
    echo ""
    
    if validate_installation; then
        echo ""
        echo -e "${GREEN}🎉 Performance Testing Infrastructure Setup Complete!${NC}"
        echo ""
        echo -e "${BLUE}Next Steps:${NC}"
        echo -e "  1. Copy ${YELLOW}.env.example${NC} to ${YELLOW}.env${NC} and configure your settings"
        echo -e "  2. Run ${YELLOW}npm run perf:test${NC} to execute the complete test suite"
        echo -e "  3. Run ${YELLOW}./scripts/run-smoke-tests.sh${NC} for quick validation"
        echo -e "  4. Check ${YELLOW}./reports/html/${NC} for detailed performance reports"
        echo ""
        echo -e "${BLUE}Available Commands:${NC}"
        echo -e "  ${YELLOW}npm run perf:test${NC}           - Run all performance tests"
        echo -e "  ${YELLOW}npm run perf:test:database${NC}  - Run database tests only"
        echo -e "  ${YELLOW}npm run perf:test:ui${NC}        - Run UI tests only"  
        echo -e "  ${YELLOW}npm run perf:test:workflows${NC} - Run workflow tests only"
        echo -e "  ${YELLOW}npm run perf:test:memory${NC}    - Run memory leak detection"
        echo -e "  ${YELLOW}npm run perf:report${NC}         - Generate HTML performance report"
        echo -e "  ${YELLOW}npm run perf:baseline${NC}       - Create performance baseline"
        echo -e "  ${YELLOW}npm run perf:compare${NC}        - Compare with previous results"
        echo ""
    else
        echo ""
        print_error "Setup completed with errors. Please review the issues above."
        exit 1
    fi
}

# Run main function
main "$@"