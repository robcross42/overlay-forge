# GearBlocks Runtime Construction Interfaces

Overlay Forge `0.2.0` marks the documented `SmashHammer.GearBlocks.Construction` namespace reference interfaces as implemented for runtime export support.

Overlay Forge tracks the GearBlocks Lua runtime construction API from the official construction namespace documentation:

```text
https://www.gearblocksgame.com/apidoc/namespace_smash_hammer_1_1_gear_blocks_1_1_construction.html
```

The backend support structures live in:

```text
src-tauri/src/gearblocks_api.rs
```

The Lua exporter records interface/member availability as `apiAttributes` entries shaped as:

```json
{
  "interface": "IPart",
  "name": "Mass",
  "valueType": "available",
  "availability": "declared"
}
```

The parts catalog UI derives available attributes from these entries and shows only attribute names. Individual runtime construction exports store the same availability metadata in SQLite through `game_runtime_construction_exports.document_json` and `game_runtime_parts.properties_json`.

## Interfaces Covered

- `IAttachment`
- `IAttachmentOperations`
- `ICheckpoint`
- `IConstruction`
- `IConstructionOperations`
- `IControllablePartBehaviour`
- `IEnergyStore`
- `IEngineCrank`
- `IEngineCylinder`
- `IEngineDrivenCrank`
- `IEngineHead`
- `ILink`
- `ILinkNode`
- `IPart`
- `IPartAttachments`
- `IPartBehaviour`
- `IPartBehaviourOperations`
- `IPartPaint`
- `IPartProperties`
- `IPopulateConstructions`
- `IResizable`
- `ITweakables`

## Implementation Boundary

The runtime API interface registry is available for indexing and UI discovery, but API getter values are not included in default chat prompt context. The exporter records interface/member availability metadata and avoids invoking documented getter commands just to populate API values. When the exporter is installed, Overlay Forge injects a known API index from existing SQLite runtime part records; the Lua exporter uses that index first, falls back to one discovery pass for unknown parts, and caches newly discovered metadata for the current GearBlocks script session. Mutating operation interfaces remain represented as support descriptors and are not called by the exporter.

## Validation Status

The `0.2.0` runtime API interface inclusion has been validated in-game against a sample vehicle by checking a random set of functional parts. The validated behavior is:

- runtime log imports create `apiAttributes` entries
- catalog part details show available API attributes without values
- DB definitions / runtime construction export context retain indexed interface availability metadata

This does not mean every individual getter on every GearBlocks runtime type has been manually tested or executed. It means the interface registry, Lua exporter metadata path, log importer, SQLite persistence, and catalog display are wired end to end.

## Troubleshooting Notes

If a future GearBlocks version or a specific part exposes bad, missing, or surprising API data, start with these checkpoints:

- `src-tauri/src/gearblocks_api.rs`: interface descriptors, getter names, value type expectations, and method invocation policy.
- `gearblocks-script-mods/OverlayForgeConstructionExporter/main.lua`: runtime object export, interface availability metadata, `apiAttributes` JSON emission, and guarded failures from GearBlocks' Lua sandbox.
- `src-tauri/src/commands.rs`: exporter install-time known API index injection, `Player.log` chunk reconstruction, runtime export import, API attribute normalization, and chat-context construction summaries. API attributes must stay out of default prompt context unless a future explicit include control is added.
- `src-tauri/src/db.rs`: SQLite persistence for `game_runtime_construction_exports.document_json` and `game_runtime_parts.properties_json`.
- `src/features/gaming/Gaming.tsx`: Parts detail display that intentionally lists attribute availability without exposing values in the catalog UI.
- `docs/GEARBLOCKS_CONSTRUCTION_DECODER.md`: runtime exporter workflow and the boundary between local save decoding and in-game runtime API data.

Common failure patterns to check:

- A getter appears in the catalog without a value: this is expected. Catalog API metadata is availability-only by default.
- A future workflow needs live getter values: add an explicit user-controlled include/snapshot action instead of adding values to default chat context.
- Export is slow after an API metadata change: reinstall the exporter so `KNOWN_API_INDEX` is regenerated from SQLite, then export again. Unknown parts may still pay one discovery pass before being cached.
- A GearBlocks update changes an interface name or method name: update `gearblocks_api.rs` first, then mirror the exporter metadata list.
- Import succeeds but chat lacks API details: this is expected unless a future explicit prompt-inclusion control is enabled.
