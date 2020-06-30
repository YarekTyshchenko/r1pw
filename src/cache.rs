use anyhow::{Context, Result, Error};
use std::io;
use std::path::{Path, PathBuf};
use std::io::ErrorKind;

use super::op::{Item};
use crate::op::Field;
use crate::model::storage;
use crate::model::op::OpConfig;
use itertools::Itertools;

const OP_CONFIG_PATH: &str = "~/.op/config";
const CACHE_PATH: &str = "~/.config/r1pw/cache.json";

fn read_op_config() -> Result<OpConfig> {
    let path = shellexpand::full(OP_CONFIG_PATH)
        .with_context(||"Bad op config path")?;
    let path = Path::new(path.as_ref());

    let config = std::fs::read_to_string(&path)?;
    let config: OpConfig = serde_json::from_str(&config)?;
    Ok(config)
}

fn read_if_found(path: &Path) -> Result<Option<String>> {
    match std::fs::read_to_string(&path) {
        Ok(c) => Ok(Some(c)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into())
    }
}

// Combine op config with what we have in cache. But only keep whats in op config
pub fn read() -> Result<storage::Cache> {
    let op_config = read_op_config()?;
    if op_config.accounts.is_empty() {
        return Err(Error::msg("No accounts configured in op, must have at least one"))
    }
    let mut cache: storage::Cache = read_if_found(&get_cache_path()?)?.map(|c|
        serde_json::from_str::<storage::Cache>(&c).with_context(||"Error de-serialising cache file")
    ).unwrap_or(Ok(storage::Cache {
        accounts:vec![]
    }))?;

    let accounts: Vec<storage::Account> = op_config.accounts.into_iter().map(|i|
        // Use from cache if exists, otherwise create blank
        cache.accounts.iter()
            .find(|a|a.uuid == i.userUUID)
            .cloned()
            .unwrap_or(storage::Account {
                token: None,
                shorthand: i.shorthand,
                email: i.email,
                uuid: i.userUUID,
                items: vec![]
            })
    ).collect();
    cache.accounts = accounts;
    Ok(cache)
}

pub fn write(cache: &storage::Cache) -> Result<()> {
    let path = get_cache_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Error ensuring path {:?} exists", parent))?;
    }
    let cache = serde_json::to_string(cache)?;
    std::fs::write(&path, cache)
        .with_context(||"Error writing cache file")
}


// Legacy stuff
const TOKEN_PATH: &str = "~/.config/r1pw/token";
const ITEMS_PATH: &str = "~/.config/r1pw/items.json";
const FIELD_CACHE_DIR: &str = "~/.config/r1pw/fields";

fn get_cache_path() -> Result<PathBuf> {
    let path = shellexpand::full(CACHE_PATH)
        .with_context(|| format!("Cache file path {} is invalid", CACHE_PATH))?;
    let path = Path::new(path.as_ref());
    Ok(path.to_owned())
}

fn get_token_path() -> Result<PathBuf> {
    let token_path = shellexpand::full(TOKEN_PATH)
        .with_context(|| format!("Token path {} is invalid", TOKEN_PATH))?;
    let token_path = Path::new(token_path.as_ref());
    Ok(token_path.to_owned())
}

fn get_items_path() -> Result<PathBuf> {
    let token_path = shellexpand::full(ITEMS_PATH)
        .with_context(|| "foo")?;
    let token_path = Path::new(token_path.as_ref());
    Ok(token_path.to_owned())
}

fn get_field_cache_path(key: &str) -> Result<PathBuf> {
    let path = shellexpand::full(FIELD_CACHE_DIR)
        .with_context(||"Field cache error")?;
    let path = Path::new(path.as_ref()).join(Path::new(key).with_extension("json"));
    Ok(path.to_owned())
}

pub fn read_token() -> Result<Option<String>> {
    let token_path = get_token_path()?;
    // Error opening file
    match std::fs::read_to_string(&token_path) {
        Ok(s) => Ok(Some(s.trim().into())),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }.with_context(|| format!("Error opening file {:?}", token_path))
}

pub fn save_token(token: &str) -> Result<()> {
    let token_path = get_token_path()?;
    if let Some(parent) = token_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Error ensuring path {:?} exists", parent))?;
    }
    std::fs::write(&token_path, token)
        .with_context(||format!("Error writing token to path {:?}", token_path))
}

pub fn save_items(items: &Vec<Item>) -> Result<()> {
    std::fs::create_dir_all(get_items_path()?.parent().unwrap())
        .with_context(||"Error ensuring cache path exists")?;
    let a = serde_json::to_string(items)
        .with_context(||"Error serialising Items to cache")?;
    std::fs::write(get_items_path()?, a)
        .with_context(||"Error writing Items cache file")
}

pub fn read_items() -> Result<Vec<Item>> {
    if let Some(a) = match std::fs::read_to_string(get_items_path()?) {
        Ok(a) => Ok(Some(a)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
        .with_context(||"Error reading items from cache file")? {
        let b: Vec<Item> = serde_json::from_str(&a)
            .with_context(||"Error de-serialising Items from cache file")?;
        return Ok(b);
    }
    return Ok(Vec::new());
}

pub fn read_credentials(key: &str) -> Result<Vec<Field>> {
    if let Some(a) = match std::fs::read_to_string(get_field_cache_path(key)?) {
        Ok(a) => Ok(Some(a)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
        .with_context(||"Error reading items from cache file")? {
        let b: Vec<Field> = serde_json::from_str(&a)
            .with_context(||"Error de-serialising Fields from cache file")?;
        return Ok(b);
    }
    return Ok(Vec::new());
}

fn redact(c: &Field) -> Field {
    Field {
        designation: c.designation.to_owned(),
        name: c.name.to_owned(),
        value: "*".repeat(c.value.len()),
    }
}

pub fn write_credentials(key: &str, fields: &Vec<Field>) -> Result<()> {
    std::fs::create_dir_all(get_field_cache_path(&key)?.parent().unwrap())
        .with_context(||"Error creating field cache")?;
    let fields = fields.iter().map(|f|redact(f)).collect_vec();
    let a = serde_json::to_string(&fields)
        .with_context(||"Error serialising fields to cache")?;
    std::fs::write(get_field_cache_path(key)?, a)
        .with_context(||"Error writing field cache")
}
