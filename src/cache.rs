use std::io;
use std::io::{Write, Read};
use std::path::Path;
use std::fs::{OpenOptions, File};
use log::debug;

const TOKEN_PATH: &str = "~/.config/1pw/token";

pub fn read_token_from_path() -> io::Result<String> {
    let token_path = shellexpand::full(TOKEN_PATH).unwrap();
    let token_path = Path::new(token_path.as_ref());
    let mut f = File::open(token_path)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
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
