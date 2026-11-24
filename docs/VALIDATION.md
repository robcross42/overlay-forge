# Overlay Forge Host Validation

## Review checks

- The Windows preview built the frontend and embedded-assets desktop application. Browser sign-in opened the default HTTPS browser, and the fixed verification prompt reached provider-confirmed completion.
- The owner confirmed that a composer follow-up streamed and used earlier turns. A subsequent restart restored the selected conversation with all three completed turns, including the automatically saved follow-up.
- The SQLite migration adds host-only tables with idempotent initialization. Completed transcripts are protected and replaced atomically. The app leaves product and legacy databases in their existing repositories.
- The review branch passes the frontend build, Windows Rust check with embedded frontend assets, version consistency check and staged diff check. Automated tests were not run for this review branch.

## Manual checks

- In **Connected Accounts**, connect ChatGPT, grant plan use, and send the fixed verification prompt. Confirm that a completed provider stream establishes verification.
- In **Chat**, leave **Use local demo** unchecked. Send a made-up word and ask for it again in the same conversation. Expect streamed replies. Use **New chat** for a separate context.
- Restart after a completed reply. Expect the latest conversation to be selected and completed turns to be available for follow-up. Drafts and unfinished replies clear when the app exits.
- Use **Stop all local work / request cleanup** during a reply. The card should retain any partial text and distinguish local stop from remote completion.
- Verify explicit product-picker launch and requested-target forwarding to an already signed-in host. The host stays resident, and the dashboard shows which products are unavailable. Google sign-in must return to the main dashboard without launching a remembered module. The legacy database stays untouched.

Sleep/wake is not validated and was explicitly declared nonblocking by the owner on 23 November 2026. Multi-monitor testing was removed from active acceptance because the owner uses one monitor; it remains unscheduled backlog [#18](https://github.com/robcross42/overlay-forge/issues/18). Linux and packaged builds require separate acceptance.

## Minimized quick-prompt regression — 23 November 2026

Minimize the quick prompt, then reopen it with Ctrl+Alt+Space and separately with Open quick prompt in the main app. Both paths must restore and focus the existing prompt, preserving its unsent draft. Repeat after Escape/hide. The shared WindowManager owns restoration; a visible minimized window must not take a focus-only path.

Agent-observed on Windows: the original failure was a minimized quick prompt. After the fix and development rebuild/restart, both Open quick prompt and Ctrl+Alt+Space restored the same minimized window with the unsent draft retained. The draft was restored manually after the development restart; this does not claim draft persistence across restart. No automated tests were added or run.

## Google identity and administrator boundary — 23 November 2026

Owner scope: one administrator-owned ChatGPT connection; family members get Google identity and product access. Family chat and curated media prompts are not implemented. Earlier main-app/Media owner validation, including tray exit, draft synchronization, demo state transitions, live Stop, and second-instance forwarding, is recorded in [#17](https://github.com/robcross42/overlay-forge/issues/17).

The frontend build, Windows Rust compile check, development build/launch, version consistency and scoped whitespace review passed for this feature. A separate native review checked session stamping, administrator enforcement, worker reservations and lock ordering. It identified a sign-in/automatic-launch race and hidden member launch errors, both corrected before handoff. Automated tests were not added or run (rapid-development policy). This is code review and compilation evidence, not real-account acceptance.

Live checks still to perform with the configured Desktop client:

- Sign in as the configured administrator; expect Administrator role, existing ChatGPT connection and saved completed conversations. No new ChatGPT sign-in should be required solely because Google identity was added.
- Sign out and check both windows: private chat/request/draft views disappear. Saved protected data stays on disk. Sign in again as administrator and confirm restored history.
- Switch to a different Google account: expect Family member role, product launcher, no host ChatGPT controls or history. Verify native hub/ChatGPT commands reject that session and stale administrator session IDs; hiding controls alone is insufficient evidence.
- Decline or cancel Google sign-in and retry. An interrupted switch should preserve the prior profile and report the error. Switching during an active request or unresolved resource cleanup must fail clearly until work is settled.
- Recheck explicit Media launch after Google sign-in, plus shortcut/Escape in signed-out and signed-in states. Successful sign-in must focus the main dashboard and leave Media closed unless it was already running. Existing standalone product processes and data are outside the host authorization boundary.

Owner sign-in succeeded with the newest Desktop client and protected JSON import; agent UI inspection confirmed Administrator status and restored host history. The owner then clarified that authentication must return to the main host. The previous automatic last-used-product launch was removed from sign-in completion, which now uses the existing WindowManager to show the main window. Live acceptance of that corrected return destination remains pending. Separately, Media's debug executable was observed loading the host frontend at port 1420 and reporting `Command identity_snapshot not found`; that Media development-port conflict remains unresolved and is not Google authentication inside Media.

Google Cloud audience/test-user configuration and actual client eligibility are owner-controlled. The original supplied client ID was replaced with the new Desktop client before the retry. No successful Google token exchange or family-account acceptance is claimed yet. App sessions last until sign-out/switch/process exit; no remote Google revocation polling is claimed.

### Desktop client exchange correction

The owner reported successful browser authorization followed by the app returning to its sign-in screen. App inspection found a rejected Google HTTP request; a bounded diagnostic request with an intentionally invalid code and the public client ID returned HTTP 400 and explicitly required the missing client secret. No real authorization code, secret or user token was used in that diagnostic. This established that the configured client needs its Desktop application secret despite Google's general parameter table describing it as optional.

The follow-up adds native JSON import with a native file picker, exact client-ID validation, bounded input, platform-protected persistence, native-only secret loading, and safe fixed diagnostics identifying token exchange versus key lookup. The frontend build and Rust compile check passed. Real JSON import and successful Google sign-in remain pending owner acceptance; no automated tests were run. Cancelled, malformed, Web-client and wrong-client imports must not replace existing protected configuration.

### Ubuntu browser launch — 23 November 2026

The Ubuntu development build opened Google Accounts in the default browser after importing the matching Desktop client JSON. A real Google sign-in returned through the loopback callback, established a verified family session, and showed the signed-in controls in Overlay Forge. This used a temporary local profile. The frontend and native debug builds and all 33 native tests passed, including HTTPS validation and forwarding the complete URL as a single GIO argument.

## Integrated Media and navigation dashboard (0.4.0)

Frontend build, host native development compilation/relaunch, Media standalone frontend/native builds, library Clippy without warnings, version consistency and scoped diff review passed. Tests not run (rapid-development policy). Process inspection found one host executable and no separate Media executable. This is not visual or live-provider acceptance.

Owner review: Home shows connection tiles without page headings, explanatory panels or chat; Chat retains its composer/history; direct Media navigation opens the library inside the host. Check movie/series/book browsing and progress against the existing Media database, navigation collapse persistence and independent scrolling. Administrator Media utilities remain available; family sessions use the library without maintenance utilities or host AI navigation. Other products retain external launch paths. Verify typed Media forwarding in an already signed-in host. The updated integrated frontend and native module have not been validated on Linux in this session; earlier 0.3.6 Linux browser-sign-in evidence does not establish that acceptance.
