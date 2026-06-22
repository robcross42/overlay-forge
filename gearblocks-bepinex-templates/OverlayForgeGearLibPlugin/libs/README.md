# Local Reference DLLs

Copy local GearBlocks, BepInEx, Unity, and GearLib reference DLLs into this folder only in an ignored working copy.

Do not commit DLLs to this repository.

Minimum references from the GearLib setup guidance:

```text
GearLib.dll
SmashHammer.dll
SmashHammer.GearBlocks.dll
UnityEngine.dll
UnityEngine.CoreModule.dll
```

Typical source locations:

```text
C:\Program Files (x86)\Steam\steamapps\common\GearBlocks\BepInEx\plugins
C:\Program Files (x86)\Steam\steamapps\common\GearBlocks\BepInEx\interop
```

If a plugin feature touches other Unity systems, add only the required extra interop DLLs to the ignored working copy and reference them from the generated `.csproj`.
