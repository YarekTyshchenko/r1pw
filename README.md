3 steps:

List and cache all items
- Serialise object to a yaml file

Done:
- [x] Test run op through rust
- [x] Op login feed stdin from dmenu
- [x] store token in a file
- [x] decode json output
- [x] Query item list

Stuff to do today:
- [ ] Cache item list in a simple file
- [ ] Pipe item list into dmenu selector
- [ ] Get choice from list after dmenu selection

Tomorrow:
- [ ] Query single item for passwords
- [ ] Find some libs for:
 - [ ] file cache for the token?

After:
- [ ] Query for first totp
- [ ] Calculate all totps by secret
- [ ] Break up program into sections/modules

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
