Rust One Password Utility
=========================

Plumbs `op` with `dmenu`. Requires correct setup of `op` to work.

Todo
----
Done:
- [x] Test run op through rust
- [x] Op login feed stdin from dmenu
- [x] store token in a file
- [x] decode json output
- [x] Query item list
- [x] Login if fetch failed
- [x] Pipe item list into dmenu selector
- [x] Get choice from list after dmenu selection
- [x] Query single item for passwords
- [x] Copy password value into clipboard
- [x] Switch everything to use rust logger

Stuff to do today:
- [ ] Refactor program flow
- [ ] Cache item list in a simple file

Tomorrow:
- [ ] Display previously selected item for quick access
- [ ] Handle all cancellations properly
- [ ] Calculate all totps by secret
- [ ] Break up program into sections/modules
- [ ] Find some libs for:
 - [ ] file cache for the token?

After:
- [ ] Query for first totp
- [ ] Support multiple accounts
