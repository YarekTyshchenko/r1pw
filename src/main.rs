extern crate r1pw;

use r1pw::*;

use std::io::{Write, Read};
use std::path::Path;
use std::fs::{File, OpenOptions};
use std::io;
use log::*;
use itertools::Itertools;

const TOKEN_PATH: &str = "~/.config/1pw/token";
// Main flow
fn main() {
    pretty_env_logger::init();
    // show previous selected item, if set.
    // check token exists
    let token = read_token_from_path()
        .map(Some)
        .unwrap_or_else(|e| {
            warn!("Token not found: {}", e);
            let t = match attempt_login() {
                Ok(t) => {
                    save_token(&t).expect("Unable to save new token");
                    Some(t)
                },
                Err(LoginError::Cancelled()) => None,
                Err(e) => panic!(e),
            };
            t
        });

    debug!("Token: '{:?}'", token);
    // if cancelled, proceed

    // @TODO: Implement caching here
    let token = token.expect("Unable to proceed because cache isn't implemented");
    let items = get_items(&token).map_err(|e| {
        // Unable to get items, clearing token and exit
        clear_token().unwrap();
        e
    }).unwrap();


    let selection = display_item_selection(&items);
    // save previous item selection

    let credential = get_credentials(selection, &token);
    let field = display_credential_selection(&credential);
    // copy into paste buffer

    debug!("Chosen field is: {}, {}, {}", field.name, field.designation, field.value);
    copy_to_clipboard(&field.value);
}

#[derive(Debug)]
pub enum LoginError {
    Cancelled(),
    FailedDmenu(io::Error),
    FailedOp(OpError),
}

pub fn attempt_login() -> Result<String, LoginError> {
    let pw = match prompt_dmenu("Unlock:") {
        Ok(pw) => Ok(pw),
        Err(DmenuError::Cancelled()) => Err(LoginError::Cancelled()),
        Err(DmenuError::Io(e)) => Err(LoginError::FailedDmenu(e)),
    }.unwrap();
    let token = login_op(&pw);
    let token = match token {
        Ok(t) => t,
        Err(OpError::Io(e)) => panic!("IO Troubles: {}", e),
        Err(OpError::CommandError(code, reason)) => panic!("Op exit code {} with error: {}", code, reason),
    };
    let token = token.trim().to_owned();
    Ok(token)
}

fn read_token_from_path() -> io::Result<String> {
    let token_path = shellexpand::full(TOKEN_PATH).unwrap();
    let token_path = Path::new(token_path.as_ref());
    let mut f = File::open(token_path)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}

fn save_token(token: &str) -> Option<()> {
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

fn clear_token() -> io::Result<()> {
    let token_path = shellexpand::full(TOKEN_PATH).unwrap();
    let token_path = Path::new(token_path.as_ref());
    std::fs::remove_file(token_path)
}

fn display_item_selection(items: &Vec<Item>) -> &Item {
    // Feed list to dmenu on stdin
    let input = items.iter()
        .map(|item| item.overview.title.to_owned())
        .join("\n");
    let choice = select_dmenu(&input);
    // Find choice in list
    debug!("Choice: {}", choice);
    // return item
    let foo = items.iter().find(|&i| i.overview.title == choice).unwrap();
    foo
}

fn format_field(field: &Field) -> String {
    format!("Designation: {}, Field name: {}, Value: {}", field.designation, field.name, field.value)
}

fn display_credential_selection(credential: &Credential) -> &Field {
    let input = credential.details.fields.iter()
        .map(|field| format_field(field))
        .join("\n");
    let choice = select_dmenu(&input);
    // find choice in list
    let foo = credential.details.fields.iter().find(|&f| format_field(f) == choice).unwrap();
    // return item
    foo
}
