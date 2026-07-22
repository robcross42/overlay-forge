# Overlay Forge Validation

## Default Validation Commands

Use validation appropriate to the changed area.

```powershell
npm install
npm run build
npm run cargo:build
npm run cargo:clippy
npm run tauri:dev
```

Do not run broader validation than needed for a small change unless the request affects shared behavior.

## Validation Matrix

| Changed area | Preferred validation |
| --- | --- |
| React / TypeScript / frontend UI | `npm run build` |
| Rust / Tauri backend | `npm run cargo:build` |
| Frontend + backend behavior | `npm run check` |
| SQLite migrations | both builds plus migration review |
| OpenAI request path | backend build plus chat-send manual check if possible |
| Screenshot capture | frontend/backend builds plus manual capture/delete flow |
| GearBlocks script exporter | build/type-check plus in-game load/export/import check where possible |
| BepInEx plugin | plugin build plus install/run check where possible |
| Scheduler | backend build plus startup/interval/run-history behavior review |
| Smoking Cessation | frontend/backend builds plus event/keybind/export behavior review |
| Media Library | frontend/backend builds plus migration, provider fixtures, missing-token, progress, refresh-preservation, separate queues, and URL checks |

For broad cleanup and architecture work, run `npm run cargo:clippy` as a review pass. Fix clear no-risk warnings immediately. Record larger warnings as explicit refactor work when they require changing public command shapes, repository APIs, or multiple call sites.

## Core Manual Regression Checklist

Use when changes are broad or touch shell/shared state.

```text
Open the app and reveal the overlay with Ctrl+Shift+Space.
```

Pass criteria:

```text
Overlay appears using existing hotkey behavior.
```

```text
Click a different application while the primary overlay is focused.
```

Pass criteria:

```text
The primary overlay hides and does not remain always-on-top. Standalone game chat and build-guide windows retain their existing always-on-top behavior.
```

```text
Switch between Calendar, Cessation, Repair Resell, Gaming, YouTube, and Settings.
```

Pass criteria:

```text
Each active module loads without disrupting persisted data.
```

```text
Restart the app.
```

Pass criteria:

```text
Persisted records restore correctly.
```

## Retired Projects Module

The former Projects module has no active manual validation path. Legacy SQLite rows are preserved for data safety only.

## Gaming Screenshot Validation

Validated workflow:

```text
Gaming -> selected game -> Capture Screenshot -> saved PNG -> in-app thumbnail preview -> screenshot context menu -> delete cleanup
```

Validate:

```text
Capture a screenshot while a game is the foreground window.
```

Pass criteria:

```text
Overlay Forge hides before capture, the PNG is saved under game-screenshots/<game-slug>/, and the app restores afterward.
```

Validate:

```text
Preview the screenshot in the selected game pane.
```

Pass criteria:

```text
The thumbnail renders through the Tauri asset path.
```

Validate:

```text
Right-click the screenshot and delete it.
```

Pass criteria:

```text
The screenshot PNG, capture manifest, screenshot metadata row, and matching local-path reference rows are removed.
```

## Media Library Validation

Validate without `TMDB_API_READ_ACCESS_TOKEN`:

```text
Open Media Library, browse/edit local entries, then open Catalogue Search.
```

Pass criteria:

```text
Local data remains usable and catalogue search shows a readable missing-credential error.
```

With a valid token, validate movie and series searches, excluding people; add each result twice and confirm the second add is reported as already saved.

Validate movie watched/unwatched state and watched-date editing. For a series, validate individual episode toggles, season/series bulk operations, mark-watched-through, next episode, completion, unwatch-after-completion, and specials settings.

Validate metadata refresh after editing notes, rating, favourite, tags, queue order, progress, and a preferred manual link. All user-owned values must remain. A failed provider refresh must retain cached availability and display a stale/error state.

Validate Watch Next add/remove/up/down behavior and restart persistence. Exercise local type, status, favourite, tag, unwatched, provider, and queue filters while offline.

Validate that `file:`, `javascript:`, and other non-HTTP(S) manual links are rejected. Delete a title and confirm only its locally owned metadata, progress, availability, links, and mappings are removed.

### Books Automated Checks

Run:

```powershell
npm run check
npm run cargo:test
npm run cargo:clippy
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
git diff --check
```

Book tests must cover the populated pre-book table rebuild, preserved IDs/foreign keys/TMDB identities, idempotence, ISBN checksums, provider fixture normalization, Hardcover GraphQL errors, exact-match merging, Open Library throttle intervals, manual books without credentials, refresh preservation, provider-failure isolation, typed progress, preferred edition, Read Next isolation, and HTTP(S)-only links. Provider unit tests use fixtures and do not call live services.

### Books Manual Matrix

1. With no `GOOGLE_BOOKS_API_KEY` and no `HARDCOVER_API_TOKEN`, confirm manual book creation and Open Library search remain usable.
2. With Google configured and Hardcover absent, confirm Google search/add succeeds, Open Library exact-ISBN enrichment is optional, and Settings reports Hardcover as optional/unconfigured.
3. Search with General, Title, Author, and ISBN. Invalid ISBN checksums must fail before a remote request. Add an exact duplicate and confirm the existing entry is returned.
4. Simulate or observe each provider failure independently. A Google failure should permit Open Library fallback; Open Library or Hardcover failure must not block a successful Google add.
5. Create/edit a manual book and edition. Set preferred edition, ownership, format, notes, rating, favourite, tags, and Read Next order; restart and confirm persistence.
6. Exercise page, percent, audiobook-minute, and chapter progress; mark read and reset; verify On Hold and Did Not Finish are not overwritten automatically.
7. Refresh after editing all user-owned fields. Confirm progress, dates, preferred edition, ownership, queue, manual links, series overrides, notes, rating, favourite, and tags remain unchanged.
8. Open provider and user links. Reject `file:`, `javascript:`, whitespace-bearing, and malformed URLs. Confirm unapproved cover hosts render a placeholder.
9. Filter the all-media Library to `BOOK`, then confirm Books rows use reading labels/progress and never display streaming-provider logos.
10. Reopen existing movie/series entries and exercise Watch Next to confirm the BOOK migration and Read Next did not regress video data.

Pass criteria:

```text
Books remain fully local-first, exact matching is conservative, optional providers fail independently,
user state survives refresh/restart, and movie/series behavior is unchanged.
```

## GearBlocks Validation

When GearBlocks runtime work changes, validate as much of this path as possible:

```text
Install or update the Overlay Forge GearBlocks script.
Launch GearBlocks.
Load the script in Script Mods.
Run Export Scene.
Refresh/import scene context in Overlay Forge.
Open GearBlocks chat or parts details.
```

Pass criteria:

```text
Runtime exports import without reparsing unchanged log prefixes, parts are indexed, and chat context reflects the latest available scene state.
```

For future GearBlocks plugin backlog work:

```text
Build the plugin.
Install the DLL under GearBlocks/BepInEx/plugins.
Restart GearBlocks.
Send a ping command, or a marker command only if marker work has explicitly resumed.
```

Pass criteria:

```text
The plugin processes command files and writes status output.
```

## User Pass/Fail Reporting Format

```markdown
# Validation Results

## Overall Result

Pass or Fail

## Failed Items

- Item:
  - Expected:
  - Actual:

## Notes

Any extra observations.
```
