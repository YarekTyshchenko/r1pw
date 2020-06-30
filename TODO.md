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
- [x] Handle all cancellations properly
- [x] Break up program into sections/modules
- [x] Cache item list in a simple file
- [x] Refactor program flow
- [x] Cache actual credentials (no, only fields)

Stuff to do today:
- [ ] Read and parse OP config file, to populate Accounts
- [ ] Support multiple accounts
- [ ] Items with the same name

Tomorrow:
- [ ] Display previously selected item for quick access
- [ ] Calculate all totps by secret
- [ ] Find some libs for:
 - [ ] file cache for the token?

After:
- [ ] Query for first totp

Multiple account support:
- Read op config file, and display correct unlock message
- Store account uuid / shorthand as part of the cache
- List items with account prefix, authorise to the right place
  when looking up credentials

Program flows:

Most common use:
- invoke command without params
- show list of cached items
- select an item
- show list of cached credentials (without passwords)
- select credential (or press escape to view password values)
- credential is copied into clipboard
- exit

Multi Account support:
- Read Op config file, to fill in some of the accounts stuff
- Ask for tokens to unlock all the accounts

