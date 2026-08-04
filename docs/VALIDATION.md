# Overlay Forge Host Validation

Run:

```powershell
npm.cmd run build
npm.cmd run cargo:build
npm.cmd run cargo:clippy
npm.cmd run version:check
git diff --check
```

Manual checks:

- With a requested available target, the product opens directly and the host exits.
- Without a requested target, the last-used available product resumes.
- Without a valid last-used product, only the minimal picker appears.
- Unavailable products are disabled with a clear build/install message.
- The host database contains only host tables.
- The legacy application database is not opened or modified.
