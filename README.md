3 steps:

List and cache all items
- Serialise object to a yaml file

Done:
- [x] Test run op through rust
- [x] Op login feed stdin from dmenu

Stuff to do today:
- [ ] store token in a file
- [ ] decode json output

Tomorrow:

After:
- [ ] Find some libs for:
 - [ ] file cache for the token?
 - [ ] json/yaml coding
 - [ ] command runner

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
