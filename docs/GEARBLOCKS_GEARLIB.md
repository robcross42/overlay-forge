# GearBlocks GearLib Integration

GearLib is a third-party GearBlocks modding library by KaBooMa. Overlay Forge may use GearLib for future BepInEx-based GearBlocks features, but Overlay Forge does not vendor GearLib or claim ownership of it.

GearLib's upstream README states that GearLib is a requirement for mods made using it and that modders should tell users about that requirement before distribution. Overlay Forge documentation and any future install UI must state that GearLib and BepInEx are separate user-installed requirements.

## Upstream References

- GearLib repository: <https://github.com/KaBooMa/GearLib>
- GearLib docs: <https://kabooma.github.io/GearLib/>
- BepInEx builds: <https://builds.bepinex.dev/projects/bepinex_be>

## Local Template

Overlay Forge stores its source template for GearLib-based plugin work here:

```text
gearblocks-bepinex-templates/OverlayForgeGearLibPlugin/
```

Overlay Forge also has a direct, non-GearLib BepInEx plugin template for owned in-game integration features:

```text
gearblocks-bepinex-templates/OverlayForgeGearBlocksPlugin/
```

Use the direct plugin path for runtime-only Overlay Forge features that do not need GearLib. See `docs/GEARBLOCKS_BEPINEX_PLUGIN.md`.

Generated working projects should be created under the ignored local folder:

```text
gearblocks-bepinex-workspace/
```

The template is intentionally source-only. Do not commit:

- `GearLib.dll`
- BepInEx DLLs
- GearBlocks interop DLLs
- Unity DLLs
- compiled plugin output

## Required User Installation

Before a GearLib-based Overlay Forge plugin can run, the user must install:

1. BepInEx 6 for GearBlocks.
2. GearLib into `GearBlocks\BepInEx\plugins`.
3. Any Overlay Forge GearBlocks plugin DLL built from this template into the GearBlocks BepInEx plugins folder.

## Development Setup

The local machine should have .NET SDK 6.0 and the BepInEx templates installed.

The current working template package is:

```powershell
dotnet new install BepInEx.Templates@2.0.0-be.4 --nuget-source https://nuget.bepinex.dev/v3/index.json
```

GearLib's README currently shows `2.0.0-be.1`; on this machine that version installed but failed to generate `bep6plugin_unity_il2cpp`. Updating to `2.0.0-be.4` resolved generation.

After copying the repo template to `gearblocks-bepinex-workspace`, copy local reference DLLs from:

```text
C:\Program Files (x86)\Steam\steamapps\common\GearBlocks\BepInEx\plugins
C:\Program Files (x86)\Steam\steamapps\common\GearBlocks\BepInEx\interop
```

Only copy them into ignored local working folders.
