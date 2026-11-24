# Overlay Forge

Overlay Forge is the resident personal AI host and launcher for independently owned desktop products and the local Journal web product.

```text
Current development version: 0.3.4
Source extraction baseline: develop@96aeb79778e148a66b8f88c2b9bcd27b8a415454
```

The app contains no product business logic. Google sign-in returns to the main Overlay Forge dashboard without opening a product. Choose Media or another product from that dashboard; typed target forwarding remains available to an already signed-in host. Ctrl+Alt+Space opens a compact prompt; closing a window hides it, and the tray provides reopen and Quit controls.

Use **Hide navigation** in the dashboard header to collapse the left pane, and **Show navigation** to restore it. The main pane scrolls independently; navigation stays in place and scrolls separately when needed in a short window. The collapsed state lasts for the current session.

## Google sign-in and family access

Google sign-in uses the Desktop OAuth client configured in the native adapter. It requests only `openid email profile` using the system browser, PKCE and a loopback callback. The configured Google Desktop client requires its application client secret during token exchange. In the app, choose **Import Google client JSON** and select the Desktop client's JSON downloaded from Google Auth Platform. Native code checks the client ID and stores only a platform-protected copy in the host database. JSON contents and secrets never enter React, command arguments or logs. The downloaded original remains under the owner's control. Import is available before sign-in for local setup/repair, or within the administrator session; an active family session cannot replace it. Repair can replace only the same compiled application client configuration and cannot change the pinned administrator or authenticate a user. Reimport a newly downloaded matching JSON here if its application secret is wrong or rotated, without deleting ChatGPT data or the admin binding. No Gmail/Calendar/Drive permissions, Google user refresh tokens or hosted service is required. The Google Cloud project owner must configure its consent audience and allow the intended accounts where required. See [Google's desktop OAuth documentation](https://developers.google.com/identity/protocols/oauth2/native-app).

Before the first run, create `%APPDATA%\com.overlayforge.desktop\google-auth.json` with the following local setup, substituting the administrator's Gmail address:

```json
{ "admin_email": "administrator@gmail.com" }
```

This local configuration stays outside source control. The first Google-verified sign-in matching that address permanently pins its issuer and subject as administrator in the local identity registry. Later email/name changes cannot grant that role to someone else, and editing the setup file cannot replace a pinned administrator. Other Google accounts become family members. ChatGPT, its connection controls, conversations, drafts, requests and approvals are administrator-only, including at the native command boundary. Existing protected ChatGPT data remains intact and becomes visible only to that administrator. Family members can launch independently owned products. Curated AI actions such as finding similar media are future work.

Sign in again after app exit; Google may reuse the browser's existing session. The local app session lasts until sign-out, account switch or process exit. ID tokens are checked when establishing it and then discarded; there is no remote revocation polling or Google service access. Signing out clears in-memory drafts and requests in both windows and retains saved protected ChatGPT data. Switching is blocked while owned work or computer cleanup remains unresolved. Signing out of Overlay Forge does not sign out of Google in the browser or disconnect the administrator's ChatGPT account.

These are local app roles, not an operating-system security boundary against administrators or arbitrary programs under the same OS user. Product databases and product-level AI integrations are independent; host sign-in does not add access control to Media or other standalone apps. Do not copy credentials or private runtime databases between devices. Linux browser sign-in is not implemented.

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
the service. Explicit requested-target forwarding also supports Journal. The host
does not start a server or store Journal data/credentials. Hosting, sign-in, and
phone-to-Windows access are deferred; this development service is local to the PC.
