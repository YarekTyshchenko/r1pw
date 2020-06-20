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
    let items = get_items(&token);

    display_item_selection();
    // save previous item selection

    display_credential_selection();
    // copy into paste buffer
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

fn get_items(token: &str) -> Vec<Item> {
    let op = Command::new("op")
        .arg("list").arg("items")
        .arg("--session").arg(token)
        .output().unwrap();
    println!("status: {}", op.status);
    println!("stderr: {}", String::from_utf8(op.stderr).unwrap());
    let items = String::from_utf8(op.stdout).unwrap();
    println!("items: {}", items);
    let items: Vec<Item> = serde_json::from_str(&items).unwrap();
    println!("Items: {:?}", items);
    items
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

fn display_item_selection() {
    // read from cache
    // pipe to dmenu, and listen for choice
    // Find choice in list
    // return item
}

fn display_credential_selection() {
    // prepare a list of credentials to copy for dmenu
    // pipe to dmenu, and listen for choice
    // find choice in list
    // return item
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
