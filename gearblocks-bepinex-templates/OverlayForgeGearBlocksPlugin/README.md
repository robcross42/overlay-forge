# Overlay Forge GearBlocks BepInEx Plugin

Source-only template for Overlay Forge's direct GearBlocks BepInEx plugin.

This plugin does not depend on GearLib. It is intended for Overlay Forge-owned in-game integration features where Lua is too limited, starting with temporary visual markers that can be placed from command files.

## Local Development

Copy this folder into the ignored workspace:

```powershell
Copy-Item -Recurse gearblocks-bepinex-templates\OverlayForgeGearBlocksPlugin gearblocks-bepinex-workspace\OverlayForgeGearBlocksPlugin
```

Rename the template files in the workspace:

```powershell
Rename-Item gearblocks-bepinex-workspace\OverlayForgeGearBlocksPlugin\OverlayForgeGearBlocksPlugin.csproj.template OverlayForgeGearBlocksPlugin.csproj
Rename-Item gearblocks-bepinex-workspace\OverlayForgeGearBlocksPlugin\Plugin.cs.template Plugin.cs
```

Copy local reference DLLs from the GearBlocks BepInEx `interop` folder into the workspace `libs` folder. Do not commit those DLLs.

Then build:

```powershell
dotnet build gearblocks-bepinex-workspace\OverlayForgeGearBlocksPlugin\OverlayForgeGearBlocksPlugin.csproj
```

Install the built DLL into GearBlocks:

```text
<GearBlocks install>\BepInEx\plugins\OverlayForgeGearBlocksPlugin\OverlayForgeGearBlocksPlugin.dll
```

## Command Bridge

At runtime the plugin watches:

```text
<GearBlocks persistent data>\OverlayForgePlugin\commands\*.json
```

Supported marker commands include camera-center and world-coordinate markers:

```json
{
  "action": "spawn_center_marker",
  "id": "example-marker",
  "label": "Front axle reference",
  "color": "#55f0c8",
  "durationSeconds": 20,
  "size": 4.0
}
```

The plugin raycasts from the center of the active camera and creates a temporary crosshair marker at the first hit point. Markers are runtime-only Unity objects and are not intended to modify saved constructions.

```json
{
  "action": "spawn_world_marker",
  "id": "rear-diff-connection",
  "label": "Rear diff input",
  "reason": "Connect the motor output shaft here.",
  "x": 2.6,
  "y": 1.1,
  "z": -0.4,
  "color": "#55f0c8",
  "durationSeconds": 45,
  "size": 4.0
}
```

World markers are the preferred bridge for chat-authored guidance because chat can use coordinates from the latest GearBlocks runtime scene export. Markers are runtime-only Unity objects with a visible center sphere plus crosshair lines and do not modify saved constructions.
