# Overlay Forge

Overlay Forge is the private launcher for independently owned desktop products and the local Journal web product.

```text
Current stable app release: 1.0.4
Source extraction baseline: develop@96aeb79778e148a66b8f88c2b9bcd27b8a415454
```

The host contains no product business logic. On startup it launches the requested product, otherwise resumes the last successfully launched product, and shows a minimal picker only when no valid target is available.

The authoritative ownership inventory and extraction provenance are in [`docs/REPOSITORY_BOUNDARIES.md`](docs/REPOSITORY_BOUNDARIES.md).

| Product | Repository |
| --- | --- |
| Media | [`robcross42/overlay-forge-media`](https://github.com/robcross42/overlay-forge-media) |
| Gaming | [`robcross42/overlay-forge-gaming`](https://github.com/robcross42/overlay-forge-gaming) |
| Retirement | [`robcross42/overlay-forge-retirement`](https://github.com/robcross42/overlay-forge-retirement) |
| Journal | [`robcross42/overlay-forge-journal`](https://github.com/robcross42/overlay-forge-journal) |

Targets are closed Rust enum values: `media`, `gaming`, `retirement`, and `journal`. Example:

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

Desktop product repositories must be built or installed before the launcher can dispatch to them.

Journal is a responsive web product in its own repository. Run `npm.cmd run dev` from
`C:\Dev\Repos\overlay-forge-journal`, then select Journal (or use `--product=journal`).
The host checks the service identity at its fixed loopback health endpoint and opens
`http://127.0.0.1:4319` in the default browser. Use **Refresh products** after starting
the service. Requested-target and last-used resume also support Journal. The host
does not start a server or store Journal data/credentials. Hosting, sign-in, and
phone-to-Windows access are deferred; this development service is local to the PC.
