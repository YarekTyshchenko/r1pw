Rust One Password Utility
-------------------------

This is a utility that plumbs 1Password's `op` utility with `dmenu` to
allow easy selection of items from your accounts.

It requires a correctly configured `op` installation.

It will store its own token in config directory `~/.config/r1pw/`,
tokens are valid for 30 minutes, and after a period of inactivity you
will be asked to unlock the account again.

When selecting a field, it will be automatically copied to your paste
buffer with `xsel` utility.
