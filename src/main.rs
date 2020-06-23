mod cache;
mod clipboard;
mod op;
mod dmenu;

use op::{Credential, Field, Item, OpError};
use dmenu::{DmenuError};

use std::io;
use log::*;
use itertools::Itertools;

// Main flow
fn main() {
    pretty_env_logger::init();
    // show previous selected item, if set.
    // check token exists
    let token = cache::read_token_from_path()
        .map(Some)
        .unwrap_or_else(|e| {
            warn!("Token not found: {}", e);
            let t = match attempt_login() {
                Ok(t) => {
                    cache::save_token(&t).expect("Unable to save new token");
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
    let items = op::get_items(&token).map_err(|e| {
        // Unable to get items, clearing token and exit
        cache::clear_token().unwrap();
        e
    }).unwrap();


    let selection = display_item_selection(&items);
    // save previous item selection

    let credential = op::get_credentials(selection, &token);
    let field = display_credential_selection(&credential);
    // copy into paste buffer

    debug!("Chosen field is: {}, {}, {}", field.name, field.designation, field.value);
    clipboard::copy_to_clipboard(&field.value);
}

#[derive(Debug)]
pub enum LoginError {
    Cancelled(),
    FailedDmenu(io::Error),
    FailedOp(OpError),
}

pub fn attempt_login() -> Result<String, LoginError> {
    let pw = match dmenu::prompt_hidden("Unlock:") {
        Ok(pw) => Ok(pw),
        Err(DmenuError::Cancelled()) => Err(LoginError::Cancelled()),
        Err(DmenuError::Io(e)) => Err(LoginError::FailedDmenu(e)),
    }.unwrap();
    let token = op::login(&pw);
    let token = match token {
        Ok(t) => t,
        Err(OpError::Io(e)) => panic!("IO Troubles: {}", e),
        Err(OpError::CommandError(code, reason)) => panic!("Op exit code {} with error: {}", code, reason),
    };
    Ok(token)
}

fn display_item_selection(items: &Vec<Item>) -> &Item {
    // Feed list to dmenu on stdin
    let input = items.iter()
        .map(|item| item.overview.title.to_owned())
        .join("\n");
    let choice = dmenu::select(&input);
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
    let choice = dmenu::select(&input);
    // find choice in list
    let foo = credential.details.fields.iter().find(|&f| format_field(f) == choice).unwrap();
    // return item
    foo
}
