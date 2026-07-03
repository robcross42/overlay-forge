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

GearBlocks coordinate axes:

- X-Axis: Controls the horizontal plane (Width). Moving a piece along this axis shifts it Left (-X) and Right (+X).
- Y-Axis: Controls the vertical plane (Height). Moving along this axis shifts it Down (-Y) and Up (+Y).
- Z-Axis: Controls the depth (Forward and Backward). In the game's building manipulators, moving along this axis shifts it Backward (-Z) and Forward (+Z).

Standard vehicle orientation: for cars and car-like vehicles, always use the Z-axis for vehicle length. The front of the car points toward +Z, the rear points toward -Z, vehicle width runs on -X/+X, and vehicle height runs on -Y/+Y.

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

GearBlocks build guides are imported from Markdown handoffs or supported Steam Community guide URLs and stored locally in SQLite. Overlay Forge keeps the raw Markdown, parsed part rows, parsed assembly steps, and first-test checklist so the guide can be displayed in-game without involving chat. Imports preserve all parsed assembly step rows so the guide can produce a complete staging manifest for the overall build.

Steam guide URL imports also download supported guide images into `game-screenshots/<game-slug>/build-guide-images/<guide-id>/` and create local game catalog reference rows for those files. Image bytes are not stored in SQLite.

GearBlocks chat can also generate a build guide directly. Use the Guide action in a GearBlocks chat after typing a build goal, or after a recent user message already describes the intended build. Overlay Forge sends the goal and current GearBlocks context to the backend OpenAI path, saves the generated Markdown under app data, imports it into SQLite, and refreshes the Build Guides list during the same session.

The Build Guides list can delete individual stored guides, including their parsed part and step rows. Deleting the active guide clears the remembered build-guide overlay selection.

The active or latest GearBlocks build guide is included in GearBlocks chat prompt context. A newly imported or generated guide becomes the current guide for chat until another guide is imported, generated, or selected.

Generated build guides include a Glossary section. The glossary maps real-world vehicle terms to exact GearBlocks part names when one part is enough, or to relative mini-assembly instructions when multiple GearBlocks parts are needed. Chat may use real-world terms such as axle tube, skid plate, rail, crossmember, hub, or jig, but those terms should remain grounded in the glossary.

Generated assembly guidance should use relative placement from the first reference part, named subassemblies, or temporary jigs. Numbered assembly steps should place at most three parts or blocks each, name those exact parts, and briefly state the connection type such as static, rotary, pivot, or aligned reference. Steps should not explain a whole subsystem or mention unrelated future parts. When imported or generated Markdown still names more than three part instances in one assembly step, the import parser splits that oversized step into smaller stored steps. Avoid absolute world coordinates because the user may build in the air and GearBlocks may introduce small angle drift.

The build-guide overlay is independent from the main Overlay Forge window and the game chat overlay. It is translucent, resizable, movable, always-on-top, and remembers its last position and size per guide. The overlay minimum size preserves the fixed build-step image size instead of allowing the image to shrink.

The overlay is optimized for a side-of-screen layout. It has a Build Info view for guide-level context such as goal, checklist, a full staging manifest, the latest in-guide Export All results, and the glossary at the bottom. The Build Steps view shows static Overlay Forge-generated isometric diagrams and diagram captions. The first visual iteration derives diagrams and captions from parsed step text and part rows at render time; it does not require GearBlocks script windows, BepInEx, or a persisted visual-step table.

The staging manifest is derived from parsed guide parts and steps, then expanded into exact staged part instances where Overlay Forge has a known build pattern. The first expansion target is the combustion-engine starter guide: high-level guide rows are expanded into a test stand, frame beams, engine core, starter drive, pulleys/fan references, optional intake pipes, and optional power/fuel support rows. Combustion-engine manifests may use parts from any relevant GearBlocks category, including Blocks, Connectors, Gears, Motors, Pipes, Fuel, and Power; they are not limited to the Combustion Engines category. Generic guides still fall back to quantity and comma-list expansion, and slash-separated catalog names such as `Straight Pipe / Corner 90 Pipe / Tee 90 Pipe` are split into individual part rows. Manifest rows are ordered for staging: duplicated paintable parts first by duplicate count descending, solo paintable parts next, duplicated unpaintable parts next, and solo unpaintable parts last. Paint slot numbers restart for each duplicated part type, so different part types can reuse slot 1 instead of consuming a global color sequence.

Default GearBlocks paint slots are the 32 in-game palette colors shown in the paint UI. Use these slot names as practical build-guide labels; they are approximate visual names from the default palette, not calibrated RGB values.

| Slot | Approximate color | Slot | Approximate color |
| --- | --- | --- | --- |
| 1 | Black | 17 | Very dark charcoal |
| 2 | Dark gray | 18 | Light gray |
| 3 | Red | 19 | Dark red |
| 4 | Orange brown | 20 | Brown |
| 5 | Yellow olive | 21 | Dark olive |
| 6 | Lime green | 22 | Olive green |
| 7 | Green | 23 | Dark green |
| 8 | Bright green | 24 | Forest green |
| 9 | Teal green | 25 | Dark teal |
| 10 | Cyan | 26 | Deep teal |
| 11 | Blue | 27 | Dark blue |
| 12 | Deep blue | 28 | Navy blue |
| 13 | Royal blue | 29 | Indigo |
| 14 | Violet | 30 | Purple |
| 15 | Magenta | 31 | Plum |
| 16 | Burgundy | 32 | Maroon |

The current two-phase build-guide workflow is:

1. Generate or import the build guide and use the Build Info staging manifest to place all required parts in GearBlocks for render capture. The displayed staging list shows only the render-staging details needed during placement: part type, instance name, paint slot, and rotation. Apply paint slots only for duplicated paintable parts. When a part needs an initial attachment target, attach it to a temporary white jig block. White jig blocks are placement aids only: do not count them as build parts, include them in final design guidance, or require them in generated build steps.
2. Click `Export All` in the build-guide overlay. Overlay Forge requests a fresh GearBlocks full-scene export, imports that new export/log data, and then shows the Latest Export section for this guide session.
3. Click `Generate Steps/Images` in the build-guide overlay. Overlay Forge uses the current in-guide export results and switches to the Build Steps view.

For the local build coordinate convention, treat north as +Z, south as -Z, east as +X, west as -X, up as +Y, and down as -Y. The first anchor part in a future matched guide should define local `0,0,0`; exported runtime coordinates remain the source for actual placed positions.

Observed `Beam x3` staging rotations use GearBlocks' in-game rotation fields directly: `0,0,0` spans north/south when facing north, `0,90,0` turns the beam into an east/west horizontal crossmember, `0,0,90` keeps the same apparent orientation as `0,0,0`, and `90,0,0` stands the beam upright. Use `90,0,0` for vertical `Beam x3` uprights in staging manifests.

The static diagrams highlight up to three new placement parts in a contrasting color, show already-placed reference parts in muted colors, use numbered white labels and leader arrows for placement callouts, and use magenta arrows and lines for connection or attachment links. Step 1 promotes the first named structural anchor from the step text, such as `Beam x3`, before other parts because later parts attach back to that anchor directly or indirectly; step 2 is the first step that may render prior parts as references. The translucent diagram grid represents 10x10 cm squares and expands to cover the current step's part bounds. The diagram grid marks X, Y, and Z axes using the GearBlocks coordinate convention: X is width left/right, Y is height down/up, and Z is depth backward/forward. Axis markers originate from the far-left rendered grid corner, such as `xMin, 0, zMax`, and show only one direction for each axis so they do not run through the build area. When no already-placed reference part exists, the grid plane is the placement anchor and new parts sit on the grid. Rendered parts use parsed guide dimensions when available, with GearBlocks size suffixes such as `Beam x3` mapped to block-grid footprints such as 1x3 squares. Known catalog parts can also use procedural part profiles when a generic box would be misleading; for example, `Engine Rear (Driven) Crank x2 & Axle` renders as a 2x2x1 crank cylinder plus a 0.5x0.5x2 axle in one 2x2x3 placement envelope. Captions stay brief by listing the active part names, connection type, and already-placed references. Current heuristic templates cover common build-guide systems such as differentials, steering, drivetrains, crankshafts, suspension corners, wheel/hub placements, frames, and generic part attachments.

Build-guide display text strips redundant size-class parentheticals from labels, captions, placement cues, and review snapshots when the GearBlocks part name already carries the useful size signal, such as `Beam x3`. Functional descriptors such as `(Driven)` or tooth counts such as `(24T)` remain visible.

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

## BepInEx Runtime Plugin

The direct Overlay Forge GearBlocks BepInEx plugin is active for Unity-side runtime experiments that are not practical through Lua alone. The current active plugin path includes a file-backed `capture_center_part_preview` command that raycasts from the active camera center, clones the hit object's Unity renderers into an isolated preview layer, optionally rotates the cloned part/object around the captured preview center on X, Y, and Z, renders only the object on a neutral background with thin dark crease/boundary edges for face readability, and writes a PNG plus status JSON under `OverlayForgePlugin`. Preview selection filters out huge environment renderers such as the boundary indicator, and edge extraction skips non-readable Unity meshes while still rendering the part material.

During part-preview iteration, the Mouse5 shortcut writes a new `capture_center_part_preview` command with an incrementing test id such as `test-part-preview-1`. This temporarily replaces Mouse5 screenshot capture; normal Gaming screenshot capture remains available through the app UI.

Validated part-preview captures can be saved as GearBlocks part render profiles in SQLite. A render profile stores a stable profile key, optional part key, canonical zero rotation, preview camera preset, selected Unity source/renderers, bounds metadata, edge metadata, latest status JSON, and latest PNG cache path. Build-guide rendering should treat the profile as the reusable definition and use explicit X/Y/Z transforms for requested placement rotations rather than storing every possible rotated image up front.

The shared GearBlocks rotation snap set is `0`, `40`, `45`, `60`, `72`, `90`, `120`, `135`, `150`, and `157.5` degrees. Build-guide rotation composition should use quaternion transforms so angled assemblies, such as 60-degree V engines, can pass parent rotation down to attached child parts without flattening the orientation into a single axis.

In-game visual markers are still disabled.

GearBlocks chat should not emit `overlay-forge-markers` blocks, request marker placement, or rely on marker support while marker rendering remains paused.

GearLib-related plugin work remains backlog. The direct Overlay Forge BepInEx plugin does not depend on GearLib.

## Part Catalog

The GearBlocks part catalog is validated for game version `0.8.96622`.

Use `docs/GEARBLOCKS_PARTS_CATALOG.md` as the reference vocabulary for part names, categories, and practical physics roles.

Runtime API imports remain the source of truth for the user's current scene and observed runtime parts.

## Official API Catalog

The GearBlocks API screen can import the official Doxygen documentation from `https://www.gearblocksgame.com/apidoc/` into SQLite.

This index is for documented metadata only: type names, members, parameters, enum values, source URLs, and summaries. Overlay Forge must not execute documented API getters or mutating methods from this import unless a future explicit user-controlled runtime action is added.

Build guide chat context uses the documented API shape to explain which exported runtime surfaces matter for building: `IPart`, `IPartPaint`, `IPartProperties`, `IPartAttachments`, `IAttachment`, `ILinkNode`, `ILink`, `ITweakables`, `IResizable`, `IControllablePartBehaviour`, and combustion-engine interfaces. Actual current values still come from the latest full runtime export.

## External Dependency Boundary

BepInEx and GearLib are third-party dependencies. Overlay Forge may document setup, but must not bundle, redistribute, install, or modify them unless a future explicit license/permission review allows it.

The direct Overlay Forge BepInEx plugin does not depend on GearLib.
