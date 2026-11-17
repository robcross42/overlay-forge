# Repository Boundaries

The host, Media, Gaming, and Retirement repositories were extracted from `overlay-forge` `develop@96aeb79778e148a66b8f88c2b9bcd27b8a415454`. Journal was created independently in October 2026. Products are private and independently owned, with separate SQLite databases; Journal's current local web phase has no packaged application identifier.

| Repository | Owned runtime surface | SQLite ownership | Application identifier |
| --- | --- | --- | --- |
| `overlay-forge` | Resident AI shell, native ChatGPT connection and conversations, typed product discovery and dispatch | Host state plus platform-protected ChatGPT credentials and transcripts in `overlay-forge-host.sqlite3` | `com.overlayforge.desktop` |
| `overlay-forge-media` | Film, television, books, music foundations, canonical media references, media settings, legacy import | Media/book/music/reference tables and `obj_legacy_import` in `overlay-forge-media.sqlite3` | `com.overlayforge.media` |
| `overlay-forge-gaming` | Games, screenshots, chat/build overlays, character builds, guides, GearBlocks tooling, gaming settings, legacy import | Gaming/GearBlocks tables and `obj_legacy_import` in `overlay-forge-gaming.sqlite3` | `com.overlayforge.gaming` |
| `overlay-forge-retirement` | Retirement profile, fresh repair/resell research, poker capture, AI-rig planning, retirement settings, legacy import | Retirement-owned tables and `obj_legacy_import` in `overlay-forge-retirement.sqlite3` | `com.overlayforge.retirement` |
| `overlay-forge-journal` | Morning check-ins, daily entries/reflections, history/patterns, explicit AI analysis | `days`, `entries`, `analyses`, `medications` in `%LOCALAPPDATA%\OverlayForgeJournal\journal.sqlite3` | Local web phase; no packaged identifier |

The former organizer, smoking-cessation, and generic YouTube module surfaces remain outside this repository. See [AI Anchor Decision and Delivery](AI_ANCHOR.md) for the host's shared AI responsibilities. Product credentials and product databases stay with their owning repositories.

YouTube material that remains useful is represented as a Media reference. Legacy imports remain explicit, allowlisted, transactional operations preserving their source database and provenance.

The launcher discovers only known desktop product executable names in installed, current-directory, and sibling development locations. Journal uses a fixed loopback service identity check and browser target, `http://127.0.0.1:4319`. It never accepts an executable path, shell command, or browser URL from persisted or user-editable data. The Journal repository owns server startup, credentials, and data.
