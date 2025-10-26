# The repo subcommands of navi

<!-- TOC -->
* [The repo subcommands of navi](#the-repo-subcommands-of-navi)
  * [Commands Reference](#commands-reference)
  * [Browsing through cheatsheet repositories](#browsing-through-cheatsheet-repositories)
  * [Importing cheatsheet repositories](#importing-cheatsheet-repositories)
<!-- TOC -->

## Commands Reference

| Command | Description                                                       |
|---------|-------------------------------------------------------------------|
| add     | Lets you import a cheatsheet repository                           |
| browser | Lets you browse through a curated list of cheatsheet repositories |

## Browsing through cheatsheet repositories

You can browse cheatsheet repositories with the following command:

```sh
navi repo browse
```

## Importing cheatsheet repositories

You can import `cheatsheet repositories` using a working git-clone format.\
This includes using an HTTPS URL or an SSH URI.

- Import using HTTPS

    ```sh
    navi repo add https://github.com/<user>/<repo>
    ```

- Import using SSH

    ```shell
    navi repo add git@github.com:<user>/<repo>
    ```

> [!TIP]
> Repositories are cloned into the default cheatsheets directory. To use a custom path, you can:
> - Clone the repository manually to your preferred location
> - Use the `--path` CLI argument to specify a custom cheatsheets directory
> - Configure custom paths in your `config.toml` file
