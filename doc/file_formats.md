# File Formats

Todo.txt TUI supports two storage formats. The format is selected automatically based on the value of `todo_path`:

| `todo_path` value | Format selected |
|---|---|
| A directory | iCalendar — vdirsyncer-style (one `.ics` file per task) |
| File ending in `.ics` or `.ical` | iCalendar — single-file (all tasks in one VCALENDAR) |
| Anything else | todo.txt |

---

## todo.txt

The default format. Each task is a single line in a plain-text file following the [todo.txt specification](https://github.com/todotxt/todo.txt).

- Pending tasks are read from and written to `todo_path`.
- When `archive_path` is set, completed tasks are stored there separately; otherwise all tasks share `todo_path`.

---

## iCalendar (vdirsyncer-style)

Tasks are stored in a directory where each task is an individual `.ics` file named `<uid>.ics`. This is the layout produced by [vdirsyncer](https://vdirsyncer.pimutils.org/) when syncing a CalDAV task list, making it possible to use any CalDAV server as a backend.

### Field mapping

| todo.txt field | iCal property |
|---|---|
| `subject` | `SUMMARY` |
| `priority` (A–D) | `PRIORITY` (1–4) |
| `priority` (E–Z / none) | no `PRIORITY` property |
| `create_date` | `DTSTART` |
| `due_date` | `DUE` |
| `finish_date` | `COMPLETED` |
| `finished` | `STATUS:COMPLETED` / `STATUS:NEEDS-ACTION` |
| `projects` (`+name`) | `CATEGORIES:+name` |
| `contexts` (`@name`) | `CATEGORIES:@name` |
| `hashtags` (`#name`) | `CATEGORIES:#name` |

### Round-trip property preservation

When a `.ics` file is loaded, the full raw VTODO block is stored as a base64-encoded task tag (`_ical_vtodo`). On save, that stored block is parsed back and updated with current field values. This means any iCal properties not listed above (e.g. `DESCRIPTION`, `X-` extensions, `ATTACH`) are preserved across edits made inside Todo.txt TUI.

### Limitations

- **No `archive_path` support.** The `archive_path` config option is ignored for the iCalendar format. Completed tasks remain in the same directory as pending tasks, distinguished by `STATUS:COMPLETED`.
- **Priority range.** Only priorities A–D map to iCal values 1–4. Priorities E–Z and tasks with no priority all produce no `PRIORITY` property, and round-trip back as no priority.
- **`threshold_date` is not mapped.** The todo.txt threshold date (`t:`) has no iCal equivalent and is lost on save.
- **Categories convention.** Projects, contexts, and hashtags are encoded with `+`, `@`, and `#` prefixes in `CATEGORIES`. Other CalDAV clients may not follow this convention, so categories added by another client will be treated as contexts (the fallback).
- **Task ordering.** Tasks are loaded sorted by filename (i.e. by UID), not by any task field.
- **Single-file mode does not clean up removed tasks automatically.** In vdirsyncer-style (directory) mode, `.ics` files for deleted tasks are removed. In single-file mode the file is rewritten as a whole, so there is no equivalent of orphan cleanup — all tasks present in memory are written out on every save.
