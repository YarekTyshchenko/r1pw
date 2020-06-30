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

fn query_or_login<T, F: Fn(&str) -> Result<T>>(prompt: &str, token: &mut Token, fun: F) -> Result<T> {
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
                    match attempt_login(prompt)? {
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
        None => match attempt_login(&format!("Unlock for {} ({}):", account.shorthand, account.email))? {
            None => None,
            Some(t) => Some(Token::Fresh(t)),
        }
    })
}

fn attempt_login(prompt: &str) -> Result<Option<String>> {
    dmenu::prompt_hidden(prompt)?
        .map(|pw| op::login(&pw))
        .transpose()
}

/// Responsible for populating cache
fn get_items(token: &str) -> Result<Vec<Item>> {
    let items = op::get_items(token)?;
    cache::save_items(&items)?;
    Ok(items)
}

// fn get_fields(selection: &Item, token: &mut Token) -> Result<Fields> {
//     let fields = cache::read_credentials(&selection.uuid)?;
//     if ! fields.is_empty() {
//         return Ok(Fields::Redacted(fields));
//     }
//     Ok(Fields::Full(query_or_login(token, |t| {
//         let fields = op::get_credentials(&selection, t)?
//             .details.fields;
//         cache::write_credentials(&selection.uuid, &fields)?;
//         Ok(fields)
//     })?))
// }

fn noop() -> Result<()> {
    Ok(())
}

// fn copy_to_clipboard(field: &Field) {
//     debug!("Chosen field is: {}, {}, {}", field.name, field.designation, field.value);
//     clipboard::copy_to_clipboard(&field.value);
// }

// Main flow
fn main() -> Result<()>{
    pretty_env_logger::init();
    // Read config and cache (as storage::Cache) and convert it into Logical cache
    let mut cache = cache::read()?;
    if cache.accounts.is_empty() {
        return Err(Error::msg("No accounts found"))
    }


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

    // Compute items from all the accounts, querying for real items where they are empty
    let items: Vec<logical::Item> = cache.accounts
        .into_iter()
        .enumerate()
        .map(|(index, account)| {
            let items: Vec<logical::Item> = account.items.into_iter().map(move |item| logical::Item {
                account: index,
                uuid: item.uuid.clone(),
                name: item.name.clone(),
                url: item.url.to_owned(),
                tags: item.tags.clone(),
                fields: logical::Fields::Redacted(item.fields.into_iter().map(move |field| logical::RedactedField {
                    name: field.name.clone(),
                    designation: field.designation.clone(),
                    value_length: field.value_length,
                }).collect()),
            }).collect();
            return items;


            // @TODO: Deal with this unwrap somehow
            // let a = accounts.get_mut(index).unwrap();
            // let op_items = query_or_login(&login_prompt(a), &mut a.token, op::get_items)?;
            // let items = op_items.into_iter().map(move |item| logical::Item {
            //     account: index,
            //     uuid: item.uuid,
            //     name: item.overview.title,
            //     url: item.overview.url,
            //     tags: item.overview.tags.into_iter().flatten().collect_vec(),
            //     fields: logical::Fields::Missing(),
            // }).collect();
            //return Ok(items.into_iter());
        })
        .collect();

    // let items = items.into_iter()
    //     .collect::<Vec<_>>();

    // if items.is_empty() {
    //     // go through each account, and populate items
    // }
    // let mut accounts: Vec<logical::Account> = Vec::new();
    // let mut items: Vec<logical::Item> = Vec::new();
    // for account in cache.accounts {
    //     let logical_account = Account {
    //         token: Token::Stale("".to_owned()), // @TODO: Get real token
    //         shorthand: account.shorthand,
    //         email: account.email,
    //         uuid: account.uuid,
    //     };
    //     items.push(logical::Item {
    //         account: Box::new(&logical_account),
    //         uuid: "".to_string(),
    //         name: "".to_string(),
    //         url: "".to_string(),
    //         tags: vec![],
    //         fields: vec![]
    //     });
    //     accounts.push(logical_account);
    // }

    // let mut tokens = HashMap::<String, Token>::new();
    // for account in &mut cache.accounts {
    //     if let Some(token) = obtain_token(account)? {
    //         // Only relevant if the token is fresh...
    //         account.token.replace((&token).into());
    //         tokens.insert((&account.uuid).into(), token);
    //     }
    //
    //     debug!("Items found in cache: {}", account.items.len());
    //     if account.items.is_empty() {
    //         let items = query_or_login(&account, tokens.get_mut((&account.uuid).into()).unwrap(), get_items)?;
    //         account.items = items.into_iter().map(|i|model::Item {
    //             uuid: i.uuid,
    //             name: i.overview.title,
    //             url: i.overview.url,
    //             fields: vec![],
    //             tags: i.overview.tags.unwrap_or(vec![])
    //         }).collect();
    //     }
    //
    // }
    debug!("{:?}", accounts);
    debug!("{:?}", items);
    // // Read items from cache, unless its empty
    // let mut items = cache::read_items()?;
    // if ! items.is_empty() {
    //     debug!("Items found in cache: {}", items.len());
    // } else {
    //     items = query_or_login(&mut token, get_items)?;
    // }
    // let items = items;
    //
    // // @TODO: show previous selected item, if set.
    // // @TODO: save previous item selection
    // if let Some(selection) = select(&items, |item| format!("{}", item.overview.title), ||Ok(()))? {
    //     // Display cached list if not empty
    //     let fields = get_fields(&selection, &mut token)?;
    //     match fields {
    //         Fields::Full(fields) => {
    //             let field = select(&fields, |field| format_field(field), noop)?
    //                 .ok_or(Error::msg("User cancelled field choice"))?;
    //             copy_to_clipboard(field);
    //         },
    //         Fields::Redacted(fields) => {
    //             // At the same time attempt to fetch selected item's real values
    //             let mut full_fields: Option<Vec<Field>> = None;
    //             let query_full_fields = || -> Result<()>{
    //                 debug!("Running something in the closure");
    //                 full_fields.replace(query_or_login(&mut token, |t| {
    //                     let fields = op::get_credentials(&selection, t)?
    //                         .details.fields;
    //                     cache::write_credentials(&selection.uuid, &fields)?;
    //                     Ok(fields)
    //                 })?);
    //                 Ok(())
    //             };
    //             let selected_field = select(&fields, |field| format_field(field), query_full_fields)?
    //                 .ok_or(Error::msg("User cancelled field choice"))?;
    //             let field = full_fields
    //                 .ok_or(Error::msg("Full fields were never fetched. Likely programming error"))?
    //                 .into_iter().find(|i| selected_field.name == i.name)
    //                 .ok_or(Error::msg("Selected field not found in full field list"))?;
    //
    //             copy_to_clipboard(&field);
    //         },
    //     };
    // }
    // // Everything is Ok
    // if let Token::Fresh(token) = &token {
    //     cache::save_token(token)?;
    //     // Update item cache on exit
    //     let items = op::get_items(&token)?;
    //     cache::save_items(&items)?;
    // }
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
