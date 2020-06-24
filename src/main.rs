mod cache;
mod clipboard;
mod op;
mod dmenu;

use op::{Credential, Field, Item, OpError};

use log::*;
use itertools::Itertools;
use anyhow::{Result, Error};

fn obtain_token() -> Result<Option<String>> {
    if let Some(token) = cache::read_token()? {
        return Ok(Some(token))
    }
    if let Some(token) = attempt_login()? {
        cache::save_token(&token)?;
        return Ok(Some(token));
    }
    Ok(None)
}

pub fn attempt_login() -> Result<Option<String>> {
    if let Some(pw) = dmenu::prompt_hidden("Unlock:")? {
        let token = op::login(&pw)
            .map_err(|e| return match e {
                OpError::CommandError(exit_status, text) =>
                    Error::msg(format!("Command exited with code {:?}: {}", exit_status.code(), text)),
                OpError::Io(e) => e.into(),
            })?;
        return Ok(Some(token))
    }
    Ok(None)
}

// Main flow
fn main() -> Result<()>{
    pretty_env_logger::init();
    // @TODO: show previous selected item, if set.
    let token = obtain_token()?;

    debug!("Token: '{:?}'", token);
    // if cancelled, proceed

    // @TODO: Implement caching here
    let token = token.expect("Unable to proceed because cache isn't implemented");
    let items = op::get_items(&token).map_err(|e| {
        // Unable to get items, clearing token and exit
        cache::clear_token().unwrap();
        e
    }).unwrap();

    // @TODO: save previous item selection
    if let Some(selection) = display_item_selection(&items)? {
        let credential = op::get_credentials(selection, &token);
        if let Some(field) = display_credential_selection(&credential)? {
            // copy into paste buffer
            debug!("Chosen field is: {}, {}, {}", field.name, field.designation, field.value);
            clipboard::copy_to_clipboard(&field.value);
        }
    }
    // Everything is Ok
    Ok(())
}

fn display_item_selection(items: &Vec<Item>) -> Result<Option<&Item>> {
    // Feed list to dmenu on stdin
    let input = items.iter()
        .map(|item| item.overview.title.to_owned())
        .join("\n");
    // Find choice in list
    Ok(
        dmenu::select(&input)?
            .map(|choice|
                items.iter().find(|&i| i.overview.title == choice)
            ).flatten()
    )
}

fn format_field(field: &Field) -> String {
    format!("Designation: {}, Field name: {}, Value: {}", field.designation, field.name, field.value)
}

fn display_credential_selection(credential: &Credential) -> Result<Option<&Field>> {
    let input = credential.details.fields.iter()
        .map(|field| format_field(field))
        .join("\n");

    Ok(
        dmenu::select(&input)?
            .map(|choice|
                credential.details.fields.iter().find(|&f| format_field(f) == choice)
            ).flatten()
    )
}
