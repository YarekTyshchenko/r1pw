use std::process::{Command, Stdio};
use std::io::Write;
use log::*;
use serde::{ Serialize, Deserialize};
use anyhow::{Result, Error, Context};
use itertools::Itertools;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub uuid: String,
    pub overview: Overview,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    pub title: String,
    pub url: Option<String>,
    pub tags: Option<Vec<String>>,
}

pub fn get_items(account_uuid: &str, token: &str) -> Result<Vec<Item>> {
    let items = op("", vec!["list", "items", "--account", account_uuid, "--session", token])?;
    // Deserialisation issues should panic
    let items: Vec<Item> = serde_json::from_str(&items)
        .with_context(||"Failed to de-serialise JSON item list")?;
    Ok(items)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Credential {
    pub uuid: String,
    pub details: Details,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Details {
    pub fields: Option<Vec<Field>>,
    pub password: Option<String>,
}
impl Details {
    pub fn get_fields(self) -> Vec<Field> {
        self.fields.unwrap_or(self.password.into_iter().map(|password| Field {
            value: password,
            designation: "password".to_string(),
            name: "password".to_string(),
        }).collect_vec())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub designation: String,
    pub name: String,
    pub value: String,
}

pub fn get_credentials(item_uuid: &str, token: &str) -> Result<Credential> {
    // Query op for title / uuid of the item
    let output = op("", ["get", "item", &item_uuid, "--session", token].to_vec())?;
    //debug!("Creds: {}", output);
    let credential: Credential = serde_json::from_str(&output)
        .with_context(||format!("Error de-serialising Credential fields from JSON: {}", &output))?;
    // Optionally top up with totp
    Ok(credential)
}

pub fn login(shorthand: &str, unlock: &str) -> Result<String> {
    let token = op(&format!("{}\n", unlock), vec!["signin", shorthand, "--output=raw"])?;
    let token = token.trim().to_owned();
    Ok(token)
}

pub fn op(input: &str, args: Vec<&str>) -> Result<String> {
    // Spawn signing, read out pipe for prompt
    let mut process = Command::new(
        "op"
        //"./mock.sh"
    );
    process.args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    debug!("Command {:?}", process);
    let mut process = process.spawn()?;
    // Stdin must always exist
    let mut stdin = process.stdin.take().unwrap();
    // Feed to stdin of op
    stdin.write_all(input.as_bytes())?;
    drop(stdin);
    debug!("Waiting for process to finish");
    let output = process.wait_with_output()?;
    if ! output.status.success() {
        return Err(Error::msg(format!(
            "{}: {}",
            match output.status.code() {
                None => "op command terminated by signal".to_owned(),
                Some(c) => format!("op command failed with exit code {}", c),
            }, String::from_utf8_lossy(&output.stderr).trim())
        ));
    }
    debug!("Done waiting.");
    // read from stdout
    let output = String::from_utf8_lossy(&output.stdout).into_owned();
    Ok(output)
}
