use anyhow::{Context, Result};
use std::io;
use std::io::Write;
use std::path::Path;
use std::fs::OpenOptions;
use log::debug;

const TOKEN_PATH: &str = "~/.config/1pw/token";

pub fn read_token_from_path() -> Result<Option<String>> {
    let token_path = shellexpand::full(TOKEN_PATH)
        .with_context(|| format!("Token path {} is invalid", TOKEN_PATH))?;

    let token_path = Path::new(token_path.as_ref());
    // Error opening file
    match std::fs::read_to_string(token_path) {
        Ok(s) => Ok(Some(s)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }.with_context(|| format!("Error opening file {:?}", token_path))
}

pub fn save_token(token: &str) -> Option<()> {
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

pub fn clear_token() -> io::Result<()> {
    let token_path = shellexpand::full(TOKEN_PATH).unwrap();
    let token_path = Path::new(token_path.as_ref());
    std::fs::remove_file(token_path)
}
