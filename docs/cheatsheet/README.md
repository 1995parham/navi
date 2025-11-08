# Navi cheatsheets

## Working with `cheatsheet repositories`

Navi works best with what we call `cheatsheet repositories`, for more details see [cheatsheet/repositories](repositories/README.md).

## Manually adding cheatsheets to navi

If you don't want to work with `cheatsheet repositories`, you can manually add your
cheatsheets to navi by putting them into the `cheats_path` of your platform.

You can find out your path using the [info](/docs/usage/commands/info/README.md) subcommands
but a quick working command to go there would be:

```bash
cd $(navi info default-cheats-path)
```

## Choosing between queries and selection with variables

Navi lets you use different methods to fill a variable value, when prompted.

|    Keyboard key    |         Preference         |
| :----------------: | :------------------------: |
|  <kbd> tab </kbd>  |   The query is preferred   |
| <kbd> enter </kbd> | The selection is preferred |

It means if you enter the <kbd> tab </kbd> key, navi will let you enter the value.

## Keyboard shortcuts

When selecting commands and working with navi, these keyboard shortcuts are available:

|        Keyboard key        | Action                                                       |
| :------------------------: | :----------------------------------------------------------- |
|     <kbd> tab </kbd>       | For variables: prefer entering a custom value                |
|    <kbd> enter </kbd>      | For variables: prefer selecting from suggestions             |
| <kbd> ctrl </kbd>+<kbd> e </kbd> | Open selected command in your `$EDITOR` before execution |
| <kbd> ctrl </kbd>+<kbd> o </kbd> | Open the cheat file in your `$EDITOR`                    |
| <kbd> ctrl </kbd>+<kbd> y </kbd> | Copy command to clipboard without executing              |

> [!TIP]
> Use <kbd>ctrl</kbd>+<kbd>e</kbd> to review and modify commands before running them. This is especially useful for complex commands with multiple variables.

## Editor support

Navi provides syntax highlighting for cheatsheet files to improve the editing experience:

- **Vim/Neovim**: See [vim syntax highlighting guide](/vim/README.md) for installation instructions
  - Supports highlighting for tags, variables, filters, comments, and more
  - Works with `.cheat` file extension

Having syntax highlighting makes it easier to write and maintain cheatsheets by clearly distinguishing between different elements like variables, filters, and commands.
