use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeProposal {
    pub id: Option<String>,
    pub name: String,
    pub number: String,
    pub project_id: String,
    pub company_id: String,
    pub contact_id: String,
    pub status: String,
    pub stage: String,
    pub issue_date: String,
    pub activity: Option<String>,
    pub package: Option<String>,
    pub strap_line: Option<String>,
    pub staff_name: Option<String>,
    pub staff_email: Option<String>,
    pub staff_phone: Option<String>,
    pub staff_position: Option<String>,
    pub rev: Option<i32>,
    #[serde(default)]
    pub revisions: Vec<serde_json::Value>,
    pub time: Option<serde_json::Value>,
}

fn main() {
    let test_json = r#"{
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
        "revisions": [
            {}
        ],
        "staff_email": "martin@emittiv.com",
        "staff_name": "Martin Robert",
        "staff_phone": "+971 5858 555 69",
        "staff_position": "Founder and Lighting Director",
        "stage": "Lost",
        "status": "Lost",
        "strap_line": "sensory design studio",
        "time": {}
    }"#;

    match serde_json::from_str::<FeeProposal>(test_json) {
        Ok(fee_proposal) => {
            println!("✅ Deserialization successful!");
            println!("Fee Proposal: {:?}", fee_proposal);
        }
        Err(e) => {
            println!("❌ Deserialization failed: {}", e);
        }
    }
}