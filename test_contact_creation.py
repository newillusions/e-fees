#!/usr/bin/env python3
"""
Test script to create Irene B Nale contact using the fixed Tauri backend.
This simulates the exact contact data provided by the user.
"""

import asyncio
import websockets
import json

async def create_irene_contact():
    """Create the specific contact requested by the user"""
    uri = 'ws://10.0.1.17:8000/rpc'
    
    contact_data = {
        'first_name': 'Irene',
        'last_name': 'B Nale', 
        'email': 'i.nale@imtiaz.ae',
        'phone': '+971 4 430 3703',
        'position': 'Design Administration Coordinator',
        'company': 'IMTZ'  # Imtiaz Developments company ID
    }
    
    try:
        async with websockets.connect(uri) as websocket:
            print("🔌 Connected to SurrealDB")
            
            # Use namespace and database
            use_msg = {
                'id': 1,
                'method': 'use',
                'params': ['emittiv', 'projects']
            }
            await websocket.send(json.dumps(use_msg))
            response = await websocket.recv()
            print(f"📋 Database selection: {response}")
            
            # Check if Imtiaz company exists
            check_company_msg = {
                'id': 2,
                'method': 'query',
                'params': ['SELECT * FROM company:IMTZ;']
            }
            await websocket.send(json.dumps(check_company_msg))
            company_response = await websocket.recv()
            print(f"🏢 Company check: {company_response}")
            
            # Create the contact using the EXACT format the fixed backend will use
            # This matches the updated create_contact function in mod.rs
            create_sql = f"""
            CREATE contacts:irene_b_nale_imtiaz SET 
                first_name = '{contact_data['first_name']}',
                last_name = '{contact_data['last_name']}',
                full_name = string::concat('{contact_data['first_name']}', ' ', '{contact_data['last_name']}'),
                email = '{contact_data['email']}',
                phone = '{contact_data['phone']}',
                position = '{contact_data['position']}',
                company = company:{contact_data['company']},
                time = {{ created_at: time::now(), updated_at: time::now() }};
            """
            
            query_msg = {
                'id': 3,
                'method': 'query',
                'params': [create_sql]
            }
            await websocket.send(json.dumps(query_msg))
            response = await websocket.recv()
            print(f"👤 Contact creation result: {response}")
            
            # Verify the contact was created successfully
            verify_msg = {
                'id': 4,
                'method': 'query',
                'params': ['SELECT count() FROM contacts WHERE email = "i.nale@imtiaz.ae";']
            }
            await websocket.send(json.dumps(verify_msg))
            verify_response = await websocket.recv()
            print(f"✅ Verification result: {verify_response}")
            
            # Show all contacts count for reference
            count_msg = {
                'id': 5,
                'method': 'query',
                'params': ['SELECT count() FROM contacts GROUP ALL;']
            }
            await websocket.send(json.dumps(count_msg))
            count_response = await websocket.recv()
            print(f"📊 Total contacts after creation: {count_response}")
            
    except Exception as e:
        print(f"❌ Error: {e}")

if __name__ == "__main__":
    print("🚀 Creating Irene B Nale contact using fixed backend logic...")
    print("📋 Contact details:")
    print("   • Name: Irene B Nale")
    print("   • Position: Design Administration Coordinator") 
    print("   • Company: Imtiaz Developments (IMTZ)")
    print("   • Phone: +971 4 430 3703")
    print("   • Email: i.nale@imtiaz.ae")
    print()
    
    asyncio.run(create_irene_contact())