# OverlayForgeGearLibPlugin Template

This folder contains the source template for a future Overlay Forge GearBlocks BepInEx plugin that depends on GearLib.

GearLib is a third-party GearBlocks modding library by KaBooMa. Overlay Forge does not vendor GearLib, BepInEx, Unity, or GearBlocks binaries. Users must install BepInEx 6 and GearLib separately before any plugin built from this template can run.

## Prerequisites

- .NET SDK 6.0
- BepInEx templates
- BepInEx 6 installed in the GearBlocks game folder
- GearLib installed in `GearBlocks\BepInEx\plugins`
- GearBlocks interop assemblies generated under `GearBlocks\BepInEx\interop`

Current local setup used while creating this template:

```powershell
dotnet new install BepInEx.Templates@2.0.0-be.4 --nuget-source https://nuget.bepinex.dev/v3/index.json
```

The older GearLib README command uses `BepInEx.Templates::2.0.0-be.1`; that package installed successfully earlier, but the local CLI threw a null-reference error when generating the IL2CPP template. Updating to `2.0.0-be.4` fixed template generation.

## Creating A Working Copy

Use an ignored local workspace so generated binaries and copied third-party DLLs do not enter the repo:

```powershell
New-Item -ItemType Directory -Force gearblocks-bepinex-workspace
Copy-Item -Recurse gearblocks-bepinex-templates\OverlayForgeGearLibPlugin gearblocks-bepinex-workspace\OverlayForgeGearLibPlugin
Rename-Item gearblocks-bepinex-workspace\OverlayForgeGearLibPlugin\OverlayForgeGearLibPlugin.csproj.template OverlayForgeGearLibPlugin.csproj
Rename-Item gearblocks-bepinex-workspace\OverlayForgeGearLibPlugin\Plugin.cs.template Plugin.cs
```

Then copy the required local references into the working copy's `libs` folder. See `libs/README.md`.

Build from the working copy:

```powershell
dotnet build gearblocks-bepinex-workspace\OverlayForgeGearLibPlugin\OverlayForgeGearLibPlugin.csproj
```

Install the built plugin DLL into:

```text
C:\Program Files (x86)\Steam\steamapps\common\GearBlocks\BepInEx\plugins\OverlayForgeGearLibPlugin\
```

## Distribution Notes

Any future published Overlay Forge GearBlocks plugin that uses this template must clearly state:

- BepInEx 6 is required.
- GearLib is required and is a third-party dependency.
- GearLib should be downloaded from the upstream GearLib project or another upstream-approved distribution channel.
- Overlay Forge does not claim ownership of GearLib.
