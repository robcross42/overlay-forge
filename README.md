# Overlay Forge

Overlay Forge is the private launcher for three independently owned local-first Tauri products.

```text
Current stable app release: 1.0.0
Source extraction baseline: develop@96aeb79778e148a66b8f88c2b9bcd27b8a415454
```

The host contains no product business logic. On startup it launches the requested product, otherwise resumes the last successfully launched product, and shows a minimal picker only when no valid target is available.

The authoritative ownership inventory and extraction provenance are in [`docs/REPOSITORY_BOUNDARIES.md`](docs/REPOSITORY_BOUNDARIES.md).

| Product | Repository |
| --- | --- |
| Media | `robcross42/overlay-forge-media` |
| Gaming | `robcross42/overlay-forge-gaming` |
| Retirement | `robcross42/overlay-forge-retirement` |

Targets are closed Rust enum values: `media`, `gaming`, and `retirement`. Example:

```powershell
overlay-forge.exe --product=gaming
```

The host database is `%APPDATA%\com.overlayforge.desktop\overlay-forge-host.sqlite3` and stores only last-used product state and host window geometry. The original `overlay-forge.sqlite3` is never modified and remains the rollback/import source for the standalone products.

## Development

```powershell
npm.cmd install
npm.cmd run build
npm.cmd run cargo:build
```

Product repositories must be built or installed before the launcher can dispatch to them.
