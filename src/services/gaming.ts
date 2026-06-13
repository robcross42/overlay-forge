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

export function listGames() {
  return invoke<Game[]>("list_games");
}

export function createGame(input: GameInput) {
  return invoke<Game>("create_game", input);
}

export function deleteGame(id: number) {
  return invoke<void>("delete_game", { id });
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
