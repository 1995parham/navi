# The Skim Overrides of Navi

Navi allows you to override certain parts of skim using command-line arguments.

<!-- TOC -->
* [The Skim Overrides of Navi](#the-skim-overrides-of-navi)
  * [Command line arguments](#command-line-arguments)
<!-- TOC -->

## Command line arguments

Navi allows you to use command line arguments in order to override skim options:

```sh
# if you want to override only when selecting snippets
navi --fzf-overrides '--height 3'

# if you want to override only when selecting argument values
navi --fzf-overrides-var '--height 3'
```

> [!NOTE]
> The CLI arguments still use `--fzf-overrides` naming for backward compatibility, but they configure skim options.
