# Browser server

The headless Rust server serves the same Media UI and product services to Edge, Brave and other normal browsers. It does not need Tauri, GTK, WebKit, a desktop login or a display session. Desktop mode remains separate. Browser users get the family Media library and Account controls; desktop ChatGPT/admin, module launching, settings mutation and server-file imports are not HTTP endpoints. All approved accounts share this server's Media library; it is not partitioned by Google account.

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

## Access and validation boundaries

Google code flow uses single-use, browser-bound state, PKCE and nonce; signature verification is shared with desktop authentication and checks issuer, audience, authorized party, times and email authority. Browser sessions are independent from desktop profiles. Only hashes of opaque session identifiers are stored in `browser-access.sqlite3`; session cookies are HttpOnly, SameSite=Lax and Secure for HTTPS. CSRF tokens stay in memory and every HTTP command requires the exact Origin and CSRF header. Sessions expire after eight hours or one hour without authenticated requests. Restarting the server retains valid sessions; unfinished sign-ins expire and must restart. Logout removes the local browser session; it does not sign out of Google. Google logout/revocation is not polled.

Code review, frontend build and headless/native compile checks are recorded in the PR. Tests not run (rapid-development policy). Real Google sign-in with the Web client, public TLS/proxy deployment, approved/denied/revoked account behavior, multi-browser sessions and Ubuntu runtime behavior require owner acceptance after configuration. Earlier native Google sign-in validation does not establish Web OAuth validation.
