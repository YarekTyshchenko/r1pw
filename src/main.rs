use std::process::{Command, Stdio, ExitStatus, exit};
use std::io::{Write, Read};
use std::path::Path;
use std::fs::{File, OpenOptions, read};
use std::io;
use log::*;
use serde::Deserialize;

const TOKEN_PATH: &str = "~/.config/1pw/token";
// Main flow
fn main() {
    pretty_env_logger::init();
    // show previous selected item, if set.
    // check token exists
    let token = read_token_from_path()
        .map(Some)
        .unwrap_or_else(|e| {
            warn!("Token not found: {}", e);
            let t = match attempt_login() {
                Ok(t) => {
                    save_token(&t).expect("Unable to save new token");
                    Some(t)
                },
                Err(LoginError::Cancelled()) => None,
                Err(e) => panic!(e),
            };
            t
        });

    debug!("Token: '{:?}'", token);
    // if cancelled, proceed

    // @TODO: Implement caching here
    let token = token.expect("Unable to proceed because cache isn't implemented");
    let items = get_items(&token).map_err(|e| {
        // Unable to get items, clearing token and exit
        clear_token().unwrap();
        e
    }).unwrap();


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

fn get_items(token: &str) -> Result<Vec<Item>, OpError> {
    let items = op("", ["list", "items", "--session", token].to_vec())?;
    // Deserialisation issues should panic
    let items: Vec<Item> = serde_json::from_str(&items).unwrap();
    Ok(items)
}


fn read_token_from_path() -> io::Result<String> {
    let token_path = shellexpand::full(TOKEN_PATH).unwrap();
    let token_path = Path::new(token_path.as_ref());
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

fn clear_token() -> io::Result<()> {
    let token_path = shellexpand::full(TOKEN_PATH).unwrap();
    let token_path = Path::new(token_path.as_ref());
    std::fs::remove_file(token_path)
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

#[derive(Debug)]
enum LoginError {
    Cancelled(),
    FailedDmenu(io::Error),
    FailedOp(OpError),
}

fn attempt_login() -> Result<String, LoginError> {
    let pw = match prompt_dmenu("Unlock:") {
        Ok(pw) => Ok(pw),
        Err(DmenuError::Cancelled()) => Err(LoginError::Cancelled()),
        Err(DmenuError::Io(e)) => Err(LoginError::FailedDmenu(e)),
    }.unwrap();
    let token = op(&(pw+"\n"), ["signin", "--output=raw"].to_vec());
    let token = match token {
        Ok(t) => t,
        Err(OpError::Io(e)) => panic!("IO Troubles: {}", e),
        Err(OpError::CommandError(code, reason)) => panic!("Op exit code {} with error: {}", code, reason),
    };
    let token = token.trim().to_owned();
    Ok(token)
}

fn select_dmenu(input: &str) -> String {
    dmenu(input, ["-b", "-l", "20"].to_vec()).unwrap()
}

fn prompt_dmenu(prompt: &str) -> Result<String, DmenuError> {
    dmenu("", ["-b", "-p", prompt, "-nb", "black", "-nf", "black"].to_vec())
}

#[derive(Debug)]
enum DmenuError {
    Cancelled(),
    Io(io::Error),
}

fn dmenu(input: &str, args: Vec<&str>) -> Result<String, DmenuError> {
    let mut dmenu = Command::new("dmenu")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn().map_err(DmenuError::Io)?;
    let mut stdin = dmenu.stdin.take().unwrap();
    stdin.write_all(input.as_bytes()).map_err(DmenuError::Io)?;
    drop(stdin);

    let output = dmenu.wait_with_output().map_err(DmenuError::Io)?;
    if ! output.status.success() {
        warn!("Dmenu process cancelled with exit code {:?}", output.status.code());
        return Err(DmenuError::Cancelled());
    }
    let choice = String::from_utf8_lossy(&output.stdout);
    let choice = choice.trim();
    Ok(choice.to_owned())
}

#[derive(Debug)]
enum OpError {
    Io(io::Error),
    CommandError(ExitStatus, String),
}

fn op(input: &str, args: Vec<&str>) -> Result<String, OpError> {
    // Spawn signing, read out pipe for prompt
    let mut process = Command::new(
        "/usr/local/bin/op"
        //"./mock.sh"
    )
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn().map_err(OpError::Io)?;
    // Stdin must always exist
    let mut stdin = process.stdin.take().unwrap();
    // Feed to stdin of op
    stdin.write_all(input.as_bytes()).map_err(OpError::Io)?;
    drop(stdin);
    debug!("Waiting for process to finish");
    let output = process.wait_with_output().map_err(OpError::Io)?;
    if ! output.status.success() {
        error!("op command failed with {}", String::from_utf8_lossy(&output.stderr));
        return Err(OpError::CommandError(output.status, "Foo".to_owned()));
    }
    debug!("Done waiting.");
    // read from stdout
    let output = String::from_utf8_lossy(&output.stdout).into_owned();
    Ok(output)
}
