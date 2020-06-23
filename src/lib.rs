mod clipboard;
mod dmenu;
mod op;

use op::*;
use dmenu::*;

pub use op::{OpError, Credential, Details, Field, Item, Overview, get_items, get_credentials};
pub use dmenu::DmenuError;
pub use clipboard::copy_to_clipboard;

pub fn select_dmenu(input: &str) -> String {
    dmenu(input, ["-b", "-l", "20"].to_vec()).unwrap()
}

pub fn prompt_dmenu(prompt: &str) -> Result<String, DmenuError> {
    dmenu("", ["-b", "-p", prompt, "-nb", "black", "-nf", "black"].to_vec())
}

pub fn login_op(unlock: &str) -> Result<String, OpError> {
    op(&format!("{}\n", unlock), ["signin", "--output=raw"].to_vec())
}
