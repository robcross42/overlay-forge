using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using UnityEngine;
using UnityEngine.Rendering;

namespace OverlayForgeGearBlocksPlugin;

internal static class PartPreviewRenderer
{
    private const int PreviewLayer = 31;
    private const float DefaultYawDegrees = 35f;
    private const float DefaultPitchDegrees = 28f;
    private const int DefaultWidth = 1024;
    private const int DefaultHeight = 576;
    private const int MaxPreviewEdgeLines = 900;
    private const int MaxGroupedPreviewRenderers = 12;
    private const float MaxPreviewBoundsDimension = 8f;
    private const float PreviewEdgeCreaseDot = 0.92f;
    private const float PreviewEdgeWidthScale = 0.004f;
    private const float MinPreviewEdgeWidth = 0.0012f;
    private const float MaxPreviewEdgeWidth = 0.0045f;

    public static PartPreviewResult CaptureCenterPartPreview(
        string commandId,
        string label,
        string renderDirectory,
        int requestedWidth,
        int requestedHeight,
        float requestedYawDegrees,
        float requestedPitchDegrees,
        float requestedPartRotationXDegrees,
        float requestedPartRotationYDegrees,
        float requestedPartRotationZDegrees
    )
    {
        Camera sourceCamera = Camera.main ?? UnityEngine.Object.FindObjectOfType<Camera>();
        if (sourceCamera == null)
            throw new InvalidOperationException("No active camera was found.");

        Ray ray = sourceCamera.ScreenPointToRay(new Vector3(Screen.width * 0.5f, Screen.height * 0.5f, 0f));
        if (!Physics.Raycast(ray, out RaycastHit hit, 1000f))
            throw new InvalidOperationException("The center raycast did not hit a renderable object.");

        Renderer[] sourceRenderers = FindSourceRenderers(hit, out PartPreviewSelection selection);

        if (sourceRenderers.Length == 0)
            throw new InvalidOperationException($"The hit object '{hit.collider.gameObject.name}' has no renderable mesh fallback. {selection.DebugSummary}");

        Bounds sourceBounds = CombinedBounds(sourceRenderers);
        int width = ClampDimension(requestedWidth, DefaultWidth);
        int height = ClampDimension(requestedHeight, DefaultHeight);
        float yawDegrees = NumberOrDefault(requestedYawDegrees, DefaultYawDegrees);
        float pitchDegrees = NumberOrDefault(requestedPitchDegrees, DefaultPitchDegrees);
        float partRotationXDegrees = NumberOrDefault(requestedPartRotationXDegrees, 0f);
        float partRotationYDegrees = NumberOrDefault(requestedPartRotationYDegrees, 0f);
        float partRotationZDegrees = NumberOrDefault(requestedPartRotationZDegrees, 0f);

        Directory.CreateDirectory(renderDirectory);
        string safeId = SanitizeFileName(string.IsNullOrWhiteSpace(commandId) ? "part-preview" : commandId);
        string outputPath = Path.Combine(renderDirectory, $"{safeId}.png");

        GameObject previewRoot = new GameObject($"Overlay Forge Part Preview {safeId}");
        GameObject cameraObject = new GameObject($"Overlay Forge Part Preview Camera {safeId}");
        GameObject lightObject = new GameObject($"Overlay Forge Part Preview Light {safeId}");
        Material edgeMaterial = null;
        RenderTexture renderTexture = null;
        RenderTexture previousActive = RenderTexture.active;

        try
        {
            previewRoot.hideFlags = HideFlags.HideAndDontSave;
            cameraObject.hideFlags = HideFlags.HideAndDontSave;
            lightObject.hideFlags = HideFlags.HideAndDontSave;
            SetLayerRecursive(previewRoot, PreviewLayer);
            previewRoot.transform.rotation = Quaternion.Euler(
                partRotationXDegrees,
                partRotationYDegrees,
                partRotationZDegrees
            );

            edgeMaterial = CreatePreviewEdgeMaterial();
            float edgeWidth = PreviewEdgeWidth(sourceBounds);
            PartPreviewRenderStats renderStats = CloneRenderersForPreview(sourceRenderers, sourceBounds.center, previewRoot, edgeMaterial, edgeWidth);
            if (renderStats.RendererCount == 0)
                throw new InvalidOperationException($"The hit object '{selection.SourceObjectName}' could not be cloned for preview rendering.");

            Light light = lightObject.AddComponent<Light>();
            light.type = LightType.Directional;
            light.intensity = 1.15f;
            lightObject.transform.rotation = Quaternion.Euler(42f, 38f, 0f);
            lightObject.layer = PreviewLayer;

            Camera camera = cameraObject.AddComponent<Camera>();
            ConfigureCamera(camera, width, height, sourceBounds, yawDegrees, pitchDegrees);

            renderTexture = new RenderTexture(width, height, 24, RenderTextureFormat.ARGB32);
            camera.targetTexture = renderTexture;
            camera.Render();

            RenderTexture.active = renderTexture;
            Texture2D texture = new Texture2D(width, height, TextureFormat.RGBA32, false);
            texture.ReadPixels(new Rect(0, 0, width, height), 0, 0);
            texture.Apply();
            File.WriteAllBytes(outputPath, texture.EncodeToPNG());
            UnityEngine.Object.Destroy(texture);

            return new PartPreviewResult(
                outputPath,
                selection.SourceObjectName,
                string.IsNullOrWhiteSpace(label) ? selection.SourceObjectName : label,
                renderStats.RendererCount,
                width,
                height,
                sourceBounds,
                selection.SelectionMode,
                selection.SelectionDistance,
                yawDegrees,
                pitchDegrees,
                partRotationXDegrees,
                partRotationYDegrees,
                partRotationZDegrees,
                renderStats.EdgeLineCount,
                renderStats.EdgeSkippedMeshCount,
                sourceRenderers
                    .Select(renderer => renderer.name ?? renderer.gameObject.name)
                    .Where(name => !string.IsNullOrWhiteSpace(name))
                    .Distinct()
                    .Take(24)
                    .ToArray()
            );
        }
        finally
        {
            RenderTexture.active = previousActive;
            if (renderTexture != null)
            {
                renderTexture.Release();
                UnityEngine.Object.Destroy(renderTexture);
            }
            UnityEngine.Object.Destroy(cameraObject);
            UnityEngine.Object.Destroy(lightObject);
            UnityEngine.Object.Destroy(previewRoot);
            if (edgeMaterial != null)
                UnityEngine.Object.Destroy(edgeMaterial);
        }
    }

    private static Renderer[] FindSourceRenderers(RaycastHit hit, out PartPreviewSelection selection)
    {
        GameObject selectedCandidate = null;
        Renderer[] selectedRenderers = Array.Empty<Renderer>();
        Bounds selectedBounds = new Bounds(hit.point, Vector3.zero);

        foreach (GameObject candidate in CandidateRoots(hit.collider))
        {
            if (IsLikelyConstructionRoot(candidate))
                continue;

            Renderer[] renderers = candidate
                .GetComponentsInChildren<Renderer>(true)
                .Where(IsRenderable)
                .ToArray();
            if (renderers.Length > 0)
            {
                Bounds bounds = CombinedBounds(renderers);
                if (!IsReasonablePreviewGroup(renderers, bounds, hit.point))
                    continue;

                if (selectedRenderers.Length == 0 || IsBetterPreviewGroup(renderers, bounds, selectedRenderers, selectedBounds, hit.point))
                {
                    selectedCandidate = candidate;
                    selectedRenderers = renderers;
                    selectedBounds = bounds;
                }
            }
        }

        if (selectedRenderers.Length > 0)
        {
            selection = new PartPreviewSelection(
                selectedCandidate.name,
                "collider_hierarchy",
                0f,
                $"candidateRoot={selectedCandidate.name}; renderers={selectedRenderers.Length}; boundsSize={FormatVector(selectedBounds.size)}"
            );
            return selectedRenderers;
        }

        Renderer[] allRenderers = AllLoadedRenderers();
        Renderer nearestRenderer = allRenderers
            .Where(IsRenderable)
            .OrderBy(renderer => BoundsDistance(renderer.bounds, hit.point))
            .FirstOrDefault();

        if (nearestRenderer == null)
        {
            selection = new PartPreviewSelection(
                hit.collider.gameObject.name,
                "none",
                0f,
                $"loadedRenderers={allRenderers.Length}; activeRenderers={UnityEngine.Object.FindObjectsOfType<Renderer>().Length}; hitPoint={FormatVector(hit.point)}"
            );
            return Array.Empty<Renderer>();
        }

        float nearestDistance = BoundsDistance(nearestRenderer.bounds, hit.point);
        Renderer[] groupedRenderers = RendererGroupForNearestHit(nearestRenderer, hit.point);
        string sourceObjectName = groupedRenderers.Length > 1 && nearestRenderer.transform.parent != null
            ? nearestRenderer.transform.parent.gameObject.name
            : nearestRenderer.gameObject.name;
        selection = new PartPreviewSelection(
            sourceObjectName,
            "nearest_renderer",
            nearestDistance,
            $"loadedRenderers={allRenderers.Length}; nearestRenderer={nearestRenderer.name}; nearestType={nearestRenderer.GetType().FullName}"
        );
        return groupedRenderers;
    }

    private static IEnumerable<GameObject> CandidateRoots(Collider collider)
    {
        HashSet<int> seen = new HashSet<int>();
        Transform current = collider.transform;
        while (current != null)
        {
            if (seen.Add(current.gameObject.GetInstanceID()))
                yield return current.gameObject;
            current = current.parent;
        }

        if (collider.attachedRigidbody != null && seen.Add(collider.attachedRigidbody.gameObject.GetInstanceID()))
            yield return collider.attachedRigidbody.gameObject;
    }

    private static Renderer[] RendererGroupForNearestHit(Renderer nearestRenderer, Vector3 hitPoint)
    {
        if (nearestRenderer.transform.parent == null)
            return new[] { nearestRenderer };

        Renderer[] bestRenderers = new[] { nearestRenderer };
        Bounds bestBounds = nearestRenderer.bounds;
        Transform current = nearestRenderer.transform.parent;

        while (current != null && !IsLikelyConstructionRoot(current.gameObject))
        {
            Renderer[] renderers = current
                .GetComponentsInChildren<Renderer>(true)
                .Where(IsRenderable)
                .ToArray();

            if (renderers.Length > 0)
            {
                Bounds bounds = CombinedBounds(renderers);
                if (IsReasonablePreviewGroup(renderers, bounds, hitPoint) &&
                    IsBetterPreviewGroup(renderers, bounds, bestRenderers, bestBounds, hitPoint))
                {
                    bestRenderers = renderers;
                    bestBounds = bounds;
                }
            }

            current = current.parent;
        }

        return bestRenderers;
    }

    private static float BoundsDistance(Bounds bounds, Vector3 point)
    {
        return Vector3.Distance(bounds.ClosestPoint(point), point);
    }

    private static bool IsRenderable(Renderer renderer)
    {
        if (renderer == null ||
            renderer.gameObject == null ||
            !renderer.gameObject.scene.IsValid() ||
            renderer.bounds.size.sqrMagnitude <= 0.0001f ||
            IsEnvironmentRenderer(renderer))
            return false;

        return true;
    }

    private static bool IsEnvironmentRenderer(Renderer renderer)
    {
        Bounds bounds = renderer.bounds;
        if (LargestBoundsDimension(bounds) > MaxPreviewBoundsDimension)
            return true;

        string objectName = renderer.gameObject != null ? renderer.gameObject.name : string.Empty;
        string rendererName = renderer.name ?? string.Empty;
        return ContainsEnvironmentName(objectName) || ContainsEnvironmentName(rendererName);
    }

    private static bool ContainsEnvironmentName(string value)
    {
        if (string.IsNullOrWhiteSpace(value))
            return false;

        return value.IndexOf("Boundary", StringComparison.OrdinalIgnoreCase) >= 0 ||
            value.IndexOf("Terrain", StringComparison.OrdinalIgnoreCase) >= 0 ||
            value.IndexOf("Sky", StringComparison.OrdinalIgnoreCase) >= 0 ||
            value.IndexOf("Water", StringComparison.OrdinalIgnoreCase) >= 0;
    }

    private static bool IsLikelyConstructionRoot(GameObject gameObject)
    {
        if (gameObject == null)
            return false;

        string name = gameObject.name ?? string.Empty;
        return name.IndexOf("Composite", StringComparison.OrdinalIgnoreCase) >= 0 ||
            name.IndexOf("Construction", StringComparison.OrdinalIgnoreCase) >= 0;
    }

    private static bool IsReasonablePreviewGroup(Renderer[] renderers, Bounds bounds, Vector3 hitPoint)
    {
        if (renderers.Length == 0 || renderers.Length > MaxGroupedPreviewRenderers)
            return false;

        if (LargestBoundsDimension(bounds) > MaxPreviewBoundsDimension)
            return false;

        float maxDistance = Math.Max(0.35f, bounds.extents.magnitude + 0.2f);
        return BoundsDistance(bounds, hitPoint) <= maxDistance;
    }

    private static bool IsBetterPreviewGroup(
        Renderer[] candidateRenderers,
        Bounds candidateBounds,
        Renderer[] currentRenderers,
        Bounds currentBounds,
        Vector3 hitPoint
    )
    {
        if (candidateRenderers.Length > currentRenderers.Length)
            return true;

        if (candidateRenderers.Length < currentRenderers.Length)
            return false;

        float candidateDistance = BoundsDistance(candidateBounds, hitPoint);
        float currentDistance = BoundsDistance(currentBounds, hitPoint);
        return candidateDistance < currentDistance;
    }

    private static float LargestBoundsDimension(Bounds bounds)
    {
        return Math.Max(bounds.size.x, Math.Max(bounds.size.y, bounds.size.z));
    }

    private static float PreviewEdgeWidth(Bounds bounds)
    {
        return Mathf.Clamp(
            LargestBoundsDimension(bounds) * PreviewEdgeWidthScale,
            MinPreviewEdgeWidth,
            MaxPreviewEdgeWidth
        );
    }

    private static Renderer[] AllLoadedRenderers()
    {
        List<Renderer> renderers = new List<Renderer>();
        HashSet<int> seen = new HashSet<int>();

        foreach (Renderer renderer in UnityEngine.Object.FindObjectsOfType<Renderer>())
        {
            if (renderer != null && seen.Add(renderer.GetInstanceID()))
                renderers.Add(renderer);
        }

        foreach (Renderer renderer in Resources.FindObjectsOfTypeAll<Renderer>())
        {
            if (renderer != null && seen.Add(renderer.GetInstanceID()))
                renderers.Add(renderer);
        }

        return renderers.ToArray();
    }

    private static Bounds CombinedBounds(IReadOnlyList<Renderer> renderers)
    {
        Bounds bounds = renderers[0].bounds;
        for (int i = 1; i < renderers.Count; i++)
            bounds.Encapsulate(renderers[i].bounds);
        return bounds;
    }

    private static PartPreviewRenderStats CloneRenderersForPreview(
        IEnumerable<Renderer> renderers,
        Vector3 sourceCenter,
        GameObject previewRoot,
        Material edgeMaterial,
        float edgeWidth
    )
    {
        int count = 0;
        int edgeLineCount = 0;
        int edgeSkippedMeshCount = 0;
        foreach (Renderer renderer in renderers)
        {
            Mesh mesh = MeshForRenderer(renderer);
            if (mesh == null)
                continue;

            GameObject clone = new GameObject($"preview-renderer-{renderer.name}");
            clone.hideFlags = HideFlags.HideAndDontSave;
            clone.layer = PreviewLayer;
            clone.transform.SetParent(previewRoot.transform, false);
            clone.transform.position = renderer.transform.position - sourceCenter;
            clone.transform.rotation = renderer.transform.rotation;
            clone.transform.localScale = renderer.transform.lossyScale;

            MeshFilter meshFilter = clone.AddComponent<MeshFilter>();
            meshFilter.sharedMesh = mesh;

            MeshRenderer meshRenderer = clone.AddComponent<MeshRenderer>();
            meshRenderer.sharedMaterials = renderer.sharedMaterials;
            meshRenderer.enabled = true;

            PartPreviewEdgeResult edgeResult = AddPreviewEdgeLines(clone, mesh, edgeMaterial, edgeWidth);
            edgeLineCount += edgeResult.LineCount;
            edgeSkippedMeshCount += edgeResult.SkippedMeshCount;
            count++;
        }
        return new PartPreviewRenderStats(count, edgeLineCount, edgeSkippedMeshCount);
    }

    private static Mesh MeshForRenderer(Renderer renderer)
    {
        if (renderer is SkinnedMeshRenderer skinnedMeshRenderer)
        {
            Mesh bakedMesh = new Mesh();
            skinnedMeshRenderer.BakeMesh(bakedMesh);
            return bakedMesh;
        }

        MeshFilter meshFilter = renderer.GetComponent<MeshFilter>() ??
            renderer.GetComponentInParent<MeshFilter>() ??
            renderer.GetComponentInChildren<MeshFilter>();
        return meshFilter != null ? meshFilter.sharedMesh : null;
    }

    private static PartPreviewEdgeResult AddPreviewEdgeLines(GameObject clone, Mesh mesh, Material edgeMaterial, float edgeWidth)
    {
        if (mesh == null || edgeMaterial == null)
            return PartPreviewEdgeResult.Empty;

        if (!mesh.isReadable)
            return PartPreviewEdgeResult.SkippedMesh;

        Vector3[] vertices;
        int[] triangles;
        try
        {
            vertices = mesh.vertices;
            triangles = mesh.triangles;
        }
        catch
        {
            return PartPreviewEdgeResult.SkippedMesh;
        }

        if (vertices == null || vertices.Length == 0 || triangles == null || triangles.Length < 3)
            return PartPreviewEdgeResult.Empty;

        Dictionary<EdgeKey, EdgeNormals> edges = new Dictionary<EdgeKey, EdgeNormals>();
        for (int index = 0; index + 2 < triangles.Length; index += 3)
        {
            int a = triangles[index];
            int b = triangles[index + 1];
            int c = triangles[index + 2];

            if (!IsValidTriangleIndex(a, vertices.Length) ||
                !IsValidTriangleIndex(b, vertices.Length) ||
                !IsValidTriangleIndex(c, vertices.Length))
                continue;

            Vector3 normal = Vector3.Cross(vertices[b] - vertices[a], vertices[c] - vertices[a]);
            if (normal.sqrMagnitude < 0.000001f)
                continue;
            normal.Normalize();

            AddEdge(edges, a, b, normal);
            AddEdge(edges, b, c, normal);
            AddEdge(edges, c, a, normal);
        }

        int addedLines = 0;
        foreach (KeyValuePair<EdgeKey, EdgeNormals> edge in edges)
        {
            if (addedLines >= MaxPreviewEdgeLines)
                break;

            if (!ShouldDrawEdge(edge.Value))
                continue;

            Vector3 start = vertices[edge.Key.A];
            Vector3 end = vertices[edge.Key.B];
            if ((end - start).sqrMagnitude < 0.000001f)
                continue;

            CreatePreviewEdgeLine(clone, start, end, edgeMaterial, edgeWidth);
            addedLines++;
        }

        return new PartPreviewEdgeResult(addedLines, 0);
    }

    private static void AddEdge(Dictionary<EdgeKey, EdgeNormals> edges, int a, int b, Vector3 normal)
    {
        EdgeKey key = new EdgeKey(a, b);
        if (!edges.TryGetValue(key, out EdgeNormals edgeNormals))
            edgeNormals = new EdgeNormals();

        edgeNormals.Add(normal);
        edges[key] = edgeNormals;
    }

    private static bool ShouldDrawEdge(EdgeNormals edge)
    {
        if (edge.Count <= 1)
            return true;

        float dot = Mathf.Abs(Vector3.Dot(edge.FirstNormal, edge.SecondNormal));
        return dot < PreviewEdgeCreaseDot;
    }

    private static void CreatePreviewEdgeLine(GameObject parent, Vector3 start, Vector3 end, Material edgeMaterial, float edgeWidth)
    {
        GameObject lineObject = new GameObject("preview-edge");
        lineObject.hideFlags = HideFlags.HideAndDontSave;
        lineObject.layer = PreviewLayer;
        lineObject.transform.SetParent(parent.transform, false);

        LineRenderer line = lineObject.AddComponent<LineRenderer>();
        line.useWorldSpace = false;
        line.positionCount = 2;
        line.SetPosition(0, start);
        line.SetPosition(1, end);
        line.startWidth = edgeWidth;
        line.endWidth = edgeWidth;
        line.numCapVertices = 0;
        line.startColor = new Color(0.045f, 0.05f, 0.06f, 0.9f);
        line.endColor = new Color(0.045f, 0.05f, 0.06f, 0.9f);
        line.material = edgeMaterial;
    }

    private static bool IsValidTriangleIndex(int index, int vertexCount)
    {
        return index >= 0 && index < vertexCount;
    }

    private static void ConfigureCamera(
        Camera camera,
        int width,
        int height,
        Bounds sourceBounds,
        float yawDegrees,
        float pitchDegrees
    )
    {
        float largestDimension = Math.Max(0.5f, Math.Max(sourceBounds.size.x, Math.Max(sourceBounds.size.y, sourceBounds.size.z)));
        float distance = Math.Max(8f, largestDimension * 5f);
        Quaternion cameraRotation = Quaternion.Euler(pitchDegrees, yawDegrees, 0f);

        camera.transform.position = cameraRotation * new Vector3(0f, 0f, -distance);
        camera.transform.LookAt(Vector3.zero, Vector3.up);
        camera.orthographic = true;
        camera.orthographicSize = Math.Max(1.25f, largestDimension);
        camera.aspect = width / (float)height;
        camera.clearFlags = CameraClearFlags.SolidColor;
        camera.backgroundColor = new Color(0.18f, 0.18f, 0.18f, 1f);
        camera.cullingMask = 1 << PreviewLayer;
        camera.nearClipPlane = 0.01f;
        camera.farClipPlane = distance * 2f;
    }

    private static Material CreateUnlitMaterial(Color color)
    {
        Shader shader =
            Shader.Find("Unlit/Color") ??
            Shader.Find("Sprites/Default") ??
            Shader.Find("Universal Render Pipeline/Unlit") ??
            Shader.Find("Standard");
        Material material = new Material(shader);
        material.color = color;
        return material;
    }

    private static Material CreatePreviewEdgeMaterial()
    {
        Shader shader =
            Shader.Find("Hidden/Internal-Colored") ??
            Shader.Find("Unlit/Color") ??
            Shader.Find("Sprites/Default") ??
            Shader.Find("Standard");
        Material material = new Material(shader);
        material.hideFlags = HideFlags.HideAndDontSave;
        material.color = new Color(0.045f, 0.05f, 0.06f, 0.9f);
        material.renderQueue = 4000;

        if (material.HasProperty("_ZWrite"))
            material.SetInt("_ZWrite", 0);
        if (material.HasProperty("_ZTest"))
            material.SetInt("_ZTest", (int)CompareFunction.LessEqual);
        if (material.HasProperty("_Cull"))
            material.SetInt("_Cull", (int)CullMode.Off);

        return material;
    }

    private static void SetLayerRecursive(GameObject gameObject, int layer)
    {
        gameObject.layer = layer;
        foreach (Transform child in gameObject.transform)
            SetLayerRecursive(child.gameObject, layer);
    }

    private static int ClampDimension(int value, int fallback)
    {
        if (value <= 0)
            return fallback;
        return Math.Max(128, Math.Min(4096, value));
    }

    private static float NumberOrDefault(float value, float fallback)
    {
        return float.IsNaN(value) || float.IsInfinity(value) ? fallback : value;
    }

    private static string FormatVector(Vector3 value)
    {
        return $"{value.x:0.###},{value.y:0.###},{value.z:0.###}";
    }

    private static string SanitizeFileName(string value)
    {
        foreach (char invalid in Path.GetInvalidFileNameChars())
            value = value.Replace(invalid, '_');
        return string.IsNullOrWhiteSpace(value) ? "part-preview" : value;
    }
}

internal sealed class PartPreviewResult
{
    public PartPreviewResult(
        string renderPath,
        string sourceObjectName,
        string label,
        int rendererCount,
        int width,
        int height,
        Bounds sourceBounds,
        string selectionMode,
        float selectionDistance,
        float cameraYawDegrees,
        float cameraPitchDegrees,
        float partRotationXDegrees,
        float partRotationYDegrees,
        float partRotationZDegrees,
        int edgeLineCount,
        int edgeSkippedMeshCount,
        string[] rendererNames
    )
    {
        RenderPath = renderPath;
        SourceObjectName = sourceObjectName;
        Label = label;
        RendererCount = rendererCount;
        Width = width;
        Height = height;
        SourceBounds = sourceBounds;
        SelectionMode = selectionMode;
        SelectionDistance = selectionDistance;
        CameraYawDegrees = cameraYawDegrees;
        CameraPitchDegrees = cameraPitchDegrees;
        PartRotationXDegrees = partRotationXDegrees;
        PartRotationYDegrees = partRotationYDegrees;
        PartRotationZDegrees = partRotationZDegrees;
        EdgeLineCount = edgeLineCount;
        EdgeSkippedMeshCount = edgeSkippedMeshCount;
        RendererNames = rendererNames ?? Array.Empty<string>();
    }

    public string RenderPath { get; }
    public string SourceObjectName { get; }
    public string Label { get; }
    public int RendererCount { get; }
    public int Width { get; }
    public int Height { get; }
    public Bounds SourceBounds { get; }
    public string SelectionMode { get; }
    public float SelectionDistance { get; }
    public float CameraYawDegrees { get; }
    public float CameraPitchDegrees { get; }
    public float PartRotationXDegrees { get; }
    public float PartRotationYDegrees { get; }
    public float PartRotationZDegrees { get; }
    public int EdgeLineCount { get; }
    public int EdgeSkippedMeshCount { get; }
    public string[] RendererNames { get; }
}

internal sealed class PartPreviewSelection
{
    public PartPreviewSelection(string sourceObjectName, string selectionMode, float selectionDistance, string debugSummary)
    {
        SourceObjectName = sourceObjectName;
        SelectionMode = selectionMode;
        SelectionDistance = selectionDistance;
        DebugSummary = debugSummary;
    }

    public string SourceObjectName { get; }
    public string SelectionMode { get; }
    public float SelectionDistance { get; }
    public string DebugSummary { get; }
}

internal sealed class PartPreviewRenderStats
{
    public PartPreviewRenderStats(int rendererCount, int edgeLineCount, int edgeSkippedMeshCount)
    {
        RendererCount = rendererCount;
        EdgeLineCount = edgeLineCount;
        EdgeSkippedMeshCount = edgeSkippedMeshCount;
    }

    public int RendererCount { get; }
    public int EdgeLineCount { get; }
    public int EdgeSkippedMeshCount { get; }
}

internal readonly struct PartPreviewEdgeResult
{
    public static readonly PartPreviewEdgeResult Empty = new PartPreviewEdgeResult(0, 0);
    public static readonly PartPreviewEdgeResult SkippedMesh = new PartPreviewEdgeResult(0, 1);

    public PartPreviewEdgeResult(int lineCount, int skippedMeshCount)
    {
        LineCount = lineCount;
        SkippedMeshCount = skippedMeshCount;
    }

    public int LineCount { get; }
    public int SkippedMeshCount { get; }
}

internal readonly struct EdgeKey : IEquatable<EdgeKey>
{
    public EdgeKey(int a, int b)
    {
        if (a < b)
        {
            A = a;
            B = b;
        }
        else
        {
            A = b;
            B = a;
        }
    }

    public int A { get; }
    public int B { get; }

    public bool Equals(EdgeKey other)
    {
        return A == other.A && B == other.B;
    }

    public override bool Equals(object obj)
    {
        return obj is EdgeKey other && Equals(other);
    }

    public override int GetHashCode()
    {
        unchecked
        {
            return (A * 397) ^ B;
        }
    }
}

internal struct EdgeNormals
{
    public int Count { get; private set; }
    public Vector3 FirstNormal { get; private set; }
    public Vector3 SecondNormal { get; private set; }

    public void Add(Vector3 normal)
    {
        if (Count == 0)
            FirstNormal = normal;
        else if (Count == 1)
            SecondNormal = normal;

        Count++;
    }
}
