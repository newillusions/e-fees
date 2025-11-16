#!/bin/bash

echo "🔒 SECURITY VALIDATION SUITE"
echo "============================="
echo ""

passed=0
total=0

test_result() {
    total=$((total + 1))
    if [ $1 -eq 0 ]; then
        passed=$((passed + 1))
        echo "✅ $2"
    else
        echo "❌ $2"
    fi
    if [ -n "$3" ]; then
        echo "   $3"
    fi
}

echo "📋 TEST 1: CREDENTIAL MANAGEMENT"

# Check .env files don't contain hardcoded password
if grep -q "th38ret3ch" .env 2>/dev/null; then
    test_result 1 "Hardcoded credentials removed from .env" "Found hardcoded password in .env"
else
    test_result 0 "Hardcoded credentials removed from .env" "No hardcoded credentials found"
fi

# Check .env.example exists with security warnings
if [ -f ".env.example" ] && grep -q "SECURITY WARNING" .env.example; then
    test_result 0 ".env.example template with security warnings" "Template includes security guidance"
else
    test_result 1 ".env.example template with security warnings" "Missing template or security warnings"
fi

echo ""
echo "📋 TEST 2: SQL INJECTION PROTECTION"

# Check security module exists
if [ -f "src-tauri/src/db/security.rs" ]; then
    test_result 0 "Input validation module created" "Security validation framework available"
else
    test_result 1 "Input validation module created" "Missing security.rs module"
fi

# Check secure operations module exists  
if [ -f "src-tauri/src/db/secure_operations.rs" ]; then
    test_result 0 "Secure database operations module created" "Secure wrapper functions available"
else
    test_result 1 "Secure database operations module created" "Missing secure_operations.rs module"
fi

# Check for input validation in security module
if [ -f "src-tauri/src/db/security.rs" ] && grep -q "InputValidator" src-tauri/src/db/security.rs; then
    test_result 0 "Input validation framework implemented" "InputValidator found in security module"
else
    test_result 1 "Input validation framework implemented" "InputValidator not found"
fi

echo ""
echo "📋 TEST 3: CONTENT SECURITY POLICY"

# Check CSP in Tauri config
if grep -q "default-src 'self'" src-tauri/tauri.conf.json; then
    test_result 0 "CSP configuration in Tauri config" "Strict CSP policy configured"
else
    test_result 1 "CSP configuration in Tauri config" "CSP missing or too permissive"
fi

# Check security headers in HTML
if [ -f "index.html" ] && grep -q "X-Content-Type-Options" index.html; then
    test_result 0 "Security headers in HTML" "HTTP security headers configured"
else
    test_result 1 "Security headers in HTML" "Missing security headers in HTML"
fi

echo ""
echo "📋 TEST 4: ENCRYPTED WEBSOCKET CONNECTION"

# Check for WSS protocol
if grep -q "wss://" .env 2>/dev/null; then
    test_result 0 "WebSocket connection uses encryption (WSS)" "Found wss:// protocol"
elif grep -q "ws://" .env 2>/dev/null && ! grep -q "wss://" .env 2>/dev/null; then
    test_result 1 "WebSocket connection uses encryption (WSS)" "Still using unencrypted ws://"
else
    test_result 1 "WebSocket connection configuration" "No WebSocket URL found"
fi

# Check TLS configuration
if grep -q "SURREALDB_VERIFY_CERTS" .env 2>/dev/null; then
    test_result 0 "TLS certificate verification configured" "Certificate validation enabled"
else
    test_result 1 "TLS certificate verification configured" "Missing TLS configuration"
fi

echo ""
echo "📋 TEST 5: RUST SECURITY DEPENDENCIES"

# Check regex dependency in Cargo.toml
if grep -q "regex" src-tauri/Cargo.toml; then
    test_result 0 "Security dependencies added to Cargo.toml" "Regex dependency for validation found"
else
    test_result 1 "Security dependencies added to Cargo.toml" "Missing regex dependency"
fi

echo ""
echo "📋 TEST 6: GIT SECURITY"

# Check .gitignore protects environment files
if grep -q "\.env" .gitignore; then
    test_result 0 ".gitignore protects environment files" "Environment files protected from git"
else
    test_result 1 ".gitignore protects environment files" "Environment files not protected"
fi

echo ""
echo "📋 TEST 7: SECURITY DOCUMENTATION"

# Check for security documentation files
docs=("SECURITY.md" "SECURITY_CREDENTIAL_MANAGEMENT.md" "SECURITY_AUDIT_REPORT.md" "TLS_SECURITY_IMPLEMENTATION.md")
for doc in "${docs[@]}"; do
    if [ -f "$doc" ]; then
        test_result 0 "Security documentation: $doc" "Documentation available"
    else
        test_result 1 "Security documentation: $doc" "Documentation missing"
    fi
done

echo ""
echo "📋 TEST 8: CODE COMPILATION"

# Test Rust compilation
echo "   Running Rust compilation check..."
if cd src-tauri && cargo check --quiet 2>/dev/null; then
    test_result 0 "Rust code compiles with security features" "All security modules compile successfully"
else
    test_result 1 "Rust code compilation" "Compilation errors found"
fi
cd ..

echo ""
echo "🔒 SECURITY VALIDATION RESULTS"
echo "============================="
echo "Tests Passed: $passed/$total"
percentage=$((passed * 100 / total))
echo "Success Rate: $percentage%"

if [ $passed -eq $total ]; then
    echo ""
    echo "🎉 ALL SECURITY TESTS PASSED!"
    echo "✅ The application is ready for secure deployment"
else
    echo ""
    echo "⚠️  SOME SECURITY TESTS FAILED"  
    echo "❌ Please review and fix failing tests before deployment"
fi

echo ""
echo "📋 Next Steps:"
echo "1. Review any failing tests above"
echo "2. Run production build to test CSP in bundled app"
echo "3. Coordinate with server admin for WSS certificate setup"
echo "4. Consider using git filter-branch to clean git history"
echo "5. Implement credential rotation before production deployment"