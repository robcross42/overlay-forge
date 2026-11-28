# Repository Boundaries

Projects is a host-owned private catalog, not another product repository. Its records and one-time
seed ledger live in the host database. References preserve external repository/document ownership;
the catalog cannot modify linked content. Native administrator authorization covers all catalog
commands. Browser clients remain Media-only and receive no catalog data.

The host, Media, Gaming, and Retirement repositories were extracted from `overlay-forge` `develop@96aeb79778e148a66b8f88c2b9bcd27b8a415454`. Journal was created independently in October 2026. Products are private and independently owned, with separate SQLite databases; Journal's current local web phase has no packaged application identifier.

| Repository | Owned runtime surface | SQLite ownership | Application identifier |
| --- | --- | --- | --- |
| `overlay-forge` | Resident AI shell, native ChatGPT connection and conversations, integrated Media composition, headless browser service and approvals, typed product discovery and dispatch | Host state plus platform-protected ChatGPT credentials and transcripts in `overlay-forge-host.sqlite3` | `com.overlayforge.desktop` |
| `overlay-forge-media` | Film, television, books, music foundations, canonical media references, media settings, legacy import | Media/book/music/reference tables and `obj_legacy_import` in `overlay-forge-media.sqlite3` | `com.overlayforge.media` |
| `hockey-card-companion` | Standalone Windows collection app and embeddable read-only COMC inventory | Product-owned saved COMC snapshots in `collection.sqlite3` | `com.robcross42.hockeycardcompanion` |
| `overlay-forge-gaming` | Games, screenshots, chat/build overlays, character builds, guides, GearBlocks tooling, gaming settings, legacy import | Gaming/GearBlocks tables and `obj_legacy_import` in `overlay-forge-gaming.sqlite3` | `com.overlayforge.gaming` |
| `overlay-forge-retirement` | Retirement profile, fresh repair/resell research, poker capture, AI-rig planning, retirement settings, legacy import | Retirement-owned tables and `obj_legacy_import` in `overlay-forge-retirement.sqlite3` | `com.overlayforge.retirement` |
| `overlay-forge-journal` | Morning check-ins, daily entries/reflections, history/patterns, explicit AI analysis | `days`, `entries`, `analyses`, `medications` in `%LOCALAPPDATA%\OverlayForgeJournal\journal.sqlite3` | Local web phase; no packaged identifier |

The former organizer, smoking-cessation, and generic YouTube module surfaces remain outside this repository. See [AI Anchor Decision and Delivery](AI_ANCHOR.md) for the host's shared AI responsibilities. Product credentials and product databases stay with their owning repositories.

YouTube material that remains useful is represented as a Media reference. Legacy imports remain explicit, allowlisted, transactional operations preserving their source database and provenance.

Media is compiled and rendered in the host from the pinned `modules/media` submodule, retaining its own app-data directory and SQLite database. Standalone mode remains for isolated development. No second Media executable or server starts. For other products, the launcher discovers only known desktop product executable names in installed, current-directory, and sibling development locations. Journal uses a fixed loopback service identity check and browser target, `http://127.0.0.1:4319`. It never accepts an executable path, shell command, or browser URL from persisted or user-editable data. The Journal repository owns server startup, credentials, and data.

Hockey Cards links the product's portable library and React view through the pinned `modules/hockey-cards` submodule. It reads saved product data using the native administrator and opt-in browser owner boundaries. The Windows app owns COMC capture and its own profile. Keep product snapshots out of the host and Media databases.

The headless browser service is host-owned; its approval/session database stays separate from the desktop profile and credential stores. The pinned Media crate supplies window-free services to this runtime, keeping product mappings and schema ownership in Media. Browser sessions never inherit a desktop administrator role or its ChatGPT data. See [browser server setup](BROWSER_SERVER.md).
