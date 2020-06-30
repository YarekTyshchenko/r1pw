use anyhow::{Context, Result, Error};
use std::path::{Path, PathBuf};
use std::io::ErrorKind;

use crate::model::storage;
use crate::model::op::OpConfig;

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
            .find(|a|a.uuid == i.user_uuid)
            .cloned()
            .unwrap_or(storage::Account {
                token: None,
                shorthand: i.shorthand,
                email: i.email,
                uuid: i.user_uuid,
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

fn get_cache_path() -> Result<PathBuf> {
    let path = shellexpand::full(CACHE_PATH)
        .with_context(|| format!("Cache file path {} is invalid", CACHE_PATH))?;
    let path = Path::new(path.as_ref());
    Ok(path.to_owned())
}
