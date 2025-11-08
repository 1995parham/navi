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
