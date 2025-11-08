<div align="center">
  <h1>navi</h1>
  <img src="https://raw.githubusercontent.com/1995parham/navi/main/assets/icon.png" alt="icon" height="28px"/>
  <br>
  <img alt="GitHub Actions Workflow Status" src="https://img.shields.io/github/actions/workflow/status/1995parham/navi/ci.yml?style=for-the-badge&logo=github&label=CI">
  <img alt="GitHub Actions Workflow Status" src="https://img.shields.io/github/actions/workflow/status/1995parham/navi/cd.yml?style=for-the-badge&logo=github&label=Publish">
</div>

An interactive cheatsheet tool for the command-line.

[![Demo](https://asciinema.org/a/406461.svg)](https://asciinema.org/a/406461)

**navi** allows you to browse through cheatsheets (that you may write yourself or download from maintainers) and execute commands. Suggested values for arguments are dynamically displayed in a list.

## Pros

- it will spare you from knowing CLIs by heart
- it will spare you from copy-pasting output from intermediate commands
- it will make you type less
- it will teach you new one-liners

It uses [fzf](https://github.com/junegunn/fzf) under the hood and it can be either used as a command or as a shell widget (_à la_ Ctrl-R).

## Usage

There are multiple ways to use **navi**:

- by typing `navi` in the terminal
  - pros: you have access to all possible subcommands and flags
- as a [shell widget](docs/widgets/README.md#installing-the-shell-widget) for the terminal
  - pros: the shell history is correctly populated (i.e. with the actual command you ran instead of `navi`) and you can edit the command as you wish before executing it
- as a [Tmux widget](docs/widgets/howto/TMUX.md)
  - pros: you can use your cheatsheets in any command-line app even in SSH sessions
- as [aliases](docs/cheatsheet/syntax/README.md#aliases)
- as a [shell scripting tool](docs/usage/shell-scripting/README.md)

## Cheatsheet repositories

Running **navi** for the first time will help you download and manage cheatsheets. By default, they are stored at `~/.local/share/navi/cheats/`.

You can also:

- [browse through featured cheatsheets](docs/usage/commands/repo/README.md#browsing-through-cheatsheet-repositories)
- [import cheatsheets from git repositories](docs/cheatsheet/repositories/README.md#importing-cheatsheet-repositories)
- [write your own cheatsheets](#cheatsheet-syntax) (and [share them](docs/cheatsheet/repositories/README.md#submitting-cheatsheets), if you want)
- [auto-update repositories](docs/cheatsheet/repositories/README.md#auto-updating-repositories)
- auto-export cheatsheets from your [TiddlyWiki](https://tiddlywiki.com/) notes using a [TiddlyWiki plugin](https://bimlas.github.io/tw5-navi-cheatsheet/)

## Cheatsheet syntax

Cheatsheets are described in `.cheat` files that look like this:

```sh
% git, code

# Change branch
git checkout <branch>

$ branch: git branch | awk '{print $NF}'
```

The full syntax and examples can be found [here](docs/cheatsheet/syntax/README.md).

## Customization

You can:

- [setup your own config file](docs/configuration/README.md)
- [set custom paths for your config file and cheat sheets](docs/configuration/README.md#paths-and-environment-variables)
- [change colors](docs/configuration/README.md#changing-colors)
- [resize columns](docs/configuration/README.md#resizing-columns)
- [change how search is performed](docs/configuration/README.md#overriding-fzf-options)

## More info

Please run the following command to read more about all possible options:

```sh
navi --help
```

In addition, please check the [/docs](docs) folder or the website.
