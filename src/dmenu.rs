use std::process::{Command, Stdio};
use std::io::Write;
use std::io;
use log::warn;

pub fn select(input: &str) -> io::Result<Option<String>> {
    dmenu(input, ["-b", "-i", "-l", "20"].to_vec())
}

pub fn prompt_hidden(prompt: &str) -> io::Result<Option<String>> {
    dmenu("", ["-b", "-p", prompt, "-nb", "black", "-nf", "black"].to_vec())
}

fn dmenu(input: &str, args: Vec<&str>) -> io::Result<Option<String>> {
    let mut dmenu = Command::new("dmenu")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdin = dmenu.stdin.take().unwrap();
    stdin.write_all(input.as_bytes())?;
    drop(stdin);

    // Yield control here

    let output = dmenu.wait_with_output()?;
    if !output.status.success() {
        warn!("Dmenu process cancelled with exit code {:?}", output.status.code());
        return Ok(None)
    }
    let choice = String::from_utf8_lossy(&output.stdout);
    let choice = choice.trim().to_owned();
    Ok(Some(choice))
}
