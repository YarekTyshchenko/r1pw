use anyhow::{Context, Result};
use std::io;
use std::path::{Path, PathBuf};
use std::io::ErrorKind;

use super::op::{Item};
use crate::op::Field;
use itertools::Itertools;

const TOKEN_PATH: &str = "~/.config/1pw/token";
const CACHE_PATH: &str = "~/.config/1pw/cache.json";
const FIELD_CACHE_DIR: &str = "~/.config/1pw/fields";

fn get_token_path() -> Result<PathBuf> {
    let token_path = shellexpand::full(TOKEN_PATH)
        .with_context(|| format!("Token path {} is invalid", TOKEN_PATH))?;
    let token_path = Path::new(token_path.as_ref());
    Ok(token_path.to_owned())
}

fn get_cache_path() -> Result<PathBuf> {
    let token_path = shellexpand::full(CACHE_PATH)
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
    std::fs::create_dir_all(get_cache_path()?.parent().unwrap())
        .with_context(||"Error ensuring cache path exists")?;
    let a = serde_json::to_string(items)
        .with_context(||"Error serialising Items to cache")?;
    std::fs::write(get_cache_path()?, a)
        .with_context(||"Error writing Items cache file")
}

pub fn read_items() -> Result<Vec<Item>> {
    if let Some(a) = match std::fs::read_to_string(get_cache_path()?) {
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
