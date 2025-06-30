# Marc

CLI tool to annotate anything.

TODO:

- [x] `add` adds a todo
    - [x] should accept a --tag option
    - [x] possibility to add more todos in one command `marc add 'first note' 'second note'`
    - [ ] should prompt the user to add a TODO when there is not stdin or input
    - [ ] should accept stdin or '-' switch
- [x] `log` lists todos
    - [ ] should have a --plain option to log a simple list
    - [ ] should have a --tag option to list todo with the same tag
- [ ] `tag` handles tags
    - [ ] option --create -c to create a new tag
    - [ ] without any arguments it lists all available tags
- [x] `edit` interactive editing of todos
    - [ ] should accept --tag option
    - [x] ability to drop todo
    - [ ] ability to complete a todo
    - [ ] ability to edit a todo, (content and tag)
