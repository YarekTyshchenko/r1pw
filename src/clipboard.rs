use std::process::{Command, Stdio};
use std::io::Write;

pub fn copy_to_clipboard(value: &str) {
    let mut copy = Command::new("xsel")
        .arg("-b")
        .stdin(Stdio::piped())
        .spawn().unwrap();
    let mut stdin = copy.stdin.take().unwrap();
    stdin.write_all(value.as_bytes()).unwrap();
    drop(stdin);
    copy.wait().unwrap();
}
