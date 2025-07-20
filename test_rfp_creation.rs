use serde_json::json;
use surrealdb::sql::Thing;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test different Thing formats to find the correct one
    
    // Format 1: String format
    let thing_string = "projects:25_97107";
    println!("Format 1 (string): {}", thing_string);
    
    // Format 2: Object with tb and id as strings
    let thing_object = json!({
        "tb": "projects",
        "id": "25_97107"
    });
    println!("Format 2 (object): {}", thing_object);
    
    // Format 3: SurrealDB Thing construction
    let thing_surreal = Thing::from(("projects", "25_97107"));
    println!("Format 3 (SurrealDB Thing): {}", serde_json::to_string(&thing_surreal)?);
    
    // Format 4: Test what the Thing actually serializes to
    let serialized = serde_json::to_value(&thing_surreal)?;
    println!("Format 4 (Thing serialized): {}", serialized);
    
    // Test RFP creation with the correct format
    let test_rfp = json!({
        "name": "DELETE ME - Test RFP",
        "number": "25-97107-FP-DELETE-ME",
        "rev": 1,
        "status": "Draft",
        "stage": "Draft", 
        "issue_date": "250720",
        "activity": "DELETE ME Testing",
        "package": "DELETE ME Package",
        "project_id": thing_surreal,
        "company_id": Thing::from(("company", "EMITTIV")),
        "contact_id": Thing::from(("contacts", "test_contact")),
        "staff_name": "DELETE ME Test Staff",
        "staff_email": "deleteme@test.com",
        "staff_phone": "+000 0000 0000",
        "staff_position": "DELETE ME Position",
        "strap_line": "DELETE ME strap line",
        "revisions": json!([]),
        "time": {
            "created_at": "2025-07-20T12:00:00Z",
            "updated_at": "2025-07-20T12:00:00Z"
        }
    });
    
    println!("Test RFP JSON: {}", serde_json::to_string_pretty(&test_rfp)?);
    
    Ok(())
}