# Future Module Concepts

This document records modules that may be added to Overlay Forge later but are not part of the
current launcher release or committed implementation scope. Each concept requires an explicit scope,
architecture, threat, validation, and release decision before development begins.

## LanInputBridge

**Status:** Deferred module concept; not implemented.

**Ownership:** Proposed Overlay Forge module for Windows. It is not part of Overlay Forge Retirement
or any other product repository.

### Purpose

`LanInputBridge` would let a laptop temporarily provide keyboard and mouse input to a second Windows
PC over a local network. The user would continue watching and hearing the PC through its directly
attached TV. The module would not capture, stream, encode, transmit, or display video or audio and
would not be a remote-desktop or unattended-administration system.

Potential uses include controlling Windows, browsers, streaming applications, media playback,
VS Code, and local Codex sessions on the receiving PC.

### Proposed Solution

Use C#/.NET with a small WPF interface and four independently buildable projects:

- `LanInputBridge.Controller`: captures laptop input and manages deliberate control-mode entry and
  release.
- `LanInputBridge.Receiver`: accepts one authenticated controller and injects ordered input into the
  active Windows user session.
- `LanInputBridge.Protocol`: owns binary framing, versioning, authentication, sequencing, message
  validation, and connection behavior.
- `LanInputBridge.Tests`: verifies protocol, pairing, state transitions, ordering, and fail-safe input
  cleanup.

Prefer built-in .NET and Windows facilities. Add an external dependency only when it materially
improves a security-critical implementation.

### Required Experience

The receiver would show its hostname, LAN address, port, pairing state, connection state, and an
unmistakable active-control indicator. It would generate short-lived single-use pairing codes,
remember approved controllers using Windows-protected storage, support disconnect and revoke, run
in the notification area, and optionally start when the current Windows user signs in.

The controller would support LAN discovery where practical and always allow direct hostname/IP and
port entry. It would pair interactively, remember the pinned receiver securely, and expose explicit
connect/disconnect controls. A configurable global hotkey—default `Ctrl+Alt+F12`—would enter and
leave remote-control mode. The same locally handled hotkey must always restore laptop input and
must never be forwardable or suppressible by the receiver. Reconnection must never automatically
resume input redirection.

### Input Behavior And Cleanup

Keyboard transport would preserve key-down/up, modifiers, navigation and function keys, media keys,
common shortcuts, and scan codes where possible. It would not support the Windows secure-attention
sequence (`Ctrl+Alt+Delete`).

Mouse transport would use relative movement and support left, right, middle, X1, and X2 buttons plus
vertical and horizontal wheels. During remote control, local mouse input would be suppressed except
for emergency release. Pointer position and visibility must restore cleanly afterward.

Held keys and buttons must be tracked centrally. Leaving control mode, disconnecting, timing out,
shutting down, or encountering any uncertain state must release all remote input and immediately
restore full local input. No failure path may leave a logical key or button held.

The receiver would use the supported Windows `SendInput` API or an equivalent documented API. It
must preserve event order and must not emulate typing through clipboard paste. A non-elevated
receiver cannot inject into a higher-integrity application; documentation would explain this and
offer an explicit user-controlled administrator restart. The module must not bypass UAC secure
desktop, Windows security boundaries, anti-cheat controls, or application input protections.

### Connection And State Model

Use a persistent low-latency LAN connection with heartbeat/timeout detection, bounded queues, sane
rate limits, and at most one actively controlling client. Mouse movement may be coalesced over very
short intervals, but keyboard and button ordering must remain intact.

Central state machines would prevent contradictory flags:

- Receiver: `Unpaired`, `Pairing`, `Ready`, `Connected`, `Controlling`.
- Controller: `Disconnected`, `Connecting`, `Connected`, `Controlling`, `Releasing`.

Invalid transitions must fail safely with input local. Background operations use cancellation
tokens; hooks, sockets, tray icons, and unmanaged resources require deterministic disposal. Network
parsing stays off the UI thread.

### Security Baseline

- Bind to private/local interfaces by default; provide no internet relay, cloud account, NAT
  traversal, or port-forwarding feature.
- Require interactive, expiring, single-use, rate-limited pairing.
- Use authenticated encryption, preferably TLS with a receiver-generated certificate and pinned
  receiver identity.
- Authenticate the paired controller separately from transport encryption.
- Protect private keys, tokens, and trust records using DPAPI or equivalent Windows protection;
  never store them as plaintext configuration.
- Version the protocol and enforce sequence numbers, numeric ranges, message-size limits, replay
  rejection, and explicit message types.
- Reject malformed, oversized, unauthenticated, replayed, and out-of-order messages.
- Never log keystrokes, typed text, raw input payloads, pairing codes, or authentication material.
  Operational logs may record start, stop, pair, connect, disconnect, timeout, rejection reason,
  and redacted unexpected errors.
- Any Windows Firewall change must be explicit, executable/port-specific, limited to private
  networks, and never silently weaken the firewall.

### Protocol Surface

Use a compact binary protocol rather than per-event JSON. It must define at least:

- negotiation/hello and protocol version;
- pairing request/result and authenticated session establishment;
- heartbeat/acknowledgement;
- control-mode begin/end;
- keyboard down/up;
- relative mouse movement;
- mouse button down/up;
- vertical/horizontal wheel;
- release-all-input;
- disconnect/error.

Every post-authentication input message carries a monotonically increasing session sequence number.
Lengths and numeric ranges are validated before processing.

### Explicit Exclusions

The initial module would not include screen sharing, video/audio streaming, remote audio control
beyond forwarded media keys, internet access, file transfer, clipboard synchronization, chat,
simultaneous multi-PC control, mobile/macOS/Linux clients, gaming or anti-cheat work, unattended
administration, login-screen or secure-desktop control, accounts, subscriptions, telemetry, cloud
services, or automatic updates.

### Verification Plan

Automated tests must cover framing and partial reads; every message type; malformed, invalid, and
oversized input; authentication failure; expired/reused pairing codes; replay and ordering checks;
all valid and invalid state transitions; release-all behavior after every failure class; mouse
coalescing without button reordering; and reconnection without automatic control activation.

Manual verification would use Notepad, VS Code, browsers, media controls, shortcuts, modifiers,
touchpad/mouse movement, every supported button, both wheel axes, and the emergency release. Failure
tests would disconnect networking and close/restart either process while keys/buttons are held.
Verification must also cover normal versus elevated targets and rejection of an unpaired device.

### Implementation Gates

Before implementation:

1. Approve the module's place in Overlay Forge and update current host-only repository boundaries.
2. Complete a Windows input-hook/injection and authenticated-transport threat review.
3. Establish the solution, state models, protocol framing, and tests.
4. Implement pairing, protected trust storage, and encrypted transport.
5. Implement ordered receiver injection and shared release-all tracking.
6. Implement controller capture and the non-forwarded emergency-release hotkey.
7. Integrate heartbeat, timeout, cleanup, reconnection, WPF/tray UI, auto-start, and firewall
   guidance.
8. Complete security/error-path testing, the manual checklist, documentation, and packaging.

### Acceptance Criteria

The concept is implementation-ready only when the module can pair once and reconnect securely on
the same LAN; deliberately toggle local/remote input; forward the specified keyboard and mouse
events reliably; preserve direct TV video/audio; restore local control immediately on release or
failure; guarantee release of held input; reject unpaired devices; protect persistent secrets with
Windows; avoid sensitive input logging; build cleanly; pass its automated tests; and include
installation, pairing, firewall, auto-start, administrator-mode, limitations, troubleshooting,
architecture/protocol, and manual-verification documentation.
