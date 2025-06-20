# Marc

CLI tool to annotate anything.

TODO:

- [x] `add` adds a todo to a tag;
    - [ ] should have a --tag option to identify a tag, if it does not exist creates one;
    - [x] possibility to add more todos in one command `marc add 'first note' 'second note'`;
- [x] `log` logs all todos of a tag;
    - [ ] command should have an option to create a new tag with a name;
    - [ ] should have a --tag option to list todo with the same tag;
- [ ] `done` marks a todo as complete;
- [ ] `tag` handles tags;
    - [ ] option --create -c to create a new tag;
    - [ ] without any arguments it lists all available tags;
- [ ] 'edit' edits a todo entry;
    - [ ] should accept an --interactive option
