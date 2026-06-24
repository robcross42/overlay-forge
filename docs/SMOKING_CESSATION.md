# Smoking Cessation Module

## Status

Complete / Passed / Successful

## Purpose

Smoking Cessation is a local-first core feature module inside the shell-owned component host.

The module records cigarette events, tracks current cigarette inventory, displays the current patch marker, supports a configurable record keybind, and renders charts from local SQLite data.

## Current Patch Marker

Initial marker:

```text
patch_label = Nicoderm Step 1
patch_started_at = 2026-06-21 15:00:00
patch_timezone = EDT
```

## Persistence

SQLite remains the source of truth.

Tables:

```text
obj_smoking_event
obj_smoking_cessation_setting
```

`obj_smoking_event` stores timestamped cigarette records with a source marker for module or keybind entry.

`obj_smoking_cessation_setting` stores the singleton patch marker and current cigarette inventory count.

## Event Behavior

- Cigarette record actions create timestamped SQLite rows.
- New cigarette records decrement the current cigarette inventory count.
- Deleting a record removes only that event row.
- Historical event rows are not inferred from inventory changes.
- Frontend charts are derived from SQLite rows at render time.

## Keybind Behavior

The optional `Record Cigarette` shortcut uses the existing Settings keybind system.

Future work should keep the shortcut configurable through Settings and should not hard-code a global key combination without user control.

## Markdown Export

The module can render a narrow Markdown export under app data for personal context review:

```text
%APPDATA%\com.overlayforge.desktop\chatgpt-exports\smoking-cessation.md
```

This file is derived output. SQLite remains authoritative.

The Scheduler framework refreshes this export on startup and then every 60 seconds through a known Rust handler.

## Scope Guard

Do not add cloud sync, health-provider integrations, medical advice workflows, or notification behavior based on health claims unless explicitly requested.
