use std::process::{Command, Stdio};
use std::io::Write;
use std::io;
use log::warn;

#[derive(Debug)]
pub enum DmenuError {
    Cancelled(),
    Io(io::Error),
}

pub fn dmenu(input: &str, args: Vec<&str>) -> Result<String, DmenuError> {
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
    if !output.status.success() {
        warn!("Dmenu process cancelled with exit code {:?}", output.status.code());
        return Err(DmenuError::Cancelled());
    }
    let choice = String::from_utf8_lossy(&output.stdout);
    let choice = choice.trim();
    Ok(choice.to_owned())
}
