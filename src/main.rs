mod cache;
mod clipboard;
mod op;
mod dmenu;

use op::Field;

use log::*;
use itertools::Itertools;
use anyhow::{Result, Context, Error};
use crate::op::{Item, Credential};
use std::borrow::Cow;

enum Token {
    Stale(String),
    Fresh(String),
}

fn query_or_login<T, F: Fn(&str) -> Result<T>>(token: &mut Token, fun: F) -> Result<T> {
    match &token {
        Token::Fresh(t) => fun(t),
        Token::Stale(t) => {
            match fun(t) {
                Ok(result) => {
                    *token = Token::Fresh(t.into());
                    Ok(result)
                },
                Err(e) => {
                    warn!("Token is stale, requesting new one: {}", e);
                    match attempt_login()? {
                        None => Err(Error::msg("Login cancelled. Unable to proceed without token")),
                        Some(t) => {
                            let result = fun(&t);
                            // New fresh token given
                            *token = Token::Fresh(t);
                            result
                        }
                    }
                },
            }
        }
    }
}

fn obtain_token() -> Result<Option<Token>> {
    Ok(match cache::read_token()? {
        Some(t) => Some(Token::Stale(t)),
        None => match attempt_login()? {
            None => None,
            Some(t) => Some(Token::Fresh(t)),
        }
    })
}

fn attempt_login() -> Result<Option<String>> {
    dmenu::prompt_hidden("Unlock:")?
        .map(|pw| op::login(&pw))
        .transpose()
}

fn clear_token(e: Error) -> Error {
    warn!("Clearing token");
    cache::clear_token().unwrap();
    e
}

enum Fields {
    Redacted(Vec<Field>),
    Full(Vec<Field>),
}

/// Responsible for populating cache
fn get_items(token: &str) -> Result<Vec<Item>> {
    let items = op::get_items(token)?;
    cache::save_items(&items)?;
    Ok(items)
}

fn get_fields(selection: &Item, token: &mut Token) -> Result<Fields> {
    let fields = cache::read_credentials(&selection.uuid)?;
    if ! fields.is_empty() {
        return Ok(Fields::Redacted(fields));
    }
    Ok(Fields::Full(query_or_login(token, |t| {
        let fields = op::get_credentials(&selection, t)?
            .details.fields;
        cache::write_credentials(&selection.uuid, &fields)?;
        Ok(fields)
    })?))
}

fn noop() -> Result<()> {
    Ok(())
}

// Main flow
fn main() -> Result<()>{
    pretty_env_logger::init();
    // If we don't have a token at all, get one, either stale, fresh, or see if user has aborted
    let mut token = match obtain_token()? {
        None => return Err(Error::msg("Unable to proceed without a token")),
        Some(token) => token,
    };

    // Read items from cache, unless its empty
    let mut items = cache::read_items()?;
    if ! items.is_empty() {
        debug!("Items found in cache: {}", items.len());
    } else {
        items = query_or_login(&mut token, get_items)?;
    }
    let items = items;

    // @TODO: show previous selected item, if set.
    // @TODO: save previous item selection
    if let Some(selection) = select(&items, |item| format!("{}", item.overview.title), ||Ok(()))? {
        // Display cached list if not empty
        let fields = get_fields(&selection, &mut token)?;
        let full_chosen_field = match fields {
            Fields::Full(fields) =>
                match select(&fields, |field| format_field(field), noop)? {
                    None => unimplemented!(),
                    Some(field) => {
                        // copy into paste buffer
                        debug!("Chosen field is: {}, {}, {}", field.name, field.designation, field.value);
                        clipboard::copy_to_clipboard(&field.value);
                    },
                }
                    // .map(|f|Cow::Borrowed(&f)),
            Fields::Redacted(fields) => {
                // At the same time attempt to fetch selected item's real values
                let mut full_fields: Option<Vec<Field>> = None;
                let query_full_fields = || -> Result<()>{
                    debug!("Running something in the closure");
                    full_fields.replace(query_or_login(&mut token, |t| {
                        let fields = op::get_credentials(&selection, t)?
                            .details.fields;
                        cache::write_credentials(&selection.uuid, &fields)?;
                        Ok(fields)
                    })?);
                    Ok(())
                };
                match select(&fields, |field| format_field(field), query_full_fields)? {
                    None => return Err(Error::msg("User cancelled field choice")),
                    Some(field) => match full_fields {
                        None => return Err(Error::msg("Full fields were never fetched. Likely programming error")),
                        Some(full_fields) => {
                            // Match up selected field against all fields
                            match full_fields.iter().find(|&i| field.name == i.name) {
                                None => return Err(Error::msg("Selected field not found in full field list")),
                                Some(field) => {
                                    // Some(Cow::Owned(field))
                                    // Unable to get Cow to work
                                    // copy into paste buffer
                                    debug!("Chosen field is: {}, {}, {}", field.name, field.designation, field.value);
                                    clipboard::copy_to_clipboard(&field.value);
                                },
                            }
                        },
                    },
                }
            },
        };
        // Would be location of the Cow field
        // if let Some(field) = full_chosen_field {
            // copy into paste buffer
            // debug!("Chosen field is: {}, {}, {}", field.name, field.designation, field.value);
            // clipboard::copy_to_clipboard(&field.value);
        // }

    }
    // Everything is Ok
    if let Token::Fresh(token) = &token {
        cache::save_token(token)?;
        // Update item cache on exit
        let items = op::get_items(&token)?;
        cache::save_items(&items)?;
    }
    Ok(())
}

fn format_field(field: &Field) -> String {
    format!("Designation: {}, Field name: {}, Value: {}", field.designation, field.name, field.value)
}

fn select<T, F: FnOnce() -> Result<()>>(items: &Vec<T>, format: fn(&T) -> String, foo: F) -> Result<Option<&T>> {
    let input = items.iter()
        .map(|item| format(item))
        .join("\n");

    let result = dmenu::select(&input, foo)
        .with_context(||"Error running dmenu")?
        .map(|choice|
            items.iter().find(|&i| format(i) == choice)
        ).flatten();
    Ok(result)
}
