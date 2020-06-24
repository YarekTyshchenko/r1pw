mod cache;
mod clipboard;
mod op;
mod dmenu;

use op::Field;

use log::*;
use itertools::Itertools;
use anyhow::{Result, Context};

fn obtain_token() -> Result<Option<String>> {
    if let Some(token) = cache::read_token()? {
        return Ok(Some(token))
    }
    attempt_login()?
        .map(|token|
            cache::save_token(&token).map(|_|token)
        ).transpose()
}

fn attempt_login() -> Result<Option<String>> {
    dmenu::prompt_hidden("Unlock:")?
        .map(|pw| op::login(&pw))
        .transpose()
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
        warn!("Clearing token");
        cache::clear_token().unwrap();
        e
    }).with_context(||"Failed fetching item list from `op`")?;

    // @TODO: save previous item selection
    if let Some(selection) = select(&items, |item| format!("{}", item.overview.title))? {
        let credential = op::get_credentials(selection, &token);
        if let Some(field) = select(&credential.details.fields, |field|format_field(field))? {
            // copy into paste buffer
            debug!("Chosen field is: {}, {}, {}", field.name, field.designation, field.value);
            clipboard::copy_to_clipboard(&field.value);
        }
    }
    // Everything is Ok
    Ok(())
}

fn format_field(field: &Field) -> String {
    format!("Designation: {}, Field name: {}, Value: {}", field.designation, field.name, field.value)
}

fn select<T>(items: &Vec<T>, format: fn(&T) -> String) -> Result<Option<&T>> {
    let input = items.iter()
        .map(|item| format(item))
        .join("\n");

    let result = dmenu::select(&input)
        .with_context(||"Error running dmenu")?
        .map(|choice|
            items.iter().find(|&i| format(i) == choice)
        ).flatten();
    Ok(result)
}
