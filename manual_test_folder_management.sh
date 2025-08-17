#!/bin/bash

echo "🧪 Manual Testing of Project Folder Management System"
echo "====================================================="

# Test environment verification
echo
echo "📁 Test Environment Verification:"
echo "✓ Base Path: /Volumes/base/mms/DevTest"

if [ -d "/Volumes/base/mms/DevTest/01 RFPs" ]; then
    echo "✓ RFP folder exists"
else
    echo "❌ RFP folder missing"
    exit 1
fi

if [ -d "/Volumes/base/mms/DevTest/11 Current" ]; then
    echo "✓ Current folder exists"
else
    echo "❌ Current folder missing"
    exit 1
fi

if [ -d "/Volumes/base/mms/DevTest/11 Current/00 Additional Folders" ]; then
    echo "✓ Template folders exist"
else
    echo "❌ Template folders missing"
    exit 1
fi

echo
echo "📋 Current RFP Projects:"
ls -1 "/Volumes/base/mms/DevTest/01 RFPs" | grep "^25-" || echo "No 25-* projects found"

echo
echo "📋 Current Active Projects:"
ls -1 "/Volumes/base/mms/DevTest/11 Current" | grep "^25-" || echo "No 25-* projects found"

echo
echo "🎯 Testing Scenario 1: Move project 25-97199 from RFP to Current"
echo "Before move:"
if [ -d "/Volumes/base/mms/DevTest/01 RFPs/25-97199 Test Project" ]; then
    echo "✓ Project found in RFP folder"
    
    # Simulate the move
    echo "📦 Simulating folder move..."
    mv "/Volumes/base/mms/DevTest/01 RFPs/25-97199 Test Project" "/Volumes/base/mms/DevTest/11 Current/"
    
    if [ -d "/Volumes/base/mms/DevTest/11 Current/25-97199 Test Project" ]; then
        echo "✅ Project successfully moved to Current folder"
        
        # Test template copying
        echo "📂 Testing template copying..."
        cp -r "/Volumes/base/mms/DevTest/11 Current/00 Additional Folders"/* "/Volumes/base/mms/DevTest/11 Current/25-97199 Test Project/"
        
        echo "✅ Template folders copied:"
        ls -1 "/Volumes/base/mms/DevTest/11 Current/25-97199 Test Project" | grep -E "^0[3-9]|^1[1-9]|^9[8-9]"
        
    else
        echo "❌ Project move failed"
    fi
else
    echo "⚠️  Project not found in RFP folder (may already be moved)"
    if [ -d "/Volumes/base/mms/DevTest/11 Current/25-97199 Test Project" ]; then
        echo "✓ Project found in Current folder"
    fi
fi

echo
echo "🎯 Testing Scenario 2: Verify folder structure after award"
if [ -d "/Volumes/base/mms/DevTest/11 Current/25-97199 Test Project" ]; then
    echo "📁 Project folder structure:"
    ls -la "/Volumes/base/mms/DevTest/11 Current/25-97199 Test Project" | grep "^d" | awk '{print $9}' | grep -v "^\.$\|^\.\.$" | sort
    
    echo
    echo "✅ Expected template folders:"
    echo "   01 Client Info (original)"
    echo "   02 Proposal (original)"
    echo "   03 Contract (new from template)"
    echo "   04 Deliverables (new from template)"
    echo "   05 Submittals (new from template)"
    echo "   11 SubContractors (new from template)"
    echo "   98 Outgoing (new from template)"
    echo "   99 Temp (new from template)"
fi

echo
echo "🎯 Testing Scenario 3: Test move to Archive"
if [ -d "/Volumes/base/mms/DevTest/11 Current/25-97199 Test Project" ]; then
    echo "📦 Moving project to Completed (Archive)..."
    mv "/Volumes/base/mms/DevTest/11 Current/25-97199 Test Project" "/Volumes/base/mms/DevTest/99 Completed/"
    
    if [ -d "/Volumes/base/mms/DevTest/99 Completed/25-97199 Test Project" ]; then
        echo "✅ Project successfully archived"
    else
        echo "❌ Archive move failed"
    fi
fi

echo
echo "🎯 Final Status Check:"
echo "📋 RFP Projects:"
ls -1 "/Volumes/base/mms/DevTest/01 RFPs" | grep "^25-" || echo "   (none)"

echo "📋 Current Projects:"
ls -1 "/Volumes/base/mms/DevTest/11 Current" | grep "^25-" || echo "   (none)"

echo "📋 Completed Projects:"
ls -1 "/Volumes/base/mms/DevTest/99 Completed" | grep "^25-" || echo "   (none)"

echo "📋 Inactive Projects:"
ls -1 "/Volumes/base/mms/DevTest/00 Inactive" | grep "^25-" || echo "   (none)"

echo
echo "🔄 Resetting test environment..."
# Move any test projects back to RFPs for fresh testing
if [ -d "/Volumes/base/mms/DevTest/11 Current/25-97199 Test Project" ]; then
    mv "/Volumes/base/mms/DevTest/11 Current/25-97199 Test Project" "/Volumes/base/mms/DevTest/01 RFPs/"
    echo "✓ Moved 25-97199 back to RFPs"
fi

if [ -d "/Volumes/base/mms/DevTest/99 Completed/25-97199 Test Project" ]; then
    mv "/Volumes/base/mms/DevTest/99 Completed/25-97199 Test Project" "/Volumes/base/mms/DevTest/01 RFPs/"
    echo "✓ Moved 25-97199 back to RFPs from Completed"
fi

if [ -d "/Volumes/base/mms/DevTest/00 Inactive/25-97199 Test Project" ]; then
    mv "/Volumes/base/mms/DevTest/00 Inactive/25-97199 Test Project" "/Volumes/base/mms/DevTest/01 RFPs/"
    echo "✓ Moved 25-97199 back to RFPs from Inactive"
fi

# Clean any copied template folders
if [ -d "/Volumes/base/mms/DevTest/01 RFPs/25-97199 Test Project/03 Contract" ]; then
    rm -rf "/Volumes/base/mms/DevTest/01 RFPs/25-97199 Test Project"/0[3-9]* 
    rm -rf "/Volumes/base/mms/DevTest/01 RFPs/25-97199 Test Project"/1[1-9]*
    rm -rf "/Volumes/base/mms/DevTest/01 RFPs/25-97199 Test Project"/9[8-9]*
    echo "✓ Cleaned template folders from test project"
fi

echo
echo "🎉 Folder Management Testing Complete!"
echo "📖 Next: Test through the application UI"