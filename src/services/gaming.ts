import { invoke } from "@tauri-apps/api/core";

export type Game = {
  id: number;
  name: string;
  slug: string;
  summary: string;
  createdAt: string;
  updatedAt: string;
};

export type GameInput = {
  name: string;
  summary: string;
};

export type GameScreenshotCaptureRequest = {
  id: number;
  gameId: number;
  title: string;
  filePath: string;
  requestId: string;
  requestPath: string;
  captureStatus: string;
  capturedAt: string;
  createdAt: string;
  updatedAt: string;
};

export type GameDataLocationType = "save" | "alternate";

export type GameDataLocation = {
  id: number;
  gameId: number;
  locationType: GameDataLocationType;
  label: string;
  directoryPath: string;
  createdAt: string;
  updatedAt: string;
};

export type GearBlocksConstructionFile = {
  name: string;
  folderPath: string;
  constructionPath: string;
  byteSize: number;
};

export type GearBlocksConstructionPartSummary = {
  index: number;
  compositeIndex: number;
  compositePartIndex: number;
  assetGuid: string;
  dimensions: number[];
  behaviours: string[];
};

export type GearBlocksConstructionSummary = {
  isFrozen: boolean | null;
  isInvulnerable: boolean | null;
  compositeCount: number;
  partCount: number;
  uniqueAssetGuidCount: number;
  attachmentCount: number;
  linkCount: number;
  intersectionCount: number;
  parts: GearBlocksConstructionPartSummary[];
};

export type GearBlocksConstructionDecode = {
  name: string;
  folderPath: string;
  constructionPath: string;
  byteSize: number;
  decodedByteSize: number;
  summary: GearBlocksConstructionSummary;
  document: unknown;
};

export type GameConstruction = {
  id: number;
  gameId: number;
  name: string;
  folderPath: string;
  constructionPath: string;
  byteSize: number;
  decodedByteSize: number;
  compositeCount: number;
  partCount: number;
  uniqueAssetGuidCount: number;
  attachmentCount: number;
  linkCount: number;
  intersectionCount: number;
  isFrozen: boolean | null;
  isInvulnerable: boolean | null;
  summaryJson: string;
  documentJson: string;
  lastIndexedAt: string;
  createdAt: string;
  updatedAt: string;
};

export type GearBlocksLuaExporterInstall = {
  scriptModPath: string;
  mainLuaPath: string;
  exportDirectory: string;
};

export type GearBlocksThirdPartyDependencyStatus = {
  name: string;
  isDetected: boolean;
  isInstalledCorrectly: boolean | null;
  isActivated: boolean | null;
  installedVersion: string | null;
  expectedPath: string;
  detail: string;
  statusDetails: string[];
  logPaths: string[];
  projectUrl: string;
};

export type GearBlocksThirdPartyDependencyStatusPayload = {
  gameRoot: string;
  dependencies: GearBlocksThirdPartyDependencyStatus[];
};

export type GearBlocksRuntimeExport = {
  id: string;
  name: string;
  intendedPath: string;
  sourceLogPath: string;
  byteSize: number;
  document: unknown;
};

export type GameRuntimeConstructionExport = {
  id: number;
  gameId: number;
  exportId: string;
  name: string;
  exportKind: string;
  intendedPath: string;
  sourceLogPath: string;
  byteSize: number;
  constructionId: string;
  exportedAt: string;
  partCount: number;
  mass: number;
  isFrozen: boolean | null;
  isInvulnerable: boolean | null;
  isPlayerCharacter: boolean | null;
  documentJson: string;
  lastIndexedAt: string;
  createdAt: string;
  updatedAt: string;
};

export type GearBlocksApiType = {
  id: number;
  namespace: string;
  typeName: string;
  typeKind: string;
  docsUrl: string;
  source: string;
  sourceVersion: string;
  notes: string;
  memberCount: number;
  enumValueCount: number;
  createdAt: string;
  updatedAt: string;
};

export type GearBlocksApiMember = {
  id: number;
  typeId: number;
  typeName: string;
  memberKey: string;
  memberName: string;
  signature: string;
  memberKind: string;
  returnType: string;
  isReadable: boolean;
  isWritable: boolean;
  isInvokable: boolean;
  isMutating: boolean;
  docsUrl: string;
  source: string;
  sourceVersion: string;
  notes: string;
  createdAt: string;
  updatedAt: string;
};

export type GearBlocksApiParameter = {
  id: number;
  memberId: number;
  position: number;
  parameterName: string;
  parameterType: string;
  defaultValue: string;
  isOptional: boolean;
  createdAt: string;
  updatedAt: string;
};

export type GearBlocksApiEnumValue = {
  id: number;
  typeId: number;
  position: number;
  valueName: string;
  numericValue: string;
  luaName: string;
  description: string;
  source: string;
  sourceVersion: string;
  createdAt: string;
  updatedAt: string;
};

export type GearBlocksApiCatalog = {
  types: GearBlocksApiType[];
  members: GearBlocksApiMember[];
  parameters: GearBlocksApiParameter[];
  enumValues: GearBlocksApiEnumValue[];
};

export type GameRuntimePartApiMember = {
  id: number;
  gameId: number;
  partKey: string;
  apiMemberId: number;
  availability: string;
  sourceExportId: string;
  sourceConstructionId: string;
  firstSeenAt: string;
  lastSeenAt: string;
  namespace: string;
  typeName: string;
  typeKind: string;
  memberKey: string;
  memberName: string;
  signature: string;
  memberKind: string;
  isReadable: boolean;
  isWritable: boolean;
  isInvokable: boolean;
  isMutating: boolean;
  docsUrl: string;
  createdAt: string;
  updatedAt: string;
};

export type GearBlocksRuntimeContextSync = {
  changed: boolean;
  runtimeExportCount: number;
  runtimePartCount: number;
  constructionCount: number;
};

export type GearBlocksMarkerInput = {
  label?: string;
  reason?: string;
  x: number;
  y: number;
  z: number;
  color?: string;
  durationSeconds?: number;
  size?: number;
};

export type GearBlocksMarkerCommandResult = {
  commandCount: number;
  commandDirectory: string;
  statusDirectory: string;
};

export type GameRuntimePart = {
  id: number;
  gameId: number;
  partKey: string;
  assetGuid: string;
  assetName: string;
  displayName: string;
  fullDisplayName: string;
  category: string;
  mass: number;
  worldX: number | null;
  worldY: number | null;
  worldZ: number | null;
  localX: number | null;
  localY: number | null;
  localZ: number | null;
  worldPositionJson: string;
  localPositionJson: string;
  propertiesJson: string;
  sourceExportId: string;
  sourceConstructionId: string;
  lastSeenAt: string;
  displayImagePath: string;
  sourceImagePath: string;
  notes: string;
  createdAt: string;
  updatedAt: string;
};

export type GameCatalogObject = {
  id: number;
  gameId: number;
  name: string;
  objectType: string;
  category: string;
  categoryIcon: string;
  categoryIconPath: string;
  description: string;
  notes: string;
  tags: string;
  thumbnailPath: string;
  sourceScreenshotPath: string;
  createdAt: string;
  updatedAt: string;
};

export type GamePartCategory = {
  name: string;
  fallbackIcon: string;
  iconPath: string;
  count: number;
};

export type GameChatConversation = {
  id: number;
  gameId: number;
  title: string;
  overlayX: number | null;
  overlayY: number | null;
  createdAt: string;
  updatedAt: string;
};

export type GameChatMessageRole = "user" | "assistant" | "system";

export type GameChatMessage = {
  id: number;
  conversationId: number;
  role: GameChatMessageRole;
  content: string;
  createdAt: string;
};

export type GameChatOverlaySelection = {
  gameId: number;
  conversationId: number;
};

export function listGames() {
  return invoke<Game[]>("list_games");
}

export function createGame(input: GameInput) {
  return invoke<Game>("create_game", input);
}

export function deleteGame(id: number) {
  return invoke<void>("delete_game", { id });
}

export function listGameDataLocations(gameId: number) {
  return invoke<GameDataLocation[]>("list_game_data_locations", { gameId });
}

export function saveGameDataLocation(
  gameId: number,
  locationType: GameDataLocationType,
  directoryPath: string
) {
  return invoke<GameDataLocation>("save_game_data_location", {
    gameId,
    locationType,
    directoryPath
  });
}

export function deleteGameDataLocation(gameId: number, locationType: GameDataLocationType) {
  return invoke<void>("delete_game_data_location", {
    gameId,
    locationType
  });
}

export function listGearBlocksConstructionFiles(gameId: number) {
  return invoke<GearBlocksConstructionFile[]>("list_gearblocks_construction_files", { gameId });
}

export function listGameConstructions(gameId: number) {
  return invoke<GameConstruction[]>("list_game_constructions", { gameId });
}

export function syncGearBlocksSavedConstructions(gameId: number) {
  return invoke<GameConstruction[]>("sync_gearblocks_saved_constructions", { gameId });
}

export function syncGearBlocksRuntimeContext(gameId: number) {
  return invoke<GearBlocksRuntimeContextSync>("sync_gearblocks_runtime_context", { gameId });
}

export function importGearBlocksRuntimeContext(gameId: number) {
  return invoke<GearBlocksRuntimeContextSync>("import_gearblocks_runtime_context", { gameId });
}

export function sendGearBlocksMarkerCommands(gameId: number, markers: GearBlocksMarkerInput[]) {
  return invoke<GearBlocksMarkerCommandResult>("send_gearblocks_marker_commands", {
    gameId,
    markers
  });
}

export function clearGearBlocksMarkers(gameId: number) {
  return invoke<GearBlocksMarkerCommandResult>("clear_gearblocks_markers", { gameId });
}

export function decodeGearBlocksConstructionFile(constructionPath: string) {
  return invoke<GearBlocksConstructionDecode>("decode_gearblocks_construction_file", {
    constructionPath
  });
}

export function decodeGearBlocksConstructionFolder(folderPath: string) {
  return invoke<GearBlocksConstructionDecode>("decode_gearblocks_construction_folder", {
    folderPath
  });
}

export function installGearBlocksLuaExporter(gameId: number) {
  return invoke<GearBlocksLuaExporterInstall>("install_gearblocks_lua_exporter", { gameId });
}

export function getGearBlocksThirdPartyDependencyStatus(gameId: number) {
  return invoke<GearBlocksThirdPartyDependencyStatusPayload>(
    "get_gearblocks_third_party_dependency_status",
    { gameId }
  );
}

export function listGearBlocksRuntimeExports(gameId: number) {
  return invoke<GearBlocksRuntimeExport[]>("list_gearblocks_runtime_exports", { gameId });
}

export function listGearBlocksApiCatalog() {
  return invoke<GearBlocksApiCatalog>("list_gearblocks_api_catalog");
}

export function importGearBlocksRuntimePartIndex(gameId: number) {
  return invoke<GameRuntimePart[]>("import_gearblocks_runtime_part_index", { gameId });
}

export function importGearBlocksCatalogScreenshotImages(
  gameId: number,
  category: string,
  imagePath: string
) {
  return invoke<GameRuntimePart[]>("import_gearblocks_catalog_screenshot_images", {
    gameId,
    category,
    imagePath
  });
}

export function listGameRuntimeParts(gameId: number) {
  return invoke<GameRuntimePart[]>("list_game_runtime_parts", { gameId });
}

export function listGameRuntimePartApiMembers(gameId: number, partId: number) {
  return invoke<GameRuntimePartApiMember[]>("list_game_runtime_part_api_members", {
    gameId,
    partId
  });
}

export function setGameRuntimePartDisplayImage(
  gameId: number,
  partId: number,
  imagePath: string
) {
  return invoke<GameRuntimePart>("set_game_runtime_part_display_image", {
    gameId,
    partId,
    imagePath
  });
}

export function clearGameRuntimePartImagesForCategory(gameId: number, category: string) {
  return invoke<GameRuntimePart[]>("clear_game_runtime_part_images_for_category", {
    gameId,
    category
  });
}

export function updateGameRuntimePartNotes(gameId: number, partId: number, notes: string) {
  return invoke<GameRuntimePart>("update_game_runtime_part_notes", {
    gameId,
    partId,
    notes
  });
}

export function listGameCatalogObjects(gameId: number) {
  return invoke<GameCatalogObject[]>("list_game_catalog_objects", { gameId });
}

export function catalogGamePartsFromScreenshots(gameId: number) {
  return invoke<GameCatalogObject[]>("catalog_game_parts_from_screenshots", { gameId });
}

export function listGamePartCategories(gameId: number) {
  return invoke<GamePartCategory[]>("list_game_part_categories", { gameId });
}

export function listGameScreenshots(gameId: number) {
  return invoke<GameScreenshotCaptureRequest[]>("list_game_screenshots", { gameId });
}

export function deleteGameScreenshot(id: number) {
  return invoke<void>("delete_game_screenshot", { id });
}

export function createGameScreenshotCaptureRequest(gameId: number, timestampLabel: string) {
  return invoke<GameScreenshotCaptureRequest>("create_game_screenshot_capture_request", {
    gameId,
    timestampLabel
  });
}

export function createGameChatScreenshotCapture(gameId: number, timestampLabel: string) {
  return invoke<GameScreenshotCaptureRequest>("create_game_chat_screenshot_capture", {
    gameId,
    timestampLabel
  });
}

export function openGameChatOverlayWindow(gameId: number, conversationId: number) {
  return invoke<GameChatOverlaySelection>("open_game_chat_overlay_window", {
    gameId,
    conversationId
  });
}

export function focusGameChatOverlayWindow() {
  return invoke<boolean>("focus_game_chat_overlay_window");
}

export function toggleGameChatOverlayWindow() {
  return invoke<boolean>("toggle_game_chat_overlay_window");
}

export function getActiveGameChatOverlay() {
  return invoke<GameChatOverlaySelection | null>("get_active_game_chat_overlay");
}

export function listGameChatConversations(gameId: number) {
  return invoke<GameChatConversation[]>("list_game_chat_conversations", { gameId });
}

export function createGameChatConversation(gameId: number, title?: string) {
  return invoke<GameChatConversation>("create_game_chat_conversation", {
    gameId,
    title: title ?? null
  });
}

export function listGameChatMessages(conversationId: number) {
  return invoke<GameChatMessage[]>("list_game_chat_messages", { conversationId });
}

export function sendGameChatMessage(
  conversationId: number,
  content: string,
  screenshotIds: number[] = []
) {
  return invoke<GameChatMessage[]>("send_game_chat_message", {
    conversationId,
    content,
    screenshotIds
  });
}

export function deleteGameChatConversation(conversationId: number) {
  return invoke<void>("delete_game_chat_conversation", { conversationId });
}
