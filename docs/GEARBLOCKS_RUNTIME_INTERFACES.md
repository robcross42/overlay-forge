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

The Lua exporter records getter snapshots as `apiAttributes` entries shaped as:

```json
{
  "interface": "IPart",
  "name": "Mass",
  "valueType": "number",
  "value": 42
}
```

The parts catalog UI derives available attributes from these entries and shows only attribute names. Individual runtime construction exports store the same entries with captured values in SQLite through `game_runtime_construction_exports.document_json` and `game_runtime_parts.properties_json`.

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

Read-only getters and zero-argument getter-style methods are captured where the runtime object exposes them. Methods that require arguments are represented as available / requires-argument attributes rather than being invoked blindly. Mutating operation interfaces remain represented as support descriptors and are not called by the exporter.

## Validation Status

The `0.2.0` runtime API interface inclusion has been validated in-game against a sample vehicle by checking a random set of functional parts. The validated behavior is:

- runtime log imports create `apiAttributes` entries
- catalog part details show available API attributes without values
- DB definitions / runtime construction export context retain the expanded captured getter data

This does not mean every individual getter on every GearBlocks runtime type has been manually tested. It means the interface registry, Lua exporter snapshot path, log importer, SQLite persistence, catalog display, and chat-context handoff are wired end to end.

## Troubleshooting Notes

If a future GearBlocks version or a specific part exposes bad, missing, or surprising API data, start with these checkpoints:

- `src-tauri/src/gearblocks_api.rs`: interface descriptors, getter names, value type expectations, and method invocation policy.
- `gearblocks-script-mods/OverlayForgeConstructionExporter/main.lua`: runtime object probing, getter invocation, `apiAttributes` JSON emission, and guarded failures from GearBlocks' Lua sandbox.
- `src-tauri/src/commands.rs`: `Player.log` chunk reconstruction, runtime export import, API attribute normalization, and chat-context construction summaries.
- `src-tauri/src/db.rs`: SQLite persistence for `game_runtime_construction_exports.document_json` and `game_runtime_parts.properties_json`.
- `src/features/gaming/Gaming.tsx`: Parts detail display that intentionally lists attribute availability without exposing values in the catalog UI.
- `docs/GEARBLOCKS_CONSTRUCTION_DECODER.md`: runtime exporter workflow and the boundary between local save decoding and in-game runtime API data.

Common failure patterns to check:

- A getter appears in the catalog but has no captured value: confirm the part was exported from a live construction and that the getter is not represented as requires-argument / unavailable.
- A value is missing only for one part type: inspect the `apiAttributes` entry in the raw runtime export JSON before changing persistence or UI code.
- A GearBlocks update changes an interface name, method name, or value shape: update `gearblocks_api.rs` first, then mirror the exporter probing logic.
- Import succeeds but chat lacks the expanded definition: check whether automatic GearBlocks context sync saw a changed `Player.log` fingerprint and whether the latest runtime export is selected.
