# Marc

Marc is a CLI tool to manage TODOs.

> Thanks to [clig](https://clig.dev/) for the wonferful guidelines on how to write a command-line program.

## Build

```bash
cargo build --release
```

### TODO

- [ ] only use prompts or interactive elements if stdin is an interactive terminal (a TTY)
- [ ] only print escaped characters if the output is not being redirected
- [ ] if --no-input is passed, don’t prompt or do anything interactive
- [x] `add` adds a todo
  - [x] should accept a `--tag` flag
  - [x] possibility to add more todos in one command `marc add 'first note' 'second note'`
  - [ ] should prompt the user when there is not stdin or input
  - [ ] should accept stdin by default if the user provides an empy input, (or by using '-')
- [x] `log` lists todos
  - [x] should have a `--tag` flag to list todo with the same tag
  - [ ] should have flags `undone` `--no-undone`, by defaults it should show completed and not completed todos
  - [ ] should have a `--plain -p` flag
- [ ] `tag` handles tags
  - [ ] flag --create -c to create a new tag
  - [ ] without any arguments it lists all available tags
  - [ ] --prune -p to delete all tags without a corresponding todo
- [x] `edit` interactive editing of todos
  - [ ] should accept --tag flag
  - [x] ability to drop todo
  - [ ] ability to complete a todo
  - [ ] ability to edit a todo, (content and tag)
- [x] `done` command, marks an entry as completed
- [x] `rm` command, removes an entry
  - [x] should accept `--done` to remove all done items
- [ ] consider writing manual for subcommands, e.g.

```
$ myapp help
$ myapp help subcommand
$ myapp subcommand --help
$ myapp subcommand -h
```

- [ ] displays an introductory description and an example (see `jq`)
- [ ] ignore any other flags and arguments that are passed—you should be able to add -h to the end of anything and it should show help. Don’t overload -h
- [ ] write more examples in the docs
- [ ] use formatting in the help text
- [ ] escape characters in an independent terminal way (when heroku apps --help is piped through a pager, the command emits no escape characters)
- [ ] web docs
- [ ] provide man pages
