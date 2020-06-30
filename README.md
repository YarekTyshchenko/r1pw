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

With thanks to the awesome
[rust-lang](https://discord.com/invite/rust-lang) community on Discord!

Installation
============
Source code is available on Github
[https://github.com/YarekTyshchenko/r1pw](https://github.com/YarekTyshchenko/r1pw)

Clone, run `cargo build --release` and copy the binary from `target/`
to a handy location which is in your `$PATH`.

Ensure you have `dmenu`, `op`, and `xsel` installed

How to use
==========

On first run of `r1pw` it will prompt you to unlock all accounts that
are configured in `op`, storing their tokens, and list of items in
cache.

On subsequent runs the cache will be re-used, yet actual password values
will always be fetched from `op`.

Only fields that are saved for are `name` and `designation` (as well as
length of the password value to print some stars, to give some sort of
indication of what you are about to copy).

Disclaimer
==========

Be warned that this was hacked together while learning Rust, so expect
various holes / security issues. Any feedback is welcome, constructive
or otherwise.
