# Test Data Creation Scripts

## Overview
This file contains safe test data that can be created and easily identified for deletion.
All test records include "DELETE ME" markers for easy cleanup.

---

## Test Contact Creation

### Via Python WebSocket Client

```python
import asyncio
import websockets
import json

async def create_test_contact():
    uri = 'ws://10.0.1.17:8000/rpc'
    
    try:
        async with websockets.connect(uri) as websocket:
            # Use namespace and database
            use_msg = {
                'id': 1,
                'method': 'use',
                'params': ['emittiv', 'projects']
            }
            await websocket.send(json.dumps(use_msg))
            response = await websocket.recv()
            print('Use response:', response)
            
            # Create test contact (FIXED - no manual full_name)
            create_sql = '''
            CREATE contacts:DELETE_ME_TEST_001 SET 
                first_name = 'DELETE ME',
                last_name = 'Test Contact',
                email = 'delete.me.test@example.com',
                phone = '+971 50 000 0001',
                position = 'DELETE ME Test Position',
                company = company:IMTZ,
                time = { created_at: time::now(), updated_at: time::now() };
            '''
            
            query_msg = {
                'id': 2,
                'method': 'query',
                'params': [create_sql]
            }
            await websocket.send(json.dumps(query_msg))
            response = await websocket.recv()
            print('Contact creation response:', response)
            
            # Verify creation
            verify_msg = {
                'id': 3,
                'method': 'query',
                'params': ['SELECT * FROM contacts:DELETE_ME_TEST_001;']
            }
            await websocket.send(json.dumps(verify_msg))
            response = await websocket.recv()
            print('Verification response:', response)
                
    except Exception as e:
        print('Error:', e)

# Run the test
asyncio.run(create_test_contact())
```

### Via Tauri Backend (Recommended)

```rust
// Test data for ContactCreate struct
let test_contact = ContactCreate {
    first_name: "DELETE ME".to_string(),
    last_name: "Test Contact".to_string(),
    email: "delete.me.backend@example.com".to_string(),
    phone: "+971 50 000 0002".to_string(),
    position: "DELETE ME Backend Test".to_string(),
    company: "IMTZ".to_string(),
};
```

---

## Test Company Creation

```python
async def create_test_company():
    uri = 'ws://10.0.1.17:8000/rpc'
    
    try:
        async with websockets.connect(uri) as websocket:
            # Use namespace and database
            use_msg = {
                'id': 1,
                'method': 'use',
                'params': ['emittiv', 'projects']
            }
            await websocket.send(json.dumps(use_msg))
            await websocket.recv()
            
            # Create test company
            create_sql = '''
            CREATE company:DELETE_ME_TEST SET 
                name = 'DELETE ME Test Company Ltd',
                name_short = 'DELETE ME Test',
                abbreviation = 'DELTEST',
                city = 'Test City',
                country = 'Test Country',
                reg_no = 'DELETE-ME-REG-001',
                tax_no = 'DELETE-ME-TAX-001',
                time = { created_at: time::now(), updated_at: time::now() };
            '''
            
            query_msg = {
                'id': 2,
                'method': 'query',
                'params': [create_sql]
            }
            await websocket.send(json.dumps(query_msg))
            response = await websocket.recv()
            print('Company creation response:', response)
                
    except Exception as e:
        print('Error:', e)

asyncio.run(create_test_company())
```

---

## Test Project Creation

```python
async def create_test_project():
    uri = 'ws://10.0.1.17:8000/rpc'
    
    try:
        async with websockets.connect(uri) as websocket:
            # Use namespace and database
            use_msg = {
                'id': 1,
                'method': 'use',
                'params': ['emittiv', 'projects']
            }
            await websocket.send(json.dumps(use_msg))
            await websocket.recv()
            
            # Create test project
            create_sql = '''
            CREATE projects:DELETE_ME_TEST_001 SET 
                name = 'DELETE ME Test Project',
                name_short = 'DELETE ME Test',
                status = 'Draft',
                area = 'DELETE ME Test Area',
                city = 'DELETE ME City',
                country = 'DELETE ME Country',
                folder = 'DELETE_ME_TEST_FOLDER',
                number = {
                    year: 25,
                    country: 000,
                    seq: 999,
                    id: '25-000999'
                },
                time = { created_at: time::now(), updated_at: time::now() };
            '''
            
            query_msg = {
                'id': 2,
                'method': 'query',
                'params': [create_sql]
            }
            await websocket.send(json.dumps(query_msg))
            response = await websocket.recv()
            print('Project creation response:', response)
                
    except Exception as e:
        print('Error:', e)

asyncio.run(create_test_project())
```

---

## Test RFP/Fee Proposal Creation

```python
async def create_test_rfp():
    uri = 'ws://10.0.1.17:8000/rpc'
    
    try:
        async with websockets.connect(uri) as websocket:
            # Use namespace and database
            use_msg = {
                'id': 1,
                'method': 'use',
                'params': ['emittiv', 'projects']
            }
            await websocket.send(json.dumps(use_msg))
            await websocket.recv()
            
            # Create test RFP
            create_sql = '''
            CREATE rfp:DELETE_ME_TEST_001 SET 
                name = 'DELETE ME Test RFP',
                number = 'DELETE-ME-RFP-001',
                status = 'Draft',
                stage = 'Draft',
                issue_date = '2025-07-19',
                activity = 'DELETE ME Test Activity',
                package = 'DELETE ME Test Package',
                project_id = projects:DELETE_ME_TEST_001,
                company_id = company:DELETE_ME_TEST,
                contact_id = contacts:DELETE_ME_TEST_001,
                staff_name = 'DELETE ME Test Staff',
                staff_email = 'delete.me.staff@example.com',
                staff_phone = '+971 50 000 0003',
                staff_position = 'DELETE ME Test Position',
                strap_line = 'DELETE ME Test Strap Line',
                rev = 1,
                revisions = [],
                time = { created_at: time::now(), updated_at: time::now() };
            '''
            
            query_msg = {
                'id': 2,
                'method': 'query',
                'params': [create_sql]
            }
            await websocket.send(json.dumps(query_msg))
            response = await websocket.recv()
            print('RFP creation response:', response)
                
    except Exception as e:
        print('Error:', e)

asyncio.run(create_test_rfp())
```

---

## Cleanup Scripts

### Delete All Test Data

```python
async def cleanup_test_data():
    uri = 'ws://10.0.1.17:8000/rpc'
    
    try:
        async with websockets.connect(uri) as websocket:
            # Use namespace and database
            use_msg = {
                'id': 1,
                'method': 'use',
                'params': ['emittiv', 'projects']
            }
            await websocket.send(json.dumps(use_msg))
            await websocket.recv()
            
            # Delete test contacts
            cleanup_queries = [
                "DELETE contacts WHERE id CONTAINS 'DELETE_ME' OR first_name CONTAINS 'DELETE ME';",
                "DELETE company WHERE id CONTAINS 'DELETE_ME' OR name CONTAINS 'DELETE ME';",
                "DELETE projects WHERE id CONTAINS 'DELETE_ME' OR name CONTAINS 'DELETE ME';",
                "DELETE rfp WHERE id CONTAINS 'DELETE_ME' OR name CONTAINS 'DELETE ME';"
            ]
            
            for i, query in enumerate(cleanup_queries):
                query_msg = {
                    'id': i + 2,
                    'method': 'query',
                    'params': [query]
                }
                await websocket.send(json.dumps(query_msg))
                response = await websocket.recv()
                print(f'Cleanup {i+1} response:', response)
                
    except Exception as e:
        print('Error:', e)

asyncio.run(cleanup_test_data())
```

### Verify Cleanup

```python
async def verify_cleanup():
    uri = 'ws://10.0.1.17:8000/rpc'
    
    try:
        async with websockets.connect(uri) as websocket:
            # Use namespace and database
            use_msg = {
                'id': 1,
                'method': 'use',
                'params': ['emittiv', 'projects']
            }
            await websocket.send(json.dumps(use_msg))
            await websocket.recv()
            
            # Check for remaining test data
            verify_queries = [
                "SELECT count() FROM contacts WHERE first_name CONTAINS 'DELETE ME' GROUP ALL;",
                "SELECT count() FROM company WHERE name CONTAINS 'DELETE ME' GROUP ALL;",
                "SELECT count() FROM projects WHERE name CONTAINS 'DELETE ME' GROUP ALL;",
                "SELECT count() FROM rfp WHERE name CONTAINS 'DELETE ME' GROUP ALL;"
            ]
            
            for i, query in enumerate(verify_queries):
                query_msg = {
                    'id': i + 2,
                    'method': 'query',
                    'params': [query]
                }
                await websocket.send(json.dumps(query_msg))
                response = await websocket.recv()
                print(f'Verification {i+1} response:', response)
                
    except Exception as e:
        print('Error:', e)

asyncio.run(verify_cleanup())
```

---

## Safe Testing Patterns

### 1. Always Use DELETE ME Markers
- Include "DELETE ME" in names, positions, or other text fields
- Use test email domains: `@example.com`, `@test.com`
- Use obviously fake phone numbers: `+971 50 000 XXXX`

### 2. Use Test ID Patterns
- Use IDs like: `DELETE_ME_TEST_001`, `DELETE_ME_TEST_002`
- Include test sequence numbers for uniqueness

### 3. Verify Before Creating
- Always check if test data already exists
- Clean up previous test data before creating new ones

### 4. Test in Order
1. Create test company first
2. Create test project (if needed)
3. Create test contact referencing the company
4. Create test RFP referencing all three

### 5. Always Clean Up
- Run cleanup scripts after testing
- Verify cleanup completed successfully
- Document any issues encountered

---

**Last Updated**: July 19, 2025  
**Purpose**: Safe testing of all database operations  
**Author**: Claude Code Assistant