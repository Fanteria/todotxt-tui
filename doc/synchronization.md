# File Synchronization

Todo.txt TUI supports automatic post-action hooks. This allows you to sync your `todo.txt` file across multiple devices using external synchronization tools.

## Configuration

Add the following options to your `todotxt-tui.toml` configuration file:

```toml
# Script to run after adding a new task
post_new_task = "/home/user/.config/todotxt-tui/sync.sh"

# Script to run after removing a task
post_remove_task = "/home/user/.config/todotxt-tui/sync.sh"

# Script to run after moving a task
post_move_task = "/home/user/.config/todotxt-tui/sync.sh"

# Script to run after updating a task
post_update_task = "/home/user/.config/todotxt-tui/sync.sh"
```

## Example Synchronization Script

Create a synchronization script (*must be executable*) at `/home/user/.config/todotxt-tui/sync.sh`:

```bash
#!/bin/bash

# Synchronize the todo directory containing todo.txt and archive.txt using rclone bisync
/usr/bin/rclone bisync -v --force \
    /home/user/Documents/todo \
    gdrive:/Documents/todo

# You can use any command-line synchronization tool. For example: git, rsync, rclone, syncthing, etc.
```

Hook-based syncing only works when changes are made inside Todo.txt TUI.
To keep files synchronized even when modified on another device or server, you should schedule periodic background synchronization with cron, systemd timer or any other tool like that.

## This Approach Is Not Ideal

Many file synchronization tools struggle when the same file is modified on multiple devices. Instead of merging differences, they often create duplicate copies. Because the application cannot intelligently combine edits, users must manually reconcile conflicts, which can be tedious.

Data loss is another significant risk. Syncing while a file is being written, losing connection during synchronization, or having conflicting changes overwritten can all result in missing information. Even hooks intended to automate tasks can fail silently, leaving you unaware that synchronization did not complete.

Performance may also suffer when synchronization runs too frequently. Executing a sync script after each change can slow down workflows, and network operations may block the application while they complete.

A more robust alternative can be use CalDAV with VTODO support instead of todo.txt.
