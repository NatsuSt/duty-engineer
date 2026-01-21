use serde::{Deserialize, Serialize};


/// Represent engineer data to be stored and displayed
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Engineer {
    pub first_name: String,
    pub last_name: String,
    pub phone_number: String,
}

impl Default for Engineer {
    fn default() -> Self {
        Self {
            first_name: "Tom".to_string(),
            last_name: "Hanks".to_string(),
            phone_number: "+38(012)-345-67-89".to_string()
        }
    }
}

