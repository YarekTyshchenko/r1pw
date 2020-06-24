use anyhow::{Context, Result, Error};
use std::io;
use std::path::{Path, PathBuf};

const TOKEN_PATH: &str = "~/.config/1pw/token";

fn get_token_path() -> Result<PathBuf, Error> {
    let token_path = shellexpand::full(TOKEN_PATH)
        .with_context(|| format!("Token path {} is invalid", TOKEN_PATH))?;
    let token_path = Path::new(token_path.as_ref());
    Ok(token_path.to_owned())
}

pub fn read_token() -> Result<Option<String>> {
    let token_path = get_token_path()?;
    // Error opening file
    match std::fs::read_to_string(&token_path) {
        Ok(s) => Ok(Some(s)),
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

pub fn clear_token() -> Result<()> {
    let token_path = get_token_path()?;
    std::fs::remove_file(&token_path)
        .with_context(||format!("Error clearing token {:?}", token_path))
}
