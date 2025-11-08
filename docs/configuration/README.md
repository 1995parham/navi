# Configuring Navi

Navi allows you to configure it with a TOML configuration file.

## Paths

On the technical side, navi uses the `etcetera` crate for rust,
which defines platform-specific locations to store the configuration files,
the cache and other types of files an application might need.

> [!TIP]
> For example, this is why cheatsheets are being stored in `~/Library/Application Support/navi` on macOS.

> [!NOTE]
> Interested on how `etcetera` works?\
> Go see their `crates.io` page: [crates.io/crates/etcetera](https://crates.io/crates/etcetera)

### The default configuration file path

The default configuration file path is `~/.config/navi/config.toml` (on Linux/macOS) or `%APPDATA%\navi\config.toml` (on Windows).

You can check your default configuration file path with the info subcommand,
see [/docs/usage/commands/info/](/docs/usage/commands/info/README.md#default-configuration-path) for more details.

### Cheatsheets paths

Navi checks the paths in the following order until it finds a value:

1. the `--path` command-line argument
2. the configuration file
3. The default value of navi

#### The default cheatsheets path

By default, navi stores the cheatsheets in the `~/.local/share/navi/cheats/` directory.

You can check your default cheatsheets path with the info subcommand,
see [/docs/usage/commands/info/](/docs/usage/commands/info/README.md#default-cheatsheets-path) for more details.

#### Defining the cheatsheets path in the configuration file

You can define the cheatsheets path in the configuration file with the following syntax:

```toml
[cheats]
paths = [
    "/path/to/some/dir",  # on unix-like os
    "F:\\path\\to\\dir"   # on Windows
]
```

## Customization

### Changing colors

#### fzf color scheme

You can change the color scheme of `fzf` by overriding fzf options.

> [!NOTE]
> See [@junegunn/fzf/wiki/Color-schemes](https://github.com/junegunn/fzf/wiki/Color-schemes) and
> [#overriding-fzf-options](#overriding-fzf-options) for more details.

#### Navi colors

You can change the text color for each column of navi in the configuration file with the following syntax:

```toml
[style.tag]
color = "<your color for tags>"

[style.comment]
color = "<your color for comments>"

[style.snippet]
color = "<your color for snippets>"
```

Below is an example of what to do if you'd like navi to look like the French flag:

- `config.toml`:

  ```toml
  [style.tag]
  color = "blue"

  [style.comment]
  color = "white"

  [style.snippet]
  color = "red"
  ```

- The result:

  ![navi-custom-colors](https://github.com/user-attachments/assets/d80352c5-d888-43e6-927d-805a8de1a7e2)

### Overriding fzf options

You can override fzf options for different cases using the configuration file or command-line arguments:

- During the cheats selection: use the `overrides` directive or `--fzf-overrides` CLI argument
- During the pre-defined variable values selection: use the `overrides_var` directive or `--fzf-overrides-var` CLI argument
- For all cases: use the `FZF_DEFAULT_OPTS` environment variable

**Example - Overriding during cheats selection:**

```toml
[finder]
overrides = "--height 3"
```

**Example - Overriding during values selection:**

```toml
[finder]
overrides_var = "--height 3"
```

**Example - Overriding for all cases:**

You can define the FZF environment variable like this:

```bash
export FZF_DEFAULT_OPTS="--height 3"
```

> [!NOTE]
> See [@junegunn/fzf](https://github.com/junegunn/fzf#layout) for more details on `$FZF_DEFAULT_OPTS`.

## Defining your own delimiter

Navi allows you to define your own delimiter to parse the selected result for a variable in your cheats.\
It is equivalent to defining `--delimiter` used with `--column`.

You can define it as such:

```toml
[finder]
delimiter_var = "<your-regex-delimiter>"  # By default the expression is \s\s+
```

> [!CAUTION]
> Defining the delimiter via the configuration file means that Navi will use this delimiter by default for
> every variable using the `--column` instruction.

You can override this configuration with the `--delimiter` instruction in the variable definition of your cheat.\
See [/docs/cheatsheet/syntax/](/docs/cheatsheet/syntax/README.md#advanced-variable-options) for more details.
