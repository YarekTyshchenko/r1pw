use std::process::{Command, Stdio};
use std::io::{Write, Read};

// Main flow
fn main() {
    // show previous selected item, if set.
    // check token exists
    // if not, attempt login
    let token = attempt_login();
    println!("Token: {}", token);
    // if success, save token
    // if failed, exit
    // if cancelled, proceed

    display_item_selection();
    // save previous item selection

    display_credential_selection();
    // copy into paste buffer
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
fn attempt_login() -> String {
    // Spawn signing, read out pipe for prompt
    let mut process = Command::new("/usr/local/bin/op")
        .arg("signin")
        .arg("--output=raw")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn().unwrap();
    let mut stdin = process.stdin.take().unwrap();
    let pw = prompt_dmenu("Unlock:");

    // Feed password to stdin of op
    stdin.write_all(format!("{}\n", pw).as_bytes()).unwrap();
    println!("Waiting for process to finish");
    let output = process.wait_with_output().unwrap();
    println!("Done waiting.");
    // read token from stdout
    let token = String::from_utf8_lossy(&output.stdout).into_owned();
    println!("Token is : {}", token);
    token
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
