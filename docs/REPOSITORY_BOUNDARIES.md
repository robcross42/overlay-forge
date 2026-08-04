# Repository Boundaries

All four repositories were extracted from `overlay-forge` `develop@96aeb79778e148a66b8f88c2b9bcd27b8a415454`. Each product is private, independently buildable, and owns a distinct application identifier and SQLite database.

| Repository | Owned runtime surface | SQLite ownership | Application identifier |
| --- | --- | --- | --- |
| `overlay-forge` | Closed product registry, installed-target discovery, launch dispatch, last-used product, picker window state | `obj_host_launcher_state`, `obj_host_window_state` in `overlay-forge-host.sqlite3` | `com.overlayforge.desktop` |
| `overlay-forge-media` | Film, television, books, music foundations, canonical media references, media settings, legacy import | Media/book/music/reference tables and `obj_legacy_import` in `overlay-forge-media.sqlite3` | `com.overlayforge.media` |
| `overlay-forge-gaming` | Games, screenshots, chat/build overlays, character builds, guides, GearBlocks tooling, gaming settings, legacy import | Gaming/GearBlocks tables and `obj_legacy_import` in `overlay-forge-gaming.sqlite3` | `com.overlayforge.gaming` |
| `overlay-forge-retirement` | Retirement profile, fresh repair/resell research, poker capture, AI-rig planning, retirement settings, legacy import | Retirement-owned tables and `obj_legacy_import` in `overlay-forge-retirement.sqlite3` | `com.overlayforge.retirement` |

The former organizer, smoking-cessation, scheduler, generic AI-host, and generic YouTube module surfaces are not owned by the launcher. YouTube material that remains useful is represented as a Media reference. Legacy imports are explicit, allowlisted, transactional operations. They preserve the source database as a rollback artifact and record import provenance in the destination.

The launcher discovers only known product executable names in installed, current-directory, and sibling development locations. It never accepts an executable path or shell command from persisted or user-editable data.
