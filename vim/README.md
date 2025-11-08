# Vim Syntax Highlighting for Navi Cheatsheets

This directory contains Vim syntax highlighting for Navi cheatsheet files (`.cheat` and `.cheat.md`).

## Features

- **Tags** (`%`) - Highlighted as titles
- **Comments** (`#`) - Highlighted as comments
- **Metacomments** (`;`) - Highlighted as comments
- **Filters** (`; os:`, `; path:`, `; hostname:`) - Special highlighting for filter keywords and values
- **Extended cheats** (`@`) - Highlighted as includes
- **Variable definitions** (`$`) - Advanced highlighting with:
  - Variable marker (`$`) highlighted distinctly
  - Variable names highlighted as identifiers
  - **Error highlighting** for invalid variable names containing hyphens (e.g., `$ my-var:` shows as error)
  - Full bash syntax highlighting in the command part
  - Support for both variable reference styles in commands:
    - `<variable>` (implicit dependencies) - highlighted as types
    - `$variable` (explicit dependencies) - highlighted as types
  - `---` delimiter highlighted as operator
  - fzf options after `---` highlighted with special emphasis (e.g., `--multi`, `--column`, `--map`)
- **Variable references** - Two styles supported:
  - `<variable>` (implicit dependencies) - highlighted as types with high priority
  - `$variable` (explicit dependencies) - highlighted as types
- **Code blocks** (` ``` `) - Full bash syntax highlighting
- **Command lines** - Full bash/shell syntax highlighting with variable references preserved

## Installation

### Using LazyVim / lazy.nvim

Create a new file `~/.config/nvim/lua/plugins/navi.lua`:

```lua
return {
  "1995parham/navi",
  lazy = true,
  ft = { "cheat" },
  config = function()
    vim.opt.runtimepath:append(vim.fn.stdpath("data") .. "/lazy/navi/vim")
  end,
}
```

The plugin will automatically load when you open `.cheat` or `.cheat.md` files.

### Using vim-plug

Add to your `.vimrc`:

```vim
Plug '1995parham/navi', { 'rtp': 'vim' }
```

Then run `:PlugInstall`

### Using Vundle

Add to your `.vimrc`:

```vim
Plugin '1995parham/navi', { 'rtp': 'vim' }
```

Then run `:PluginInstall`

### Using Pathogen

```bash
cd ~/.vim/bundle
git clone https://github.com/1995parham/navi.git
```

Then create symlinks:

```bash
mkdir -p ~/.vim/syntax ~/.vim/ftdetect
ln -s ~/.vim/bundle/navi/vim/syntax/cheat.vim ~/.vim/syntax/
ln -s ~/.vim/bundle/navi/vim/ftdetect/cheat.vim ~/.vim/ftdetect/
```

### Manual Installation

Copy the files to your Vim directory:

```bash
mkdir -p ~/.vim/syntax ~/.vim/ftdetect
cp vim/syntax/cheat.vim ~/.vim/syntax/
cp vim/ftdetect/cheat.vim ~/.vim/ftdetect/
```

## Usage

The syntax highlighting will automatically activate when you open files with `.cheat` or `.cheat.md` extensions.

## Example

Here's how a cheatsheet will look with syntax highlighting:

```cheat
% git, code

; os: linux
; path: **/projects/**
; hostname: dev-server

# Change branch
git checkout <branch>

# Find and delete merged branches
git branch --merged | grep -v "\*" | xargs -n 1 git branch -d

# Search in git history
git log --all --grep="<search_term>" --oneline

# Echo with variable dependency
echo "$branch is the current branch"

$ branch: git branch | awk '{print $NF}' --- --column 1 --header-lines 1
$ search_term: echo "fix\nbug\nfeature" | tr '\n' ' ' --- --multi --preview 'git log --grep={}'
$ pictures_folder: echo "<home_dir>/pictures"
$ invalid-var: echo "test"  # This will be highlighted as ERROR

@ common, variables
```

**Syntax features shown:**
- Tag lines (`%`) are highlighted distinctly
- Filter metacomments with special highlighting for filter types (`os:`, `path:`, `hostname:`)
- Regular comments (`#`) use comment colors
- Commands have full bash syntax highlighting (keywords like `git`, `grep`, `awk`, pipes `|`, etc.)
- Variable references in two styles:
  - `<branch>` and `<search_term>` - implicit dependencies (highlighted as types)
  - `$branch` - explicit dependencies in commands (highlighted as types)
- Variable definitions starting with `$`:
  - Variable names (`branch`, `search_term`) highlighted as identifiers
  - Bash commands with full syntax highlighting
  - `---` delimiter highlighted as operator
  - fzf options (`--column`, `--multi`, `--preview`, etc.) highlighted specially
  - Nested variable references like `<home_dir>` properly highlighted
  - **Invalid variable names with hyphens** (`invalid-var`) highlighted as errors
- Extended cheats (`@`) are highlighted as includes

## Color Scheme

The syntax file uses standard Vim highlight groups:

- `Title` - for tags (`%`)
- `Comment` - for comments (`#`), metacomments (`;`), and fzf options region
- `Special` - for filter lines, variable markers (`$`), and fzf option flags
- `String` - for filter values
- `Identifier` - for variable names in definitions
- `Type` - for variable references (both `<var>` and `$var` styles)
- `Operator` - for the `---` delimiter
- `Delimiter` - for the `:` separator in variable definitions
- `Include` - for extended cheats (`@`)
- `Error` - for invalid variable names containing hyphens
- `Normal` - for variable definition lines (base highlighting)

Colors will vary based on your colorscheme.
