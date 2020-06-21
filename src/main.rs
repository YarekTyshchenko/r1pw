use std::process::{Command, Stdio};
use std::io::{Write, Read};
use std::path::Path;
use std::fs::{File, OpenOptions};
use std::io;

use serde::Deserialize;

// Main flow
fn main() {
    // show previous selected item, if set.
    // check token exists
    let token_path = shellexpand::full("~/.config/1pw/token").unwrap().into_owned();
    let token_path = Path::new(&token_path);
    let token = match read_token_from_path(token_path) {
        Ok(t) => t,
        Err(e) => {
            println!("Error {}", e);
            let t = attempt_login().expect("Unable to get token");
            let t = t.trim().to_owned();
            // if success, save token
            // if failed, exit
            save_token(&t, &token_path).expect("Unable to save new token");
            t
        },
    };
    println!("Token: '{}'", token);
    // if cancelled, proceed

    // Wrap this in cache
    let items = get_items(&token).unwrap_or_else(|| {
        let t = attempt_login().unwrap().trim().to_owned();
        save_token(&t, &token_path);
        get_items(&t).unwrap()
    });
    let selection = display_item_selection(&items);
    // save previous item selection

    let credential = get_credentials(selection, &token);
    let field = display_credential_selection(&credential);
    // copy into paste buffer
    copy_to_clipboard(field);
}

fn copy_to_clipboard(field: &Field) {
    println!("Chosen field is: {}, {}, {}", field.name, field.designation, field.value);
    let mut copy = Command::new("xsel")
        .arg("-b")
        .stdin(Stdio::piped())
        .spawn().unwrap();
    let mut stdin = copy.stdin.take().unwrap();
    stdin.write_all(field.value.as_bytes()).unwrap();
    drop(stdin);
    copy.wait().unwrap();
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    uuid: String,
    overview: Overview,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Overview {
    title: String,
    url: Option<String>,
    tags: Option<Vec<String>>,
}

fn get_items(token: &str) -> Option<Vec<Item>> {
    let op = Command::new("op")
        .arg("list").arg("items")
        .arg("--session").arg(token)
        .output().unwrap();
    println!("status: {}", op.status);
    if ! op.status.success() {
        return None
    }
    println!("stderr: {}", String::from_utf8(op.stderr).unwrap());
    let items = String::from_utf8(op.stdout).unwrap();
    println!("items: {}", items);
    let items: Vec<Item> = serde_json::from_str(&items).unwrap();
    println!("Items: {:?}", items);
    Some(items)
}

fn read_token_from_path(path: &Path) -> io::Result<String> {
    let mut f = File::open(path)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}

fn save_token(token: &str, token_path: &Path) -> Option<()> {
    println!("Attempting to save token {} to path {}", token, token_path.to_str().unwrap());
    std::fs::create_dir_all(token_path.parent().unwrap()).unwrap();
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(false)
        .open(token_path)
        .unwrap();
    file.write_all(token.as_bytes()).unwrap();
    Some(())
}

fn display_item_selection(items: &Vec<Item>) -> &Item {
    // pipe to dmenu, and listen for choice
    let mut dmenu = Command::new("dmenu")
        .arg("-b")
        .arg("-l")
        .arg("40")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn().unwrap();
    // Feed list to dmenu on stdin
    let mut input: Vec<String> = Vec::new();
    let mut stdin = dmenu.stdin.take().unwrap();
    for item in items {
        //input.push(format!("{} (uuid: {})", item.overview.title, item.uuid));
        input.push(item.overview.title.to_owned());
    }

    stdin.write_all(input.join("\n").as_bytes()).unwrap();
    stdin.flush().unwrap();
    drop(stdin);
    // Find choice in list
    let output = dmenu.wait_with_output().ok().unwrap();
    let choice = String::from_utf8_lossy(&output.stdout);
    let choice = choice.trim();
    println!("dmenu status: {}", output.status);
    println!("Choice: {}", choice);
    // return item
    let foo = items.iter().find(|&i| i.overview.title == choice).unwrap();
    foo
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Credential {
    uuid: String,
    details: Details,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Details {
    fields: Vec<Field>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Field {
    designation: String,
    name: String,
    value: String,
}

fn get_credentials(selection: &Item, token: &str) -> Credential {
    // Query op for title / uuid of the item
    let op = Command::new("op")
        .arg("get").arg("item").arg(&selection.uuid)
        .arg("--session").arg(token)
        .output().unwrap();
    // get a list of credentials
    let output = String::from_utf8_lossy(&op.stdout);
    let output = output.trim();
    println!("Creds: {}", output);
    let credential: Credential = serde_json::from_str(output).unwrap();
    // Optionally top up with totp
    credential
}

fn format_field(field: &Field) -> String {
    format!("{} ({}) Field name: {}", field.designation, field.value, field.name)
}

fn display_credential_selection(credential: &Credential) -> &Field {
    let mut dmenu = Command::new("dmenu")
        .arg("-b").arg("-l").arg("20")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn().unwrap();
    // prepare a list of credentials to copy for dmenu
    let mut input: Vec<String> = Vec::new();
    for field in &credential.details.fields {
        input.push(format_field(field));
    }
    // pipe to dmenu, and listen for choice
    let mut stdin = dmenu.stdin.take().unwrap();
    stdin.write_all(input.join("\n").as_bytes()).unwrap();
    drop(stdin);

    let output = dmenu.wait_with_output().unwrap();
    let choice = String::from_utf8_lossy(&output.stdout);
    let choice = choice.trim();
    // find choice in list
    let foo = credential.details.fields.iter().find(|&f| format_field(f) == choice).unwrap();
    // return item
    foo
}

// @TODO: op tool is silent when no tty is present
fn attempt_login() -> Option<String> {
    // Spawn signing, read out pipe for prompt
    let mut process = Command::new(
        "/usr/local/bin/op"
        //"./mock.sh"
    )
        .arg("signin")
        .arg("--output=raw")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn().ok()?;
    let mut stdin = process.stdin.take()?;
    let pw = prompt_dmenu("Unlock:");

    // Feed password to stdin of op
    stdin.write_all(format!("{}\n", pw).as_bytes()).ok()?;
    println!("Waiting for process to finish");
    let output = process.wait_with_output().ok()?;
    println!("Done waiting.");
    // read token from stdout
    let token = String::from_utf8_lossy(&output.stdout).into_owned();
    println!("Token is : {}", token);
    Some(token)
}

fn prompt_dmenu(prompt: &str) -> String {
    let dmenu = Command::new("dmenu")
        .arg("-b")
        .arg("-p").arg(prompt)
        .arg("-nb").arg("black")
        .arg("-nf").arg("black")
        .output().unwrap();
    println!("status: {}", dmenu.status);
    let pw = String::from_utf8(dmenu.stdout).unwrap();
    pw
}
