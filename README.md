3 steps:

List and cache all items
- Serialise object to a yaml file

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
- [ ] Cache item list in a simple file

Tomorrow:
- [ ] Handle all cancellations properly
- [ ] Calculate all totps by secret
- [ ] Break up program into sections/modules
- [ ] Find some libs for:
 - [ ] file cache for the token?

After:
- [ ] Query for first totp
- [ ] Support multiple accounts

Program main function:
- Display list of items (with accounts) via dmenu, and select one
- display a list of credentials for selection
- Repopulate cache

Notes:
- If item isn't in cache, repopulate cache

Program Flow:

- Read token from cache, escape to abort
- if not found, assume login required
    - login, save token to cache

- 
