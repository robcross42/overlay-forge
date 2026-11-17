# Overlay Forge

Overlay Forge is the resident personal AI host and launcher for independently owned desktop products and the local Journal web product.

```text
Current development version: 0.2.0
Source extraction baseline: develop@96aeb79778e148a66b8f88c2b9bcd27b8a415454
```

The app contains no product business logic. On startup it dispatches a requested product or resumes the last available one, then remains resident with its dashboard. Ctrl+Alt+Space opens a compact prompt; closing a window hides it, and the tray provides reopen and Quit controls.

In the Windows app, connect your account in **Connected Accounts** and grant ChatGPT plan use. Use **Chat** or the compact prompt to stream replies and continue a selected conversation. Completed conversations are protected and saved locally. Drafts and unfinished replies clear on exit. The app uses the connected ChatGPT plan with no API-key fallback. See [AI Anchor Decision and Delivery](docs/AI_ANCHOR.md) for the provider and lifecycle details.

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

The host database is `%APPDATA%\com.overlayforge.desktop\overlay-forge-host.sqlite3` and stores last-used product state, host window geometry, and platform-protected ChatGPT credentials and completed conversations. The original `overlay-forge.sqlite3` remains the rollback/import source for the standalone products.

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
