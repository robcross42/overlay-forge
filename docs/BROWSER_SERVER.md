# Browser server

The headless Rust server serves the same Media UI and product services to Edge, Brave and other normal browsers. It does not need Tauri, GTK, WebKit, a desktop login or a display session. Desktop mode remains separate. Browser users get the family Media library and Account controls. The configured browser owner also gets Local LLM Chat in the same navigation pane. Desktop ChatGPT and admin commands, module launching, settings mutation and server-file imports are not HTTP endpoints. All approved accounts share this server's Media library; it is not partitioned by Google account.

## Configure Google and HTTPS

Choose an HTTPS hostname, such as `https://app.example.com`. In Google Auth Platform create an OAuth client with application type **Web application**, rather than reusing the Desktop OAuth client. Register the exact authorized redirect URI `https://app.example.com/auth/callback`. Download its Web client JSON and store it outside the repository at `/etc/overlay-forge/google-web-client.json`, readable only by the server's OS user (`chmod 600` on Ubuntu). Do not paste its secret into chat, frontend environment variables or Git. The server reads the JSON and uses fixed Google endpoints; it never sends the client secret to the renderer. See [Google's Web server OAuth setup](https://developers.google.com/identity/protocols/oauth2/web-server).

Configure the Google consent audience/test-user list as required by your Google project's publishing state. Google's consent audience and this app's manual allowlist are independent gates: completing Google sign-in alone does not grant access.

Copy [server.example.json](../deploy/server.example.json) to `/etc/overlay-forge/server.json`, change `public_origin`, and set the paths for your installation. Relative paths resolve from the config file's directory. The server creates a private data directory on Ubuntu; an existing one must have mode `700` and belong to the service user. Keep Google JSON and data directories outside the frontend directory; startup rejects these private paths inside the public static directory. `frontend_dir` must point only to the built `dist` folder.

`media_database` is optional. By default Media uses `/var/lib/overlay-forge/overlay-forge-media.sqlite3`. To reuse an existing Ubuntu library, point it at that machine's existing product database (typically `$HOME/.local/share/com.overlayforge.media/overlay-forge-media.sqlite3`) and give the service user access to that directory. Nothing copies credentials, databases or private data from Windows. Do not point Media at the desktop host database. Metadata provider credentials use the same backend environment variables documented in the Media repository.

The server binds only to loopback (`127.0.0.1:4280` by default). Terminate public TLS with your existing reverse proxy and preserve the original Host header. Configure the app's exact public HTTPS origin; the app ignores forwarded-host/protocol headers and rejects another Host or cross-origin API writes. Do not expose the loopback port as a public plaintext service. [nginx.example.conf](../deploy/nginx.example.conf) shows TLS proxying and per-client login request limiting; supply your own domain/certificates before enabling it. The bounded OAuth-attempt pool is not a substitute for edge rate limiting. If another proxy sits in front of Nginx, configure its trusted real-IP handling there so rate limiting uses client addresses. See [Nginx rate limiting](https://nginx.org/en/docs/http/ngx_http_limit_req_module.html).

## Run on Ubuntu

From the host repository root, with a Rust toolchain, C build tools and Node/npm installed:

```sh
git submodule update --init --recursive
npm ci
npm run build
cargo run --manifest-path server/Cargo.toml -- --config /etc/overlay-forge/server.json serve
```

This is development execution, not an installer or packaged release. Leave this process running, or use your existing process supervisor under an unprivileged OS account with access only to the configuration and data it needs. Public TLS, DNS, firewall and service installation are owner-managed; no deployment is performed by this change.

## Metadata search credentials

Google sign-in does not configure metadata providers. Movie/TV search and import require `TMDB_API_READ_ACCESS_TOKEN` in the **running server process** environment. Books uses `GOOGLE_BOOKS_API_KEY` for Google Books, with Open Library fallback and optional Hardcover configuration documented in the Media repository. A missing token is a server setup issue regardless of the client browser; saved local titles and manual entry remain available.

For the current Ubuntu user service, store provider variables in a private file such as `~/.config/overlay-forge/providers.env` (directory mode 700, file mode 600), outside Git. Enter credentials locally; do not put them in frontend variables, chat or command arguments. Then run `systemctl --user edit overlay-forge-browser.service` and add this service override, replacing the home path if needed:

```ini
[Service]
EnvironmentFile=/home/robolinux/.config/overlay-forge/providers.env
```

After the file exists, run `systemctl --user daemon-reload` and `systemctl --user restart overlay-forge-browser.service`. A shell export alone does not update an already running systemd service, and Rust does not automatically load repository .env files. Missing-credential responses now tell signed-in browser users that server search is not configured without exposing provider details or secrets.

## Approve or revoke an email

Run these locally on the Ubuntu server using the same configuration and OS permissions. The server can remain running:

```sh
cargo run --manifest-path server/Cargo.toml -- --config /etc/overlay-forge/server.json approve parent@gmail.com
cargo run --manifest-path server/Cargo.toml -- --config /etc/overlay-forge/server.json list-access
cargo run --manifest-path server/Cargo.toml -- --config /etc/overlay-forge/server.json revoke parent@gmail.com
```

Approval requires a Google-authoritative Gmail or Google Workspace address. Third-party email addresses merely attached to a Google account are rejected even if Google's `email_verified` flag is true; Google is not authoritative for their ongoing mailbox ownership. See [Google's ID-token verification guidance](https://developers.google.com/identity/gsi/web/guides/verify-google-id-token). The first approved login pins the immutable Google subject. Revoking deletes all sessions for that email and every data request rechecks approval. Reapproving preserves the pinned account; it does not silently transfer access to a replacement Google identity.

Parents browse to your configured HTTPS URL, choose **Continue with Google**, and use their approved account. No desktop application installation is needed on their devices. Media links open on the requesting browser, never as programs on the server.

## Local Windows or Ubuntu browser development

Use the same server and frontend build. Put a local config outside Git with `public_origin` set to `http://localhost:4280`, `allow_http_localhost` set to `true`, writable local `data_dir`, your local `dist` path, and a Google Web client JSON registering the exact `http://localhost:4280/auth/callback` URI. Set `bind` to `127.0.0.1:4280`. Run the same command with that config path and browse to `http://localhost:4280`. On Windows use a full PowerShell path for the config; use `npm.cmd` for npm commands.

HTTP cookies without Secure are supported only in this explicit localhost mode; production requires HTTPS. `localhost` and `127.0.0.1` are different origins: browse using the configured host. `http://localhost:1420` is the desktop Vite preview, not the browser server.

## Hockey Cards

Set `hockey_inventory_owner_email` to the owner's approved Google email to enable the **Hockey Cards** tab. If the setting is omitted, browser access is disabled. The optional `hockey_inventory_database` points to an existing compatible product SQLite file. Without it, the service saves browser inventory snapshots to `comc-browser-inventory.sqlite3` in `data_dir`. An explicitly configured database is read-only. On Ubuntu the file must be private to the service user (chmod 600) and outside the frontend directory. Keep it separate from Media, browser access, and the desktop host database.

From **Hockey Cards**, open COMC Inventory Manager and sign in on COMC. In Brave DevTools under **Network**, record each page with **All** selected, search empty, and the same sort and page size. Export a sanitized HAR and choose **Import inventory HAR**. A partial snapshot shows the cards it captured when no complete snapshot is saved. A complete snapshot takes priority.

### Read inventory directly from Brave

Sign in to Overlay Forge at `http://127.0.0.1:3000`, then open and sign in to COMC Inventory Manager in another tab. In the optional **Overlay Forge COMC Read-only Probe** extension, choose **Read COMC inventory**, leave the popup open, then choose **Save to Overlay Forge**. The first save asks for local browser permission. Refresh Hockey Cards to load the saved snapshot. The extension reads the Inventory Manager pager in your existing COMC session and sends only validated inventory fields to the local importer. It stops on a repeated card, unexpected page, or after 25 pages. Save is available after a complete scan.

To prepare the unpacked extension from a checkout with the Hockey Cards submodule and Node dependencies installed, copy `scripts/comc-brave/manifest.json`, `popup.html`, and `popup.js` into a private directory. Run this from the host repository root, replacing `/path/to/extension` with that directory.

```sh
npm exec --offline -- esbuild scripts/comc-brave/read.js --bundle --format=iife --outfile=/path/to/extension/read.js
```

In `brave://extensions`, enable **Developer mode** and choose **Load unpacked**. Select the extension directory. Use the extension’s **Reload** button after updating its files.

### Saved sales

Use **Saved cards** or enter a COMC card URL or catalog ID to query saved sales. To add observations, open a COMC Hockey card’s Recent Sales page with Network recording, export a sanitized HAR, and choose **Import sales HAR**. Brave extracts the displayed card fields locally. The app sends only these fields to the signed-in browser backend. The backend keeps one identity per catalog ID, stores individual sales separately, and deduplicates repeat imports.

Select **Hockey Cards** to search all words in the saved card titles or item IDs and combine that with a year, sale status, and a USD asking-price range. Sorting keeps the applied filters. Moving a collection from Windows requires a separately reviewed logical export and import plan.

Set `hockey_sales_database` to the existing private product-owned COMC sales history database and `hockey_market_snapshot` to a sanitized captured market JSON file to enable market research. Both files must reside outside the static frontend directory. The browser owner can combine Recently Added, a hide-at-copy-count threshold, a fixed-price-only filter and a minimum asking price. The view shows the capture time and keeps unknown copy counts visible. Verified COMC card URLs and catalog IDs open saved sales. New observations can be inserted through the owner-authorized backend once an approved source is available.

## Local LLM Chat on Ubuntu

The Ubuntu runner is Ollama with the installed model tag `qwen3:8b`. The server calls Ollama at `http://127.0.0.1:11434/api/chat`. Keep port 11434 bound to Ubuntu loopback and forward only Overlay Forge through SSH. Edge uses Overlay Forge's own origin for every chat request. Prompts and replies stay in the open browser tab's memory and go to the local runner for inference. Overlay Forge does not log prompt text or save transcripts. Each request carries up to six recent exchanges for context. **New Chat**, a reload or sign-out clears the transcript.

To enable chat, set `local_llm_owner_email` in the server JSON to the owner's approved Google email. The server requires the usual approved, Google-subject-bound session and its Origin and CSRF checks. Only that configured owner gets the navigation entry and can send prompts. Omitting the field disables chat. Media approval remains the same.

For the deployed port 3000 service, use the existing Windows SSH tunnel and open `http://127.0.0.1:3000/` in your regular Edge profile. Sign in as the owner and select **Local LLM Chat** beside **Media**.

### Isolated SSH preview

From the Overlay Forge checkout on Ubuntu, use the installed Node and Rust toolchains. Put the preview settings, data and build outputs in a private directory outside Git. Leave the running server's port, config, assets and data as they are.

```sh
preview=$(mktemp -d /tmp/overlay-forge-local-llm.XXXXXX)
./node_modules/.bin/tsc
./node_modules/.bin/vite build --outDir "$preview/dist"
cargo build --offline --manifest-path server/Cargo.toml --target-dir "$preview/target"
```

Create `$preview/server.json` with `bind` set to `127.0.0.1:3001`, `public_origin` set to `http://127.0.0.1:3000`, and `allow_http_localhost` set to `true`. Port 3000 in the browser origin preserves the Google Web client's registered callback. Point `google_client_file` at the existing private Web client JSON, `data_dir` at `$preview/data`, and `frontend_dir` at `$preview/dist`. Set `local_llm_owner_email` and omit `media_database` to give the preview its own empty library. Substitute the actual preview directory in JSON paths.

```sh
"$preview/target/debug/overlay-forge-server" --config "$preview/server.json" approve <owner-email>
"$preview/target/debug/overlay-forge-server" --config "$preview/server.json" serve
```

Add a local forward for Windows `127.0.0.1:3001` to Ubuntu `127.0.0.1:3001` using the same SSH host. If your existing SSH session cannot add a forward, open a PowerShell window and run the following alongside it.

```powershell
ssh.exe -N -o ExitOnForwardFailure=yes -L 127.0.0.1:3001:127.0.0.1:3001 robolinux@10.0.0.18
```

Open a separate Edge profile from another PowerShell window. The address bar stays at the OAuth origin while this profile sends its traffic through the preview forward.

```powershell
& "${env:ProgramFiles(x86)}\Microsoft\Edge\Application\msedge.exe" "--user-data-dir=$env:TEMP\OverlayForgeLocalLlmEdge" '--host-resolver-rules=MAP 127.0.0.1:3000 127.0.0.1:3001' 'http://127.0.0.1:3000/'
```

Sign in with the configured owner, select **Local LLM Chat**, and send `Remember cedar-74619. Reply READY.` Then ask for the exact string in the same chat. Expect Qwen to recall it. **New Chat** clears the transcript. If Ollama cannot be reached, the chat reports how to check it and keeps the draft available to retry.

## Access and validation boundaries

Google code flow uses single-use, browser-bound state, PKCE and nonce; signature verification is shared with desktop authentication and checks issuer, audience, authorized party, times and email authority. Browser sessions are independent from desktop profiles. Only hashes of opaque session identifiers are stored in `browser-access.sqlite3`; session cookies are HttpOnly, SameSite=Lax and Secure for HTTPS. CSRF tokens stay in memory and every HTTP command requires the exact Origin and CSRF header. Sessions expire after eight hours or one hour without authenticated requests. Restarting the server retains valid sessions; unfinished sign-ins expire and must restart. Logout removes the local browser session; it does not sign out of Google. Google logout/revocation is not polled.

Code review, frontend build and headless/native compile checks are recorded in the PR. Tests not run (rapid-development policy). Real Google sign-in with the Web client, public TLS/proxy deployment, approved/denied/revoked account behavior, multi-browser sessions and Ubuntu runtime behavior require owner acceptance after configuration. Earlier native Google sign-in validation does not establish Web OAuth validation.
