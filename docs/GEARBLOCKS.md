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
| API | Official GearBlocks API documentation index imported into the local SQLite catalog. |
| Tools | In-game script-window tooling for scene export, builder helpers, weld helpers, and status. |
| Build Guides | Imported Markdown construction handoffs rendered in a narrow independent overlay window. |

## Context Priority

When preparing GearBlocks reasoning context, prefer sources in this order:

1. Latest imported runtime scene export.
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

User-tested player character reference:

```text
player character height = 20 GearBlocks units = 20 blocks = 200 cm
```

Use this player-character height when sizing cabins, seats, roll cages, cockpit clearance, doors, standing clearance, ladders, steps, and human-scale build features.

## Saved Construction Context

GearBlocks saved construction folders contain `construction.bytes` files. Overlay Forge decodes those files as raw DEFLATE-compressed BSON and stores compact summaries plus decoded JSON in SQLite.

Saved-file context is useful for saved part additions and removals. It does not include all human-readable runtime metadata.

See `docs/GEARBLOCKS_RUNTIME.md`.

## Runtime Scene Context

Runtime scene context comes from the Overlay Forge GearBlocks script mod. The script exports currently loaded scene parts through marked `Player.log` JSON chunks when direct Lua file writes are unavailable.

Overlay Forge imports those log chunks into SQLite and derives a semantic construction understanding model from the latest indexed export.

Runtime imports normalize successful exports into reusable definition and latest-scene instance rows. Raw full-scene export JSON is treated as a transient ingest artifact and is not retained in SQLite after a successful import.

GearBlocks chat reads the current scene through a backend scene-context service that queries normalized SQLite rows instead of depending on raw export payloads. The service exposes compact scene facts, part references, aliases, coordinates, construction groups, and observed metadata/settings/output details for prompt context.

The intended iterative workflow:

1. Load the current build in GearBlocks.
2. Run a full scene export once.
3. Keep the Overlay Forge script loaded while building.
4. Use normal GearBlocks chat Send / Enter for fast responses against the latest normalized SQLite scene context.
5. Use the chat `D↑` action when you want to include the latest already-computed scene diff in the prompt.
6. Use manual scene refresh when you want Overlay Forge to request a fresh full scene export, import it, and compute a new latest-vs-previous scene diff.

Normal chat navigation must not synchronously parse full-scene runtime logs.

## Build Guide Overlay

GearBlocks build guides are imported from Markdown handoffs or supported Steam Community guide URLs and stored locally in SQLite. Overlay Forge keeps the raw Markdown, parsed part rows, parsed assembly steps, and first-test checklist so the guide can be displayed in-game without involving chat.

Steam guide URL imports also download supported guide images into `game-screenshots/<game-slug>/build-guide-images/<guide-id>/` and create local game catalog reference rows for those files. Image bytes are not stored in SQLite.

GearBlocks chat can also generate a build guide directly. Use the Guide action in a GearBlocks chat after typing a build goal, or after a recent user message already describes the intended build. Overlay Forge sends the goal and current GearBlocks context to the backend OpenAI path, saves the generated Markdown under app data, imports it into SQLite, and refreshes the Build Guides list during the same session.

The Build Guides list can delete individual stored guides, including their parsed part and step rows. Deleting the active guide clears the remembered build-guide overlay selection.

The active or latest GearBlocks build guide is included in GearBlocks chat prompt context. A newly imported or generated guide becomes the current guide for chat until another guide is imported, generated, or selected.

Generated build guides include a Glossary section. The glossary maps real-world vehicle terms to exact GearBlocks part names when one part is enough, or to relative mini-assembly instructions when multiple GearBlocks parts are needed. Chat may use real-world terms such as axle tube, skid plate, rail, crossmember, hub, or jig, but those terms should remain grounded in the glossary.

Generated assembly guidance should use relative placement from the first reference part, named subassemblies, or temporary jigs. Numbered assembly steps should place at most three parts or blocks each, name those exact parts, and briefly state the connection type such as static, rotary, pivot, or aligned reference. Steps should not explain a whole subsystem or mention unrelated future parts. When imported or generated Markdown still names more than three part instances in one assembly step, the import parser splits that oversized step into smaller stored steps. Avoid absolute world coordinates because the user may build in the air and GearBlocks may introduce small angle drift.

The build-guide overlay is independent from the main Overlay Forge window and the game chat overlay. It is translucent, resizable, movable, always-on-top, and remembers its last position and size per guide. The overlay minimum size preserves the fixed build-step image size instead of allowing the image to shrink.

The overlay is optimized for a side-of-screen layout. It has a Build Info view for guide-level context such as goal, scale, geometry, glossary, checklist, part count, step count, and readiness, plus a Build Steps view for static Overlay Forge-generated isometric diagrams and diagram captions. The first visual iteration derives diagrams and captions from parsed step text and part rows at render time; it does not require GearBlocks script windows, BepInEx, or a persisted visual-step table.

The static diagrams highlight up to three new placement parts in a contrasting color, show already-placed reference parts in muted colors, use white labels and leader arrows for new placement callouts, and use magenta arrows and lines for connection or attachment links. Step 1 starts from the first three real guide parts only, with no already-placed references or synthesized future parts; step 2 is the first step that may render prior parts as references. The translucent diagram grid represents 10x10 cm squares and expands to cover the current step's part bounds. When no already-placed reference part exists, the grid plane is the placement anchor and new parts sit on the grid. Rendered parts use parsed guide dimensions when available, with GearBlocks size suffixes such as `Beam x3` mapped to block-grid footprints such as 1x3 squares. Known catalog parts can also use procedural part profiles when a generic box would be misleading; for example, `Engine Rear (Driven) Crank x2 & Axle` renders as a 2x2x1 crank cylinder plus a 0.5x0.5x2 axle in one 2x2x3 placement envelope. Captions stay brief by listing the active part names, connection type, and already-placed references. Current heuristic templates cover common build-guide systems such as differentials, steering, drivetrains, crankshafts, suspension corners, wheel/hub placements, frames, and generic part attachments.

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
- Build-guide-relevant API details from the latest full runtime export, including paint target colour, material, attachment type names, attachment lock/joint/interior flags, link-node types, tweakable values, control state, resize settings, and engine crank/cylinder/head relationships.
- Parent construction groups for disambiguating repeated part IDs and repeated identical parts that are attached into the same GearBlocks construction.
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

## Official API Catalog

The GearBlocks API screen can import the official Doxygen documentation from `https://www.gearblocksgame.com/apidoc/` into SQLite.

This index is for documented metadata only: type names, members, parameters, enum values, source URLs, and summaries. Overlay Forge must not execute documented API getters or mutating methods from this import unless a future explicit user-controlled runtime action is added.

Build guide chat context uses the documented API shape to explain which exported runtime surfaces matter for building: `IPart`, `IPartPaint`, `IPartProperties`, `IPartAttachments`, `IAttachment`, `ILinkNode`, `ILink`, `ITweakables`, `IResizable`, `IControllablePartBehaviour`, and combustion-engine interfaces. Actual current values still come from the latest full runtime export.

## External Dependency Boundary

BepInEx and GearLib are third-party dependencies for backlog plugin work. Overlay Forge may document setup, but must not bundle, redistribute, install, or modify them unless a future explicit license/permission review allows it.

The direct Overlay Forge BepInEx plugin does not depend on GearLib.
