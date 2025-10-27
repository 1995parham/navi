# Vim Syntax Highlighting for Navi Cheatsheets

This directory contains Vim syntax highlighting for Navi cheatsheet files (`.cheat` and `.cheat.md`).

## Features

- **Tags** (`%`) - Highlighted as titles
- **Comments** (`#`) - Highlighted as comments
- **Metacomments** (`;`) - Highlighted as comments
- **Filters** (`; os:`, `; path:`, `; hostname:`) - Special highlighting
- **Variables** (`$`) - Highlighted as identifiers
- **Extended cheats** (`@`) - Highlighted as includes
- **Variable references** (`<variable>`) - Highlighted as types (with high priority)
- **Code blocks** (` ``` `) - Bash syntax highlighting
- **Variable delimiters** (`---`) - Highlighted as operators
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

$ branch: git branch | awk '{print $NF}'
$ search_term: echo "fix\nbug\nfeature" | tr '\n' ' '

@ common, variables
```

**Syntax features shown:**
- Tag lines (`%`) are highlighted distinctly
- Filter metacomments with special highlighting for filter types
- Regular comments (`#`) use comment colors
- Commands have full bash syntax highlighting (keywords like `git`, `grep`, `awk`, pipes `|`, etc.)
- Variable references `<branch>` and `<search_term>` are prominently highlighted
- Variable definitions starting with `$` show bash syntax for the command part
- Extended cheats (`@`) are highlighted as includes

## Color Scheme

The syntax file uses standard Vim highlight groups:

- `Title` - for tags
- `Comment` - for comments and metacomments
- `Special` - for filter lines
- `String` - for filter values
- `Identifier` - for variable definitions
- `Type` - for variable references
- `Operator` - for variable delimiters
- `Include` - for extended cheats

Colors will vary based on your colorscheme.
