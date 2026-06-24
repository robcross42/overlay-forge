# GearBlocks Module

## Purpose

The GearBlocks module gives Overlay Forge local-first context about GearBlocks saves, runtime scenes, parts, screenshots, and optional in-game tooling.

GearBlocks support is intended for practical build assistance: part identification, construction decoding, runtime scene summaries, vehicle reasoning, screenshot context, and script-window tooling.

## Default User Data Location

GearBlocks' default user data location is:

```text
%USERPROFILE%\AppData\LocalLow\SmashHammer Games\GearBlocks\
```

Overlay Forge derives default subpaths from that root unless a feature explicitly uses a configured game data location:

```text
SavedConstructions
ScriptMods
OverlayForgeExports
OverlayForgePlugin
```

## Current Workspace Areas

| Area | Purpose |
| --- | --- |
| Home | Save location controls, alternate data location controls, construction decoder entry points, exporter install controls. |
| Chat | GearBlocks-specific chat using saved-file context and latest runtime context. |
| Parts | Runtime-indexed part catalog, part images, API availability display, notes. |
| Constructions | Saved construction folder index and decoded construction summaries. |
| Tools | In-game script-window tooling for scene export, builder helpers, weld helpers, and status. |
| Build Guides | Imported Markdown construction handoffs rendered in a narrow independent overlay window. |

## Context Priority

When preparing GearBlocks reasoning context, prefer sources in this order:

1. Latest imported runtime scene export or scene-delta snapshot.
2. Latest decoded saved construction file.
3. Runtime part index and part notes.
4. Validated GearBlocks parts catalog vocabulary.
5. Official GearBlocks API documentation.
6. Steam guides and Steam discussion posts when current public game behavior is needed.

## Scale And Units

GearBlocks developer guidance on Steam confirms:

```text
1 unit = 10 cm
```

Use centimeters and/or GearBlocks units for spacing, dimensions, alignment, movement, and vehicle scale.

Scale exceptions: the player character, wheels, and some other parts are slightly oversized for gameplay clearance. Treat those parts as gameplay-clearance exceptions rather than strict real-world scale references.

## Saved Construction Context

GearBlocks saved construction folders contain `construction.bytes` files. Overlay Forge decodes those files as raw DEFLATE-compressed BSON and stores compact summaries plus decoded JSON in SQLite.

Saved-file context is useful for saved part additions and removals. It does not include all human-readable runtime metadata.

See `docs/GEARBLOCKS_RUNTIME.md`.

## Runtime Scene Context

Runtime scene context comes from the Overlay Forge GearBlocks script mod. The script exports currently loaded scene parts through marked `Player.log` JSON chunks when direct Lua file writes are unavailable.

Overlay Forge imports those log chunks into SQLite and derives a semantic construction understanding model from the latest indexed export.

The intended iterative workflow:

1. Load the current build in GearBlocks.
2. Run a full scene export once.
3. Keep the Overlay Forge script loaded while building.
4. Let scene-delta records capture additions, moves, resizes, and removals when available.
5. Import new log additions before prompt assembly.

Normal chat navigation must not synchronously parse full-scene runtime logs.

## Build Guide Overlay

GearBlocks build guides are imported from Markdown handoffs and stored locally in SQLite. Overlay Forge keeps the raw Markdown, parsed part rows, parsed assembly steps, and first-test checklist so the guide can be displayed in-game without involving chat.

GearBlocks chat can also generate a build guide directly. Use the Guide action in a GearBlocks chat after typing a build goal, or after a recent user message already describes the intended build. Overlay Forge sends the goal and current GearBlocks context to the backend OpenAI path, saves the generated Markdown under app data, imports it into SQLite, and refreshes the Build Guides list during the same session.

The build-guide overlay is independent from the main Overlay Forge window and the game chat overlay. It is translucent, resizable, movable, always-on-top, and remembers its last position and size per guide.

The overlay is optimized for a side-of-screen layout. Parts are rendered as compact stacked rows instead of wide tables so the window can stay pinned to roughly one quarter to one third of the screen while building.

Automated construction, live scene validation against guide steps, and GearBlocks API execution are deferred.

## Friendly Part Names

The GearBlocks script includes a `Names` section for applying local friendly names to exact runtime part instances.

The workflow is:

1. Select a pivot part or aim at a part.
2. Open `Overlay Forge > Names`.
3. Enter a friendly name and click `Save Name`.
4. Overlay Forge imports the `[OverlayForgePartAlias]` event from `Player.log` and stores it in SQLite.

Chat context includes imported aliases as exact instance references. Aliases are not written back into GearBlocks parts because the public runtime API exposes part display metadata as runtime-readable values rather than a safe general-purpose user metadata field.

## Semantic Construction Model

GearBlocks chat context should summarize constructions semantically instead of treating the scene as only a visual mesh.

The derived model should include:

- Aggregated structural frame and connector inventories.
- Functional systems classified as suspension, steering, drivetrain, engine, brakes/clutches, wheels/tires, controls/data, bodywork, or unknown.
- Functional parts with inferred purpose, behaviours, link-node counts, local position, world position, and current unit size where available.
- First-class world and local coordinate columns for runtime-indexed parts.
- Coarse local chassis envelope bounds.
- Compact latest-vs-previous runtime scene diff where available.

## Visual Marker Backlog

In-game visual markers are currently disabled.

GearBlocks chat should not emit `overlay-forge-markers` blocks, request marker placement, or rely on BepInEx marker support while this feature is paused.

The marker implementation and direct BepInEx plugin templates remain in the repository for future work. Treat markers, BepInEx plugin status, and GearLib-related plugin work as backlog items until the user explicitly resumes that feature.

## Part Catalog

The GearBlocks part catalog is validated for game version `0.8.96622`.

Use `docs/GEARBLOCKS_PARTS_CATALOG.md` as the reference vocabulary for part names, categories, and practical physics roles.

Runtime API imports remain the source of truth for the user's current scene and observed runtime parts.

## External Dependency Boundary

BepInEx and GearLib are third-party dependencies for backlog plugin work. Overlay Forge may document setup, but must not bundle, redistribute, install, or modify them unless a future explicit license/permission review allows it.

The direct Overlay Forge BepInEx plugin does not depend on GearLib.
