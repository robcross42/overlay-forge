# Project Rules

Project rules now live in the repository-level agent instruction file:

```text
AGENTS.md
```

Read `AGENTS.md` before planning, implementation, validation, changelog, or milestone handoff work.

`AGENTS.md` also defines the active Reasoning Model Selection Rules. Codex should classify each request before execution, state the selected reasoning level, and default to Medium Reasoning unless complexity, uncertainty, architectural impact, or risk justifies High or Very High Reasoning. If a request needs High or Very High Reasoning, Codex should stop and ask the user to change the VS Code/Codex reasoning setting before execution. After Medium Reasoning tasks, Codex should flag in the final response when Low Reasoning would likely have been sufficient.
