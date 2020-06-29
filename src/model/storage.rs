use serde::{ Serialize, Deserialize};

// Storage model
#[derive(Debug, Serialize, Deserialize)]
pub struct Cache {
    pub accounts: Vec<Account>,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct Account {
    pub token: Option<String>,
    pub shorthand: String,
    pub email: String,
    pub uuid: String,
    pub items: Vec<Item>
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct Item {
    pub uuid: String,
    pub name: String,
    pub url: Option<String>,
    pub fields: Vec<Field>,
    pub tags: Vec<String>,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub designation: String,
    pub value_length: usize,
}
