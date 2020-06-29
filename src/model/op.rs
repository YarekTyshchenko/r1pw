use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OpConfig {
    pub latest_signin: String,
    pub accounts: Vec<OpAccount>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpAccount {
    pub shorthand: String,
    pub url: String,
    pub email: String,
    pub userUUID: String,
}
