struct Cache {
    accounts: Vec<Account>,
}

struct Account {
    token: Option<String>,
    shorthand: String,
    uuid: String,
    items: Vec<Item>
}

struct Item {
    uuid: String,
    name: String,
    url: String,
    fields: Vec<Field>,
    tags: Vec<String>,
}

struct Field {
    name: String,
    designation: String,
    value: String,
}
