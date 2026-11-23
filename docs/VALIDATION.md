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
- Verify requested and last-used product dispatch. The host stays resident, and the dashboard shows which products are unavailable. The legacy database stays untouched.

Linux sign-in and desktop recovery, multi-display behavior, and packaged builds require separate acceptance.

## Minimized quick-prompt regression — 23 November 2026

Minimize the quick prompt, then reopen it with Ctrl+Alt+Space and separately with Open quick prompt in the main app. Both paths must restore and focus the existing prompt, preserving its unsent draft. Repeat after Escape/hide. The shared WindowManager owns restoration; a visible minimized window must not take a focus-only path.

Agent-observed on Windows: the original failure was a minimized quick prompt. After the fix and development rebuild/restart, both Open quick prompt and Ctrl+Alt+Space restored the same minimized window with the unsent draft retained. The draft was restored manually after the development restart; this does not claim draft persistence across restart. No automated tests were added or run.
