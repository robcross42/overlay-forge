# Known Issues

This file records confirmed issues that are not being fixed immediately.

## Codex Reasoning Escalation

- 2026-06-24 11:49:17 EDT - A prompt was submitted while the session was on Low reasoning, but the task required High reasoning. Codex identified High as the appropriate level yet continued processing instead of stopping and asking the user to switch reasoning level and resubmit. This escalation enforcement bug is deferred for later investigation.
