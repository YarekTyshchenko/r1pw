use std::process::{Command, Stdio};
use std::io::{Write, Read};
use std::path::Path;
use std::fs::{File, OpenOptions};
use std::io;
use log::{error, debug};
use serde::Deserialize;

const TOKEN_PATH: &str = "~/.config/1pw/token";
// Main flow
fn main() {
    pretty_env_logger::init();
    // show previous selected item, if set.
    // check token exists
    let token = match read_token_from_path() {
        Ok(t) => t,
        Err(e) => {
            debug!("Error {}", e);
            let t = attempt_login().expect("Unable to get token");
            let t = t.trim().to_owned();
            // if success, save token
            // if failed, exit
            save_token(&t).expect("Unable to save new token");
            t
        },
    };
    debug!("Token: '{}'", token);
    // if cancelled, proceed

    // Wrap this in cache
    let items = get_items(&token).unwrap_or_else(|| {
        let t = attempt_login().unwrap().trim().to_owned();
        save_token(&t);
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
    debug!("Chosen field is: {}, {}, {}", field.name, field.designation, field.value);
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
    let items = op("", ["list", "items", "--session", token].to_vec())?;
    let items: Vec<Item> = serde_json::from_str(&items).ok()?;
    Some(items)
}

fn read_token_from_path() -> io::Result<String> {
    let token_path = shellexpand::full(TOKEN_PATH).unwrap().into_owned();
    let token_path = Path::new(&token_path);
    let mut f = File::open(token_path)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}

fn save_token(token: &str) -> Option<()> {
    let token_path = shellexpand::full(TOKEN_PATH).unwrap().into_owned();
    let token_path = Path::new(&token_path);

    debug!("Attempting to save token {} to path {}", token, token_path.to_str().unwrap());
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
    // Feed list to dmenu on stdin
    let mut input: Vec<String> = Vec::new();
    for item in items {
        //input.push(format!("{} (uuid: {})", item.overview.title, item.uuid));
        input.push(item.overview.title.to_owned());
    }

    let choice = select_dmenu(&input.join("\n"));
    // Find choice in list
    debug!("Choice: {}", choice);
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
    let output = op("", ["get", "item", &selection.uuid, "--session", token].to_vec()).unwrap();
    //debug!("Creds: {}", output);
    let credential: Credential = serde_json::from_str(&output).unwrap();
    // Optionally top up with totp
    credential
}

fn format_field(field: &Field) -> String {
    format!("{} ({}) Field name: {}", field.designation, field.value, field.name)
}

fn display_credential_selection(credential: &Credential) -> &Field {
    let mut input: Vec<String> = Vec::new();
    for field in &credential.details.fields {
        input.push(format_field(field));
    }
    let choice = select_dmenu(&input.join("\n"));
    // find choice in list
    let foo = credential.details.fields.iter().find(|&f| format_field(f) == choice).unwrap();
    // return item
    foo
}

fn attempt_login() -> Option<String> {
    let pw = prompt_dmenu("Unlock:");
    op(&(pw+"\n"), ["signin", "--output=raw"].to_vec())
}

fn select_dmenu(input: &str) -> String {
    dmenu(input, ["-b", "-l", "20"].to_vec())
}

fn prompt_dmenu(prompt: &str) -> String {
    dmenu("", ["-b", "-p", prompt, "-nb", "black", "-nf", "black"].to_vec())
}

fn dmenu(input: &str, args: Vec<&str>) -> String {
    let mut dmenu = Command::new("dmenu")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn().unwrap();
    let mut stdin = dmenu.stdin.take().unwrap();
    stdin.write_all(input.as_bytes()).unwrap();
    drop(stdin);

    let output = dmenu.wait_with_output().unwrap();
    let choice = String::from_utf8_lossy(&output.stdout);
    let choice = choice.trim();
    choice.to_owned()
}

fn op(input: &str, args: Vec<&str>) -> Option<String> {
    // Spawn signing, read out pipe for prompt
    let mut process = Command::new(
        "/usr/local/bin/op"
        //"./mock.sh"
    )
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn().unwrap();
    let mut stdin = process.stdin.take().unwrap();
    // Feed to stdin of op
    stdin.write_all(input.as_bytes()).unwrap();
    drop(stdin);
    debug!("Waiting for process to finish");
    let output = process.wait_with_output().unwrap();
    if ! output.status.success() {
        error!("op command failed with {}", String::from_utf8_lossy(&output.stderr));
        return None
    }
    debug!("Done waiting.");
    // read from stdout
    let output = String::from_utf8_lossy(&output.stdout).into_owned();
    Some(output)
}
