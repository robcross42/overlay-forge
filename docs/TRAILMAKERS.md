# Trailmakers

## Purpose And Status

Trailmakers is a seeded local-first Gaming workspace intended for use while designing and iterating on vehicles. It reuses shared game chat, screenshots, the game context picker, and the generic module scaffold. Its initial focused sections are Vehicle Builds, Build Reference, and Modding.

The current entry is a planning and source-grounding scaffold. It does not yet parse blueprints, index blocks, generate Trailmakers build guides, or write/install mods.

## Source Authority

Use sources according to the question being answered:

1. For gameplay and vehicle-building information, use the user-requested primary reference: <https://trailmakers.fandom.com/wiki/Trailmakers_Wiki>.
2. For supported modding workflow and safety boundaries, use the official modding reference: <https://trailmakers.wiki.gg/wiki/Modding>.
3. For exact Lua names, signatures, types, and capabilities available in the locally installed game version, inspect `trailmakers_docs.lua` and the example mods shipped with that installation.

Do not treat cached wiki text as guaranteed current. Fetch the live page when version-sensitive guidance is requested, identify uncertainty when a page does not state its game-version coverage, and prefer the installed API definition when web documentation and the installed game disagree about mod APIs. If live retrieval is unavailable, state that current information could not be verified.

## Local Lua Boundary

The verified local mod root is:

```text
C:\Program Files (x86)\Steam\steamapps\common\Trailmakers\mods
```

The installation currently contains `trailmakers_docs.lua` plus shipped example folders including `blockmod`, `documentationmod`, `kickmod`, `spawnmod`, `spawnmodadv`, and `trackmakermod`.

The installed API definition is:

```text
C:\Program Files (x86)\Steam\steamapps\common\Trailmakers\mods\trailmakers_docs.lua
```

Treat the game installation as read-only reference material until the user requests a specific mod. Future mod projects should live in a clearly scoped mod folder with an explicit `main.lua`, should be validated against the installed API, and should never overwrite shipped examples or `trailmakers_docs.lua`.

Trailmakers supports Lua mods through its sandboxed `tm.*` API. DLL injection and game-DLL replacement are outside the supported boundary. Never introduce arbitrary Lua or shell execution through SQLite, scheduler rows, or user-editable configuration.

## Persistence And Prompt Context

Trailmakers uses stable `def_game.id_game = 4` and `obj_game.slug = 'trailmakers'`. The definition-owned `obj_game_setting` row with `setting_key = 'authority_sources'` records the two web references, the local mod root, and the installed API-definition path.

The Rust game-context path supplies this setting and its safety policy to Trailmakers chat. React renders the module sections but does not own source precedence, local paths, or modding rules.

## Deferred Work

- Trailmakers-specific vehicle/build records.
- Blueprint or vehicle-file discovery and parsing.
- Block catalogs and build-guide workflows.
- Local Lua API/example indexing.
- Writing, installing, enabling, or publishing a specific Lua mod.
