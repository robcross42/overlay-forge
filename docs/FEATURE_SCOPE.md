# Overlay Forge Host Scope

In scope: private product discovery, typed target dispatch, last-used resume, minimal fallback picker, host window state, and host-global lifecycle behavior.

Out of scope: embedded product UI or business logic, product settings, product credentials, product persistence, dynamic code loading, marketplace behavior, automatic updates, cloud services, and executable commands stored in SQLite or editable configuration.

Product work belongs in the Media, Gaming, or Retirement repository and changes only that product's version.

Deferred module concepts are not current scope. `LanInputBridge` is recorded in
[Future Module Concepts](FUTURE_FEATURES.md); implementing it requires an explicit decision to
change the current host-only repository boundary.
