use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeProposal {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub number: String,
    pub project_id: String,
    pub company_id: String,
    pub contact_id: String,
    pub status: String,
    pub stage: String,
    pub issue_date: String, // YYMMDD format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strap_line: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staff_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staff_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staff_phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staff_position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<i32>, // auto-computed
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revisions: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<serde_json::Value>,
}

fn main() {
    // Test data that matches what we see in the database
    let test_json = r#"
    {
        "activity": "Technical Systems Design and Consultancy",
        "company_id": "company:SLG",
        "contact_id": "contacts:f0itcziq0buolzzdzkkf",
        "id": "fee:⟨22_96601_1⟩",
        "issue_date": "220830",
        "name": "Fee Proposal",
        "number": "22-96601-FP",
        "package": "Lighting, Video, Sound, SFX, Show Control, IPTV, Mechanical Effects",
        "project_id": "projects:⟨22_96601⟩",
        "rev": 1,
        "revisions": [{}],
        "staff_email": "martin@emittiv.com",
        "staff_name": "Martin Robert",
        "staff_phone": "+971 5858 555 69",
        "staff_position": "Founder and Lighting Director",
        "stage": "Lost",
        "status": "Lost",
        "strap_line": "sensory design studio",
        "time": {}
    }
    "#;
    
    println!("Testing deserialization...");
    
    // Parse the JSON first
    let mut json_value: serde_json::Value = serde_json::from_str(test_json).unwrap();
    
    // Apply the same transformations as in the Rust code
    if let Some(obj) = json_value.as_object_mut() {
        // Handle empty time object
        if let Some(time_value) = obj.get("time") {
            if time_value.is_object() && time_value.as_object().unwrap().is_empty() {
                obj.remove("time");
            }
        }
        
        // Handle empty revisions array with empty objects
        if let Some(revisions_value) = obj.get("revisions") {
            if let Some(revisions_array) = revisions_value.as_array() {
                if revisions_array.len() == 1 && revisions_array[0].is_object() && revisions_array[0].as_object().unwrap().is_empty() {
                    obj.insert("revisions".to_string(), serde_json::Value::Array(vec![]));
                }
            }
        }
    }
    
    println!("Modified JSON: {}", serde_json::to_string_pretty(&json_value).unwrap());
    
    // Try to deserialize
    match serde_json::from_value::<FeeProposal>(json_value) {
        Ok(proposal) => {
            println!("✅ Deserialization successful!");
            println!("Proposal: {} - {}", proposal.number, proposal.name);
        }
        Err(e) => {
            println!("❌ Deserialization failed: {}", e);
        }
    }
}