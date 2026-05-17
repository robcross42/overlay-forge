# Milestone 12 - Project Markdown Context

## Status

Planned / Not Started

## Goal

Add project-level local Markdown context for project chat and bridge draft workflows.

Milestone 12 should prove this workflow:

```text
Project workspace -> configured local project path -> README.md -> referenced Markdown files -> fresh project context for chat and bridge drafts
```

This milestone is project-scoped, not conversation-scoped. The user should not need to attach the same repository documentation to every chat conversation.

## Intended Behavior

- Store or configure a local project documentation root/path for a project.
- Load a fresh copy of `README.md` whenever a new project chat starts or an existing project chat loads.
- Parse `README.md` for references to other local Markdown files.
- Resolve and read referenced local Markdown files when they are inside the configured project root.
- Use the resulting Markdown context as project-level context for:
  - project chat sends
  - Prompt Preview
  - bridge-file draft generation
- Keep conversation-level manual attachments as an additional layer.

## Initial Markdown Sources

Milestone 12 should prioritize:

```text
README.md
CHANGELOG.md
docs/*.md
bridge-files/*.md
```

The first implementation can follow explicit Markdown links and known project documentation paths. It should avoid broad unrestricted filesystem reads.

## Boundary

Milestone 12 should remain local-first.

It should not:

- Read arbitrary filesystem paths outside the configured project root.
- Read GitHub repository file contents through the GitHub API.
- Upload files.
- Add vector stores.
- Add semantic search.
- Add token budgeting unless needed for a basic size guard.
- Export bridge files.
- Hand off directly to Codex.
- Automatically trust generated or external content without user visibility.

## Data Model Direction

Prefer a small additive model, such as:

```text
project_markdown_context
- id
- project_id
- root_path
- readme_path
- created_at
- updated_at
```

If cached file snapshots are needed, use a separate non-destructive table. The default should be fresh reads from disk on chat load/new chat so local documentation changes are reflected.

## Validation Direction

Validate that:

- A project can be linked to a local documentation root.
- `README.md` is loaded freshly when a project chat is opened.
- Referenced Markdown files are included when they resolve inside the project root.
- Chat sees Markdown instructions and repository docs.
- Prompt Preview shows the loaded Markdown context.
- Bridge drafts include relevant project Markdown context.
- Missing files produce readable warnings and do not crash the app.
- Existing Milestone 0 through Milestone 11 behavior remains intact.
