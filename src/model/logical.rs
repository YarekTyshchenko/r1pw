#[derive(Debug)]
pub struct Cache {
    pub items: Vec<Item>,
}

#[derive(Debug)]
pub enum Fields {
    Redacted(Vec<RedactedField>),
    Full(Vec<FullField>),
    Missing(),
}


#[derive(Debug)]
pub struct FullField {
    pub name: String,
    pub designation: String,
    pub value: String,
}

#[derive(Debug)]
pub struct RedactedField {
    pub name: String,
    pub designation: String,
    pub value_length: usize,
}

#[derive(Debug)]
pub struct Item {
    pub account: usize,
    pub uuid: String,
    pub name: String,
    pub url: Option<String>,
    pub tags: Vec<String>,
    pub fields: Fields,
}

#[derive(Debug)]
pub enum Token {
    Stale(String),
    Fresh(String),
}

impl From<&Token> for String {
    fn from(t: &Token) -> Self {
        match t {
            Token::Stale(t) => t.to_owned(),
            Token::Fresh(t) => t.to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct Account {
    pub token: Token,
    pub shorthand: String,
    pub email: String,
    pub uuid: String,
}
