# Getting Started with Navi Cheatsheets

This guide will help you create your first navi cheatsheet.

## What is a cheatsheet?

A cheatsheet is a file with `.cheat` or `.cheat.md` extension containing commands and their descriptions.
Cheatsheets allow you to store commonly used commands with dynamic variables that can be filled in interactively.

## Creating your first cheatsheet

1. Navigate to your cheats directory:

   ```bash
   cd $(navi info default-cheats-path)
   ```

2. Create a new file called `my-commands.cheat`:

   ```sh
   % git, basics

   # Show current branch
   git branch --show-current

   # Switch to a branch
   git checkout <branch>

   $ branch: git branch | awk '{print $NF}'
   ```

3. Run `navi` in your terminal to see your cheatsheet in action!

## Understanding the syntax

In the example above:
- `% git, basics` - Tags that help categorize and search for this cheat
- `# Show current branch` - Description of what the command does
- `git branch --show-current` - The actual command to execute
- `<branch>` - A variable that will be filled interactively
- `$ branch: ...` - Defines how to generate suggestions for the `<branch>` variable

## Next steps

- Learn about the full [cheatsheet syntax](../syntax/README.md)
- Explore [filtering commands](../syntax/README.md#filtering-commands) by OS, path, or hostname
- Check out [cheatsheet repositories](../repositories/README.md) to share your cheats
- Set up [vim syntax highlighting](/vim/README.md) for better editing experience
