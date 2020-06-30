mod model;
mod cache;
mod clipboard;
mod op;
mod dmenu;

use op::{Field, Item};
use model::logical;
use model::logical::*;
use model::storage;

use log::*;
use itertools::Itertools;
use anyhow::{Result, Context, Error};
use std::collections::HashMap;

fn login_prompt(account: &Account) -> String {
    format!("Unlock for {} ({}):", account.shorthand, account.email)
}

fn query_or_login<T, F: Fn(&str) -> Result<T>>(shorthand: &str, prompt: &str, token: &mut Token, fun: F) -> Result<T> {
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
                    match attempt_login(shorthand, prompt)? {
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

// @FIXME: Should this be using storage account?
fn obtain_token(account: &storage::Account) -> Result<Option<Token>> {
    Ok(match &account.token {
        Some(t) => Some(Token::Stale(t.into())),
        None => match attempt_login(&account.shorthand, &format!("Unlock for {} ({}):", account.shorthand, account.email))? {
            None => None,
            Some(t) => Some(Token::Fresh(t)),
        }
    })
}

fn attempt_login(shorthand: &str, prompt: &str) -> Result<Option<String>> {
    dmenu::prompt_hidden(prompt)?
        .map(|pw| op::login(shorthand, &pw))
        .transpose()
}

fn noop() -> Result<()> {
    Ok(())
}

fn copy_to_clipboard(field: &logical::FullField) {
    debug!("Chosen field is: {}, {}, {}", field.name, field.designation, field.value);
    clipboard::copy_to_clipboard(&field.value);
}

// Main flow
fn main() -> Result<()>{
    pretty_env_logger::init();
    // Read config and cache (as storage::Cache) and convert it into Logical cache
    let mut cache = cache::read()?;
    if cache.accounts.is_empty() {
        return Err(Error::msg("No accounts found"))
    }

    // Convert cached accounts into a vec of mutable accounts, with tokens
    let mut accounts: Vec<logical::Account> = cache.accounts
        .iter()
        .map(|account| {
            let nested_result = obtain_token(account).map(|token| token.map(|token|
                logical::Account {
                    token,
                    shorthand: account.shorthand.clone(),
                    email: account.email.clone(),
                    uuid: account.uuid.clone()
                }
            ).ok_or(Error::msg("Must have a token")));
            match nested_result {
                Ok(Ok(t)) => Ok(t),
                Err(e) => Err(e),
                Ok(Err(e)) => Err(e),
            }
        })
        .collect::<Result<Vec<logical::Account>>>()?;
    debug!("{:?}", accounts);

    // Compute items from all the accounts, querying for real items where they are empty
    let items = cache.accounts
        .into_iter()
        .enumerate()
        .map(|(index, account)| {
            // Borrow mutable to update the token
            let a = accounts.get_mut(index).unwrap();
            debug!("Fetching items for account {}={}, Token: {:?}", account.shorthand, a.shorthand, a.token);
            let items: Vec<logical::Item> = account.items.into_iter().map(|item| logical::Item {
                account_name: (&a.shorthand).clone(),
                account_index: index,
                uuid: item.uuid.clone(),
                name: item.name.clone(),
                url: item.url.to_owned(),
                tags: item.tags.clone(),
                fields: logical::Fields::Redacted(item.fields.into_iter().map(move |field| logical::RedactedField {
                    name: field.name.clone(),
                    designation: field.designation.clone(),
                    value_length: field.value_length,
                }).collect::<Vec<_>>()),
            }).collect::<Vec<_>>();
            if items.is_empty() {
                // @TODO: Deal with this unwrap somehow
                let account_shorthand = &a.shorthand;
                let op_items = query_or_login(account_shorthand, &login_prompt(a), &mut a.token, |t|op::get_items(account_shorthand, t))?;
                let items = op_items.into_iter().map(|item| logical::Item {
                    account_name: account_shorthand.clone(),
                    account_index: index,
                    uuid: item.uuid,
                    name: item.overview.title,
                    url: item.overview.url,
                    tags: item.overview.tags.into_iter().flatten().collect_vec(),
                    fields: logical::Fields::Missing(),
                }).collect::<Vec<_>>();
                return Ok(items);
            }
            return Ok(items);
        })
        .collect::<Result<Vec<_>>>();

    let items: Vec<logical::Item> = items?.into_iter().flat_map(|a|a)
        .collect::<Vec<_>>();
    debug!("{:?}", items);

    // @TODO: show previous selected item, if set.
    // @TODO: save previous item selection
    if let Some(selection) = select(&items, |item| {
        return format!("{}: {}", item.account_name, item.name)
    }, noop)? {
        // Display cached list if not empty
        debug!("{:?}", selection.fields);
        let a = accounts.get_mut(selection.account_index).unwrap();
        match &selection.fields {
            // Simply fetch them from remote and display
            Fields::Missing() => {
                let fields = query_or_login(&a.shorthand, &login_prompt(a), &mut a.token, |t|op::get_credentials(&selection.uuid, t))?;
                debug!("{:?}", fields);
                // Convert to actual fields
                let fields: Vec<logical::FullField> = fields.details.fields.into_iter().map(|f| logical::FullField {
                    name: f.name,
                    designation: f.designation,
                    value: f.value
                })
                    .collect::<Vec<_>>();
                let field = select(&fields, |field| format_field(field), noop)?
                    .ok_or(Error::msg("User cancelled field choice"))?;
                copy_to_clipboard(field);
            },
            Fields::Redacted(fields) => {
                // At the same time attempt to fetch selected item's real values
                let mut full_fields: Option<Vec<logical::FullField>> = None;
                let query_full_fields = || -> Result<()>{
                    debug!("Running something in the closure");
                    full_fields.replace(query_or_login(
                        &a.shorthand, &login_prompt(a), &mut a.token, |t| {
                        let fields = op::get_credentials(&selection.uuid, t)?
                            .details.fields.into_iter().map(|f| logical::FullField {
                            name: f.name,
                            designation: f.designation,
                            value: f.value,
                        })
                            .collect::<Vec<_>>();
                        Ok(fields)
                    })?);
                    Ok(())
                };
                let selected_field = select(&fields, |field| format_redacted_field(field), query_full_fields)?
                    .ok_or(Error::msg("User cancelled field choice"))?;
                let field = full_fields
                    .ok_or(Error::msg("Full fields were never fetched. Likely programming error"))?
                    .into_iter().find(|i| selected_field.name == i.name)
                    .ok_or(Error::msg("Selected field not found in full field list"))?;

                copy_to_clipboard(&field);
            },
        }
    }
    // Save cache
    // Everything is Ok
    // if let Token::Fresh(token) = &token {
    //     cache::save_token(token)?;
    //     // Update item cache on exit
    //     let items = op::get_items(&token)?;
    //     cache::save_items(&items)?;
    // }
    Ok(())
}

fn format_field(field: &logical::FullField) -> String {
    format!("Designation: {}, Field name: {}, Value: {}", field.designation, field.name, field.value)
}

fn format_redacted_field(field: &logical::RedactedField) -> String {
    format!("Designation: {}, Field name: {}, Value: {}", field.designation, field.name, field.value_length)
}

fn select<T, H: Fn(&T) -> String, F: FnOnce() -> Result<()>>(items: &Vec<T>, format: H, foo: F) -> Result<Option<&T>> {
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
