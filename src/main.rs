mod cache;
mod clipboard;
mod op;
mod dmenu;

use op::Field;

use log::*;
use itertools::Itertools;
use anyhow::{Result, Context, Error};
use crate::op::{Item, Credential};

enum Token {
    Refused(),
    Stale(String),
    Fresh(String),
}

fn obtain_token() -> Result<Token> {
    if let Some(token) = cache::read_token()? {
        return Ok(Token::Stale(token))
    }
    info!("Attempting to get token from user");
    match attempt_login()? {
        None => Ok(Token::Refused()),
        Some(token) => {
            cache::save_token(&token)?;
            Ok(Token::Fresh(token))
        },
    }
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
enum ActualToken {
    Refused(),
    NotNeeded(),
    Here(String),
}
/// Get items from cache, also getting token if needed
fn get_items() -> Result<(Vec<Item>, Option<Token>)> {
    // Get from cache if possible
    let items = cache::read_items()?;
    if ! items.is_empty() {
        debug!("Items found in cache: {}", items.len());
        return Ok((items, None));
    }

    // We need the token to query for more, we have no choice
    let token = obtain_token()?;
    return match &token {
        Token::Refused() => Ok((items, Some(token))),
        // If its fresh, use it and report the error
        Token::Fresh(t) => {
            // Get items with it
            let items = op::get_items(t)?;
            cache::save_items(&items)?;
            Ok((items, Some(token)))
        },
        // If its stale, attempt, and prompt for login on error
        Token::Stale(t) => {
            // Attempt to get items
            match op::get_items(t) {
                // If success, the token is still fresh
                Ok(items) => {
                    cache::save_items(&items)?;
                    Ok((items, Some(Token::Fresh(t.into()))))
                },
                // On failure attempt to refresh the token
                Err(e) => {
                    warn!("Error getting items with stale token: {:?}", e);
                    let token_foo = attempt_login()?;
                    match token_foo {
                        None => Ok((items, Some(Token::Refused()))),
                        Some(new_token) => {
                            let items = op::get_items(&new_token)?;
                            cache::save_items(&items)?;
                            Ok((items, Some(Token::Fresh(new_token))))
                        },
                    }
                },
            }
        },
    }
}

enum Fields {
    Redacted(Vec<Field>),
    Full(Vec<Field>),
}

fn get_credentials(item: &Item, token: Option<Token>) -> Result<(Fields, Option<Token>)> {
    let fields = cache::read_credentials(&item.uuid)?;
    if ! fields.is_empty() {
        return Ok((Fields::Redacted(fields), None));
    }

    // Its empty, we have no choice but to attempt to read them from Op with token
    return match &token {
        Some(Token::Fresh(t)) | Some(Token::Stale(t)) => {
            let fields = op::get_credentials(item, t)?
                .details.fields;
            cache::write_credentials(&item.uuid, &fields)?;
            Ok((Fields::Full(fields), token))
        }
        Some(Token::Refused()) => Ok((Fields::Redacted(fields), Some(Token::Refused()))),
        None => {
            let token = obtain_token()?;
            match &token {
                Token::Refused() => Ok((Fields::Redacted(fields), Some(Token::Refused()))),
                Token::Fresh(t) => {
                    let fields = op::get_credentials(item, t)?
                        .details.fields;
                    cache::write_credentials(&item.uuid, &fields)?;
                    Ok((Fields::Full(fields), Some(token)))
                },
                Token::Stale(t) => {
                    match op::get_credentials(item, t) {
                        Ok(credential) => {
                            cache::write_credentials(&item.uuid, &credential.details.fields)?;
                            Ok((Fields::Full(credential.details.fields), Some(Token::Fresh(t.into()))))
                        },
                        Err(e) => {
                            warn!("Error getting credential fields with stale token: {:?}", e);
                            let token_foo = attempt_login()?;
                            match token_foo {
                                None => Ok((Fields::Redacted(fields), Some(Token::Refused()))),
                                Some(new_token) => {
                                    let fields = op::get_credentials(item, &new_token)?
                                        .details.fields;
                                    cache::write_credentials(&item.uuid, &fields)?;
                                    Ok((Fields::Full(fields), Some(Token::Fresh(new_token))))
                                },
                            }
                        },
                    }
                },
            }
        },
    }
}

// Main flow
fn main() -> Result<()>{
    pretty_env_logger::init();
    // @TODO: show previous selected item, if set.
    let (items, token) = get_items()?;
    // @TODO: save previous item selection
    if let Some(selection) = select(&items, |item| format!("{}", item.overview.title))? {
        // Display cached list if not empty
        // At the same time attempt to fetch selected item's real values

        let (fields, token) = get_credentials(selection, token)?;
        // Check if value is in item cache
        match &fields {
            Fields::Full(fields) => {
                // If yes, return it
                if let Some(field) = select(&fields, |field|format_field(field))? {
                    // copy into paste buffer
                    debug!("Chosen field is: {}, {}, {}", field.name, field.designation, field.value);
                    clipboard::copy_to_clipboard(&field.value);
                }
            },
            Fields::Redacted(field) => {
                // If yes, return it
                // Spawn another thread to lookup actual values
                let a = match &token {
                    Some(Token::Fresh(t)) | Some(Token::Stale(t)) => {
                        op::get_credentials(selection, t)?
                    },
                    Some(Token::Refused()) => {
                        unimplemented!()
                    }
                    None => {
                        let token = obtain_token()?;
                        match &token {
                            Token::Refused() => {
                                unimplemented!()
                            },
                            Token::Fresh(t) => {
                                op::get_credentials(selection, t)?
                            },
                            Token::Stale(t) => {
                                match op::get_credentials(selection, t) {
                                    Ok(credential) => credential,
                                    Err(e) => {
                                        warn!("Error getting credential fields with stale token: {:?}", e);
                                        let token_foo = attempt_login()?;
                                        match token_foo {
                                            None => unimplemented!(),
                                            Some(new_token) => op::get_credentials(selection, &new_token)?,
                                        }
                                    },
                                }
                            },
                        }
                    },
                };
                let f = match &fields {
                    Fields::Full(f) | Fields::Redacted(f) => f
                };
                if let Some(field) = select(f, |field|format_field(field))? {
                    // Wait for previous thread to finish
                    // @TODO: This has severe race conditions with dmenu Login
                    let field = a.details.fields.iter().find(|&i|field.name == i.name)
                        .unwrap();
                    // copy into paste buffer
                    debug!("Chosen field is: {}, {}, {}", field.name, field.designation, field.value);
                    clipboard::copy_to_clipboard(&field.value);
                }
            },
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
