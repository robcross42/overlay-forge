# Design Notes

This file stores design thoughts to revisit later.

Entries in this file are notes only. They do not imply planned work, implementation scope, or prioritization unless a later request explicitly moves them into the project plan or deferred items.

## Inbox

- 2026-06-24 10:59:27 EDT - Review why Codex did not establish a shared `window` or `standalone-window` class before adding standalone overlay windows. The missing shared class led to repeated troubleshooting and regressions as additional windows were introduced. Revisit project markdown guidance to make this kind of reusable UI contract more likely in future work.
- 2026-06-24 11:35:54 EDT - Review why the standalone window opacity troubleshooting progressed one suspected cause at a time instead of mapping all layers that can affect transparency up front. Each fix attempt appeared to address the first plausible element, then stopped, which required repeated follow-up attempts for downstream layers.
