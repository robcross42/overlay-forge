import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useMemo, useRef, useState } from "react";
import { ChatWorkspace } from "../../components/ChatWorkspace";
import {
  catalogGamePartsFromScreenshots,
  clearGameRuntimePartImagesForCategory,
  createGame,
  createGameChatConversation,
  createGameChatScreenshotCapture,
  createGameScreenshotCaptureRequest,
  decodeGearBlocksConstructionFile,
  installGearBlocksLuaExporter,
  importGearBlocksCatalogScreenshotImages,
  importGearBlocksRuntimePartIndex,
  deleteGameChatConversation,
  deleteGameDataLocation,
  deleteGameScreenshot,
  deleteGame,
  listGameChatConversations,
  listGameDataLocations,
  listGameChatMessages,
  listGameCatalogObjects,
  listGameConstructions,
  listGearBlocksConstructionFiles,
  listGamePartCategories,
  listGameRuntimeParts,
  listGameScreenshots,
  saveGameDataLocation,
  setGameRuntimePartDisplayImage,
  sendGameChatMessage,
  syncGearBlocksSavedConstructions,
  updateGameRuntimePartNotes
} from "../../services/gaming";
import type {
  Game,
  GameChatConversation,
  GameChatMessage,
  GameCatalogObject,
  GameConstruction,
  GameDataLocation,
  GameDataLocationType,
  GameRuntimePart,
  GearBlocksConstructionDecode,
  GearBlocksConstructionFile,
  GearBlocksRuntimeExport,
  GamePartCategory,
  GameScreenshotCaptureRequest
} from "../../services/gaming";

type GamingProps = {
  chatOverlayMode?: boolean;
  gameSections: Game[];
  navAction: GameNavAction | null;
  onEnterChatOverlayMode?: (game: Game, conversationId: number) => void;
  onGameCreated: (game: Game) => void;
  onGameChatConversationsChanged: (
    gameId: number,
    conversations: GameChatConversation[]
  ) => void;
  onGameDeleted: (gameId: number) => void;
  onExitChatOverlayMode?: () => void;
  onSelectGame: (gameId: number | null) => void;
  selectedGameId: number | null;
};

type GameNavAction = {
  type: "home" | "newChat" | "chat" | "screenshots" | "parts";
  gameId?: number;
  conversationId?: number;
  nonce: number;
};

type ScreenshotContextMenu = {
  screenshot: GameScreenshotCaptureRequest;
  x: number;
  y: number;
};

type GameView = "home" | "chat" | "screenshots" | "parts" | "constructions";
const CHAT_SCREENSHOTS_PER_PAGE = 8;
const GEARBLOCKS_PARTS_CATALOG_METADATA = {
  gameVersion: "0.8.96622",
  completeness: "Complete",
  validation: "Validated"
};
const SHOW_GEARBLOCKS_CATALOG_MAINTENANCE_CONTROLS = false;
const GAME_DATA_LOCATION_OPTIONS: Array<{
  type: GameDataLocationType;
  title: string;
  description: string;
}> = [
  {
    type: "save",
    title: "Save Location",
    description: "Primary folder for game saves or build files."
  },
  {
    type: "alternate",
    title: "Alternate Data Location",
    description: "Secondary folder for mods, exports, or other game data."
  }
];

export function Gaming({
  chatOverlayMode = false,
  gameSections,
  navAction,
  onEnterChatOverlayMode,
  onGameCreated,
  onGameChatConversationsChanged,
  onGameDeleted,
  onExitChatOverlayMode,
  onSelectGame,
  selectedGameId
}: GamingProps) {
  const [newGameName, setNewGameName] = useState("");
  const [status, setStatus] = useState(`${gameSections.length} game section(s)`);
  const [isRequestingScreenshot, setIsRequestingScreenshot] = useState(false);
  const [isCatalogingParts, setIsCatalogingParts] = useState(false);
  const [gameView, setGameView] = useState<GameView>("home");
  const [selectedPartCategory, setSelectedPartCategory] = useState("all");
  const [screenshots, setScreenshots] = useState<GameScreenshotCaptureRequest[]>([]);
  const [parts, setParts] = useState<GameCatalogObject[]>([]);
  const [runtimeParts, setRuntimeParts] = useState<GameRuntimePart[]>([]);
  const [partCategories, setPartCategories] = useState<GamePartCategory[]>([]);
  const [dataLocations, setDataLocations] = useState<GameDataLocation[]>([]);
  const [updatingDataLocationType, setUpdatingDataLocationType] =
    useState<GameDataLocationType | null>(null);
  const [constructionFiles, setConstructionFiles] = useState<GearBlocksConstructionFile[]>([]);
  const [gameConstructions, setGameConstructions] = useState<GameConstruction[]>([]);
  const [isSyncingGameConstructions, setIsSyncingGameConstructions] = useState(false);
  const [decodedConstruction, setDecodedConstruction] =
    useState<GearBlocksConstructionDecode | null>(null);
  const [decodingConstructionPath, setDecodingConstructionPath] = useState("");
  const [isInstallingLuaExporter, setIsInstallingLuaExporter] = useState(false);
  const [luaExporterPath, setLuaExporterPath] = useState("");
  const [runtimeExports, setRuntimeExports] = useState<GearBlocksRuntimeExport[]>([]);
  const [selectedRuntimeExportId, setSelectedRuntimeExportId] = useState("");
  const [isImportingRuntimeExports, setIsImportingRuntimeExports] = useState(false);
  const [isImportingCatalogScreenshot, setIsImportingCatalogScreenshot] = useState(false);
  const [isClearingRuntimeCategoryImages, setIsClearingRuntimeCategoryImages] = useState(false);
  const [updatingRuntimePartImageId, setUpdatingRuntimePartImageId] = useState<number | null>(null);
  const [selectedRuntimePartId, setSelectedRuntimePartId] = useState<number | null>(null);
  const [runtimePartNotesDraft, setRuntimePartNotesDraft] = useState("");
  const [isSavingRuntimePartNotes, setIsSavingRuntimePartNotes] = useState(false);
  const [chatConversations, setChatConversations] = useState<GameChatConversation[]>([]);
  const [selectedChatConversationId, setSelectedChatConversationId] = useState<number | null>(null);
  const [chatMessages, setChatMessages] = useState<GameChatMessage[]>([]);
  const [newChatTitle, setNewChatTitle] = useState("");
  const [chatDraft, setChatDraft] = useState("");
  const [isSendingChat, setIsSendingChat] = useState(false);
  const [isCapturingPromptScreenshot, setIsCapturingPromptScreenshot] = useState(false);
  const [selectedPromptScreenshotIds, setSelectedPromptScreenshotIds] = useState<number[]>([]);
  const [chatScreenshotPage, setChatScreenshotPage] = useState(0);
  const [isChatScreenshotPickerOpen, setIsChatScreenshotPickerOpen] = useState(false);
  const [toastMessage, setToastMessage] = useState("");
  const [screenshotContextMenu, setScreenshotContextMenu] =
    useState<ScreenshotContextMenu | null>(null);
  const [deletingScreenshotId, setDeletingScreenshotId] = useState<number | null>(null);
  const toastTimeoutRef = useRef<number | null>(null);

  const selectedGame = useMemo(
    () => gameSections.find((game) => game.id === selectedGameId) ?? null,
    [gameSections, selectedGameId]
  );
  const displayedParts = useMemo(
    () =>
      selectedPartCategory === "all"
        ? parts
        : parts.filter((part) => part.category === selectedPartCategory),
    [parts, selectedPartCategory]
  );
  const displayedRuntimeParts = useMemo(
    () =>
      selectedPartCategory === "all"
        ? runtimeParts
        : runtimeParts.filter((part) => part.category === selectedPartCategory),
    [runtimeParts, selectedPartCategory]
  );
  const selectedRuntimePart = useMemo(
    () => runtimeParts.find((part) => part.id === selectedRuntimePartId) ?? null,
    [runtimeParts, selectedRuntimePartId]
  );
  const chatScreenshotPageCount = Math.max(
    1,
    Math.ceil(screenshots.length / CHAT_SCREENSHOTS_PER_PAGE)
  );
  const visibleChatScreenshots = useMemo(
    () =>
      screenshots.slice(
        chatScreenshotPage * CHAT_SCREENSHOTS_PER_PAGE,
        chatScreenshotPage * CHAT_SCREENSHOTS_PER_PAGE + CHAT_SCREENSHOTS_PER_PAGE
      ),
    [chatScreenshotPage, screenshots]
  );
  const selectedRuntimeExport = useMemo(
    () => runtimeExports.find((runtimeExport) => runtimeExport.id === selectedRuntimeExportId),
    [runtimeExports, selectedRuntimeExportId]
  );

  useEffect(() => {
    if (
      selectedGame?.slug === "gearblocks" &&
      gameView === "parts" &&
      selectedPartCategory === "all" &&
      partCategories.length > 0
    ) {
      setSelectedPartCategory(partCategories[0].name);
    }
  }, [gameView, partCategories, selectedGame?.slug, selectedPartCategory]);

  useEffect(() => {
    if (selectedGame && selectedGame.slug !== "gearblocks" && gameView === "constructions") {
      setGameView("home");
    }
  }, [gameView, selectedGame?.id, selectedGame?.slug]);

  useEffect(() => {
    if (!selectedGame) {
      setScreenshots([]);
      setParts([]);
      setRuntimeParts([]);
      setPartCategories([]);
      setDataLocations([]);
      setUpdatingDataLocationType(null);
      setConstructionFiles([]);
      setGameConstructions([]);
      setIsSyncingGameConstructions(false);
      setDecodedConstruction(null);
      setDecodingConstructionPath("");
      setIsInstallingLuaExporter(false);
      setLuaExporterPath("");
      setRuntimeExports([]);
      setSelectedRuntimeExportId("");
      setIsImportingRuntimeExports(false);
      setIsImportingCatalogScreenshot(false);
      setIsClearingRuntimeCategoryImages(false);
      setUpdatingRuntimePartImageId(null);
      setSelectedRuntimePartId(null);
      setRuntimePartNotesDraft("");
      setIsSavingRuntimePartNotes(false);
      setChatConversations([]);
      setSelectedChatConversationId(null);
      setChatMessages([]);
      setSelectedPromptScreenshotIds([]);
      setChatScreenshotPage(0);
      setIsChatScreenshotPickerOpen(false);
      setSelectedPartCategory("all");
      setGameView("home");
      return;
    }

    listGameScreenshots(selectedGame.id)
      .then(setScreenshots)
      .catch((error) => setStatus(formatError(error)));
    listGameCatalogObjects(selectedGame.id)
      .then(setParts)
      .catch((error) => setStatus(formatError(error)));
    listGameRuntimeParts(selectedGame.id)
      .then(setRuntimeParts)
      .catch((error) => setStatus(formatError(error)));
    listGamePartCategories(selectedGame.id)
      .then(setPartCategories)
      .catch((error) => setStatus(formatError(error)));
    listGameDataLocations(selectedGame.id)
      .then((locations) => {
        setDataLocations(locations);
        if (selectedGame.slug === "gearblocks") {
          void refreshGearBlocksConstructionFiles(selectedGame.id);
          void syncGearBlocksConstructions(selectedGame.id);
        } else {
          setConstructionFiles([]);
          setGameConstructions([]);
          setDecodedConstruction(null);
        }
      })
      .catch((error) => setStatus(formatError(error)));
    listGameChatConversations(selectedGame.id)
      .then((conversations) => {
        setChatConversations(conversations);
        setSelectedChatConversationId((current) =>
          current && conversations.some((conversation) => conversation.id === current)
            ? current
            : null
        );
        onGameChatConversationsChanged(selectedGame.id, conversations);
      })
      .catch((error) => setStatus(formatError(error)));
  }, [selectedGame?.id]);

  useEffect(() => {
    if (!navAction || !navAction.gameId) {
      return;
    }

    if (navAction.gameId !== selectedGameId) {
      onSelectGame(navAction.gameId);
    }

    setGameView(navAction.type === "newChat" || navAction.type === "chat" ? "chat" : navAction.type);
    setSelectedPromptScreenshotIds([]);
    setIsChatScreenshotPickerOpen(false);

    if (navAction.type === "chat") {
      setSelectedChatConversationId(navAction.conversationId ?? null);
    } else if (navAction.type === "newChat") {
      setSelectedChatConversationId(null);
    }
  }, [navAction?.nonce]);

  useEffect(() => {
    setSelectedRuntimePartId(null);
  }, [selectedPartCategory]);

  useEffect(() => {
    setRuntimePartNotesDraft(selectedRuntimePart?.notes ?? "");
  }, [selectedRuntimePart?.id, selectedRuntimePart?.notes]);

  useEffect(() => {
    if (chatOverlayMode && (!selectedGame || gameView !== "chat" || !selectedChatConversationId)) {
      onExitChatOverlayMode?.();
    }
  }, [chatOverlayMode, selectedGame?.id, gameView, selectedChatConversationId, onExitChatOverlayMode]);

  useEffect(() => {
    setChatScreenshotPage((current) => Math.min(current, chatScreenshotPageCount - 1));
  }, [chatScreenshotPageCount]);

  useEffect(() => {
    if (!selectedChatConversationId) {
      setChatMessages([]);
      return;
    }

    listGameChatMessages(selectedChatConversationId)
      .then(setChatMessages)
      .catch((error) => setStatus(formatError(error)));
  }, [selectedChatConversationId]);

  useEffect(
    () => () => {
      if (toastTimeoutRef.current !== null) {
        window.clearTimeout(toastTimeoutRef.current);
      }
    },
    []
  );

  useEffect(() => {
    function closeContextMenuOnEscape(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setScreenshotContextMenu(null);
      }
    }

    window.addEventListener("keydown", closeContextMenuOnEscape);
    window.addEventListener("resize", closeScreenshotContextMenu);
    return () => {
      window.removeEventListener("keydown", closeContextMenuOnEscape);
      window.removeEventListener("resize", closeScreenshotContextMenu);
    };
  }, []);

  function showToast(message: string) {
    setToastMessage(message);
    if (toastTimeoutRef.current !== null) {
      window.clearTimeout(toastTimeoutRef.current);
    }
    toastTimeoutRef.current = window.setTimeout(() => {
      setToastMessage("");
      toastTimeoutRef.current = null;
    }, 1800);
  }

  async function addGameSection() {
    const name = newGameName.trim();

    if (!name) {
      setStatus("Game name is required");
      return;
    }

    if (gameSections.some((game) => game.name.toLowerCase() === name.toLowerCase())) {
      setStatus("Game section already exists");
      return;
    }

    try {
      const created = await createGame({
        name,
        summary: `Game-specific workspace section for ${name} planning, object cataloging, references, and screenshots.`
      });
      onGameCreated(created);
      onSelectGame(created.id);
      setNewGameName("");
      setStatus("Game section added");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function removeSelectedGame() {
    if (!selectedGame) {
      setStatus("Select a game section first");
      return;
    }

    try {
      await deleteGame(selectedGame.id);
      onGameDeleted(selectedGame.id);
      setStatus("Game section removed");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function requestScreenshotCapture(game: Game) {
    setIsRequestingScreenshot(true);
    try {
      const request = await createGameScreenshotCaptureRequest(
        game.id,
        screenshotTimestampLabel(new Date())
      );
      setScreenshots((current) => [request, ...current.filter((item) => item.id !== request.id)]);
      setChatScreenshotPage(0);
      setStatus(`Screenshot saved: ${request.requestId}`);
      showToast("Successful");
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setIsRequestingScreenshot(false);
    }
  }

  async function capturePromptScreenshot(game: Game) {
    if (!selectedChatConversationId || isCapturingPromptScreenshot) {
      return;
    }

    setIsCapturingPromptScreenshot(true);
    setStatus("Capturing screenshot");
    try {
      const request = await createGameChatScreenshotCapture(
        game.id,
        screenshotTimestampLabel(new Date())
      );
      setScreenshots((current) => [request, ...current.filter((item) => item.id !== request.id)]);
      setSelectedPromptScreenshotIds((current) =>
        current.includes(request.id) ? current : [...current, request.id]
      );
      setChatScreenshotPage(0);
      setStatus("Screenshot added to current prompt");
    } catch (error) {
      setStatus(formatError(error));
    } finally {
      setIsCapturingPromptScreenshot(false);
    }
  }

  async function catalogParts(game: Game) {
    setIsCatalogingParts(true);
    try {
      const catalogedParts = await catalogGamePartsFromScreenshots(game.id);
      const categories = await listGamePartCategories(game.id);
      setParts(catalogedParts);
      setPartCategories(categories);
      setSelectedPartCategory("all");
      setGameView("parts");
      setStatus(`${catalogedParts.length} parts cataloged`);
      showToast("Successful");
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setIsCatalogingParts(false);
    }
  }

  async function setRuntimePartImage(game: Game, part: GameRuntimePart) {
    if (updatingRuntimePartImageId !== null) {
      return;
    }

    try {
      const selected = await open({
        multiple: false,
        directory: false,
        title: `Select display image for ${part.displayName || part.assetName || "part"}`,
        filters: [
          {
            name: "Images",
            extensions: ["png", "jpg", "jpeg", "webp", "bmp"]
          }
        ]
      });
      const imagePath = Array.isArray(selected) ? selected[0] : selected;
      if (!imagePath) {
        return;
      }

      setUpdatingRuntimePartImageId(part.id);
      const updated = await setGameRuntimePartDisplayImage(game.id, part.id, imagePath);
      setRuntimeParts((current) =>
        current.map((runtimePart) => (runtimePart.id === updated.id ? updated : runtimePart))
      );
      setStatus(`Display image set for ${updated.displayName || updated.assetName || updated.partKey}`);
      showToast("Successful");
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setUpdatingRuntimePartImageId(null);
    }
  }

  async function saveRuntimePartNotes(game: Game, part: GameRuntimePart) {
    setIsSavingRuntimePartNotes(true);
    try {
      const updated = await updateGameRuntimePartNotes(game.id, part.id, runtimePartNotesDraft);
      setRuntimeParts((current) =>
        current.map((runtimePart) => (runtimePart.id === updated.id ? updated : runtimePart))
      );
      setStatus(`Notes saved for ${updated.displayName || updated.assetName || updated.partKey}`);
      showToast("Successful");
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setIsSavingRuntimePartNotes(false);
    }
  }

  async function createGameChat() {
    if (!selectedGame) {
      setStatus("Select a game first");
      return;
    }

    const title = newChatTitle.trim();
    if (!title) {
      setStatus("Chat title is required");
      return;
    }

    try {
      const created = await createGameChatConversation(selectedGame.id, title);
      const nextConversations = [created, ...chatConversations];
      setChatConversations(nextConversations);
      onGameChatConversationsChanged(selectedGame.id, nextConversations);
      setSelectedChatConversationId(created.id);
      setChatMessages([]);
      setNewChatTitle("");
      setGameView("chat");
      setStatus("Chat created");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function sendGameChat() {
    if (!selectedChatConversationId) {
      setStatus("Create or select a chat first");
      return;
    }

    const content = chatDraft.trim();
    if (!content) {
      setStatus("Message is required");
      return;
    }

    const pendingMessage: GameChatMessage = {
      id: Date.now() * -1,
      conversationId: selectedChatConversationId,
      role: "user",
      content,
      createdAt: new Date().toISOString()
    };

    setChatDraft("");
    setIsSendingChat(true);
    setChatMessages((current) => [...current, pendingMessage]);
    setStatus("Waiting for OpenAI");

    try {
      const nextMessages = await sendGameChatMessage(
        selectedChatConversationId,
        content,
        selectedPromptScreenshotIds
      );
      setChatMessages(nextMessages);
      setSelectedPromptScreenshotIds([]);
      setIsChatScreenshotPickerOpen(false);
      if (selectedGame) {
        const nextConversations = await listGameChatConversations(selectedGame.id);
        setChatConversations(nextConversations);
        onGameChatConversationsChanged(selectedGame.id, nextConversations);
      }
      setStatus("Response saved");
    } catch (error) {
      const persistedMessages = await listGameChatMessages(selectedChatConversationId).catch(
        () => null
      );
      if (persistedMessages) {
        setChatMessages(persistedMessages);
      } else {
        setChatMessages((current) => current.filter((message) => message.id !== pendingMessage.id));
      }
      setStatus(formatError(error));
    } finally {
      setIsSendingChat(false);
    }
  }

  async function removeGameChat(conversation: GameChatConversation) {
    if (!window.confirm(`Delete chat "${conversation.title}"?`)) {
      return;
    }

    try {
      await deleteGameChatConversation(conversation.id);
      const nextConversations = chatConversations.filter((item) => item.id !== conversation.id);
      setChatConversations(nextConversations);
      if (selectedGame) {
        onGameChatConversationsChanged(selectedGame.id, nextConversations);
      }
      setSelectedChatConversationId((current) => (current === conversation.id ? null : current));
      setSelectedPromptScreenshotIds([]);
      setIsChatScreenshotPickerOpen(false);
      setStatus("Chat deleted");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  function openChatDefaults() {
    setGameView("chat");
    setSelectedChatConversationId(null);
    setSelectedPromptScreenshotIds([]);
    setIsChatScreenshotPickerOpen(false);
  }

  async function browseGameDataLocation(game: Game, locationType: GameDataLocationType) {
    setUpdatingDataLocationType(locationType);
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: `Select ${gameDataLocationTitle(locationType)}`
      });

      if (!selected || Array.isArray(selected)) {
        setStatus("Directory selection cancelled");
        return;
      }

      const saved = await saveGameDataLocation(game.id, locationType, selected);
      setDataLocations((current) => upsertGameDataLocation(current, saved));
      if (game.slug === "gearblocks") {
        void refreshGearBlocksConstructionFiles(game.id);
      }
      setStatus(`${gameDataLocationTitle(locationType)} set`);
      showToast("Successful");
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setUpdatingDataLocationType(null);
    }
  }

  async function clearGameDataLocation(game: Game, locationType: GameDataLocationType) {
    setUpdatingDataLocationType(locationType);
    try {
      await deleteGameDataLocation(game.id, locationType);
      setDataLocations((current) =>
        current.filter((location) => location.locationType !== locationType)
      );
      if (game.slug === "gearblocks") {
        void refreshGearBlocksConstructionFiles(game.id);
      }
      setStatus(`${gameDataLocationTitle(locationType)} cleared`);
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setUpdatingDataLocationType(null);
    }
  }

  async function refreshGearBlocksConstructionFiles(gameId: number) {
    try {
      const files = await listGearBlocksConstructionFiles(gameId);
      setConstructionFiles(files);
      setDecodedConstruction((current) =>
        current && files.some((file) => file.constructionPath === current.constructionPath)
          ? current
          : null
      );
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function refreshGameConstructions(gameId: number) {
    try {
      const constructions = await listGameConstructions(gameId);
      setGameConstructions(constructions);
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function syncGearBlocksConstructions(gameId: number) {
    setIsSyncingGameConstructions(true);
    try {
      const constructions = await syncGearBlocksSavedConstructions(gameId);
      setGameConstructions(constructions);
      setStatus(`Indexed ${constructions.length} GearBlocks construction(s)`);
    } catch (error) {
      setStatus(formatError(error));
      await refreshGameConstructions(gameId);
    } finally {
      setIsSyncingGameConstructions(false);
    }
  }

  async function decodeGearBlocksConstruction(construction: GearBlocksConstructionFile) {
    setDecodingConstructionPath(construction.constructionPath);
    try {
      const decoded = await decodeGearBlocksConstructionFile(construction.constructionPath);
      setDecodedConstruction(decoded);
      setStatus(`Decoded ${decoded.name}`);
      showToast("Successful");
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setDecodingConstructionPath("");
    }
  }

  async function installGearBlocksExporter(game: Game) {
    setIsInstallingLuaExporter(true);
    try {
      const install = await installGearBlocksLuaExporter(game.id);
      setLuaExporterPath(install.scriptModPath);
      setStatus(`Lua exporter installed: ${install.exportDirectory}`);
      showToast("Successful");
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setIsInstallingLuaExporter(false);
    }
  }

  async function importGearBlocksRuntimeExports(game: Game) {
    setIsImportingRuntimeExports(true);
    try {
      const indexedParts = await importGearBlocksRuntimePartIndex(game.id);
      const categories = await listGamePartCategories(game.id);
      setRuntimeParts(indexedParts);
      setPartCategories(categories);
      setRuntimeExports([]);
      setSelectedRuntimeExportId("");
      setStatus(
        `Imported GearBlocks runtime log and indexed ${indexedParts.length} unique part(s)`
      );
      showToast("Successful");
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setIsImportingRuntimeExports(false);
    }
  }

  async function importCatalogScreenshotImages(game: Game) {
    if (selectedPartCategory === "all") {
      setStatus("Select a specific GearBlocks category before importing a catalog screenshot");
      return;
    }

    setIsImportingCatalogScreenshot(true);
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        title: `Select ${selectedPartCategory} catalog screenshot`,
        filters: [
          {
            name: "PNG screenshots",
            extensions: ["png"]
          }
        ]
      });
      const imagePath = Array.isArray(selected) ? selected[0] : selected;
      if (!imagePath) {
        return;
      }

      const updatedParts = await importGearBlocksCatalogScreenshotImages(
        game.id,
        selectedPartCategory,
        imagePath
      );
      const orderedParts = await listGameRuntimeParts(game.id);
      setRuntimeParts(orderedParts);
      setStatus(
        `Refreshed Player.log and imported ${updatedParts.length} ${selectedPartCategory} catalog image(s)`
      );
      showToast("Successful");
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setIsImportingCatalogScreenshot(false);
    }
  }

  async function clearRuntimeCategoryImages(game: Game) {
    if (selectedPartCategory === "all") {
      setStatus("Select a specific GearBlocks category before clearing images");
      return;
    }

    if (
      !window.confirm(
        `Remove all display image associations from ${selectedPartCategory} parts? Image files will remain on disk.`
      )
    ) {
      return;
    }

    setIsClearingRuntimeCategoryImages(true);
    try {
      const clearedParts = await clearGameRuntimePartImagesForCategory(
        game.id,
        selectedPartCategory
      );
      const orderedParts = await listGameRuntimeParts(game.id);
      setRuntimeParts(orderedParts);
      setSelectedRuntimePartId((current) =>
        current && clearedParts.some((part) => part.id === current) ? null : current
      );
      setStatus(`Cleared ${clearedParts.length} ${selectedPartCategory} image association(s)`);
      showToast("Successful");
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setIsClearingRuntimeCategoryImages(false);
    }
  }

  function togglePromptScreenshot(screenshotId: number) {
    setSelectedPromptScreenshotIds((current) =>
      current.includes(screenshotId)
        ? current.filter((id) => id !== screenshotId)
        : [...current, screenshotId]
    );
  }

  function openScreenshotContextMenu(
    event: React.MouseEvent<HTMLElement>,
    screenshot: GameScreenshotCaptureRequest
  ) {
    event.preventDefault();
    const menuWidth = 238;
    const menuHeight = 274;
    const x = Math.min(event.clientX, window.innerWidth - menuWidth - 8);
    const y = Math.min(event.clientY, window.innerHeight - menuHeight - 8);

    setScreenshotContextMenu({
      screenshot,
      x: Math.max(8, x),
      y: Math.max(8, y)
    });
  }

  function closeScreenshotContextMenu() {
    setScreenshotContextMenu(null);
  }

  function runScreenshotMenuPlaceholder(label: string) {
    setStatus(`${label} is a visual test action`);
    setScreenshotContextMenu(null);
  }

  async function removeScreenshot(screenshot: GameScreenshotCaptureRequest) {
    if (!window.confirm("Delete this screenshot and its capture metadata?")) {
      return;
    }

    setDeletingScreenshotId(screenshot.id);
    setScreenshotContextMenu(null);
    try {
      await deleteGameScreenshot(screenshot.id);
      setScreenshots((current) => current.filter((item) => item.id !== screenshot.id));
      setStatus("Screenshot deleted");
      showToast("Deleted");
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setDeletingScreenshotId(null);
    }
  }

  if (selectedGame) {
    return (
      <section
        className={
          chatOverlayMode
            ? "game-element-canvas game-element-canvas-chat-overlay-mode"
            : "game-element-canvas"
        }
        aria-label="Selected game workspace"
      >
        {toastMessage && (
          <div className="game-toast" role="status">
            {toastMessage}
          </div>
        )}

        {!chatOverlayMode && (
        <div className="game-canvas-toolbar" aria-label="Game workspace actions">
          <button
            className={gameView === "home" ? "primary-button" : "ghost-button"}
            onClick={() => setGameView("home")}
            type="button"
          >
            Home
          </button>
          <button
            className={gameView === "chat" ? "primary-button" : "ghost-button"}
            onClick={openChatDefaults}
            type="button"
          >
            Chats
            <span className="button-count">{chatConversations.length}</span>
          </button>
          {selectedGame.slug === "gearblocks" && (
            <button
              className={gameView === "constructions" ? "primary-button" : "ghost-button"}
              onClick={() => setGameView("constructions")}
              type="button"
            >
              Constructions
            </button>
          )}
          <button
            className={gameView === "screenshots" ? "primary-button" : "ghost-button"}
            onClick={() => setGameView("screenshots")}
            type="button"
          >
            Screenshots
            <span className="button-count">{screenshots.length}</span>
          </button>
          <button
            className={gameView === "parts" ? "primary-button" : "ghost-button"}
            onClick={() => setGameView("parts")}
            type="button"
          >
            Parts
            <span className="button-count">
              {selectedGame.slug === "gearblocks" ? runtimeParts.length : parts.length}
            </span>
          </button>
          <button className="ghost-button" type="button">
            Add Reference
          </button>
        </div>
        )}

        {!chatOverlayMode &&
          gameView === "parts" &&
          selectedGame.slug !== "gearblocks" &&
          partCategories.length > 0 && (
          <div className="game-part-filter-bar" aria-label="Part category filters">
            <button
              className={selectedPartCategory === "all" ? "active" : ""}
              onClick={() => setSelectedPartCategory("all")}
              type="button"
            >
              <span>All</span>
              <strong>{selectedGame.slug === "gearblocks" ? runtimeParts.length : parts.length}</strong>
            </button>
            {partCategories.map((category) => (
              <button
                className={selectedPartCategory === category.name ? "active" : ""}
                key={category.name}
                onClick={() => setSelectedPartCategory(category.name)}
                title={category.name}
                type="button"
              >
                {category.iconPath ? (
                  <img alt="" src={convertFileSrc(category.iconPath)} />
                ) : (
                  <span>{category.fallbackIcon}</span>
                )}
                <strong>{category.count}</strong>
              </button>
            ))}
          </div>
        )}

        <div className="game-canvas-scroll-area">
          {gameView === "home" && (
            <section className="game-home-panel" aria-label={`${selectedGame.name} home`}>
              <div>
                <p>Game Workspace</p>
                <h3>{selectedGame.name}</h3>
              </div>
              <span>{screenshots.length} screenshots</span>
              <span>{parts.length} cataloged parts</span>
              <span>{runtimeParts.length} runtime API parts</span>
              <span>{chatConversations.length} chats</span>
              {selectedGame.slug === "gearblocks" && (
                <section className="game-data-locations-panel" aria-label="Game data locations">
                  <div className="game-data-locations-head">
                    <div>
                      <p>Local Data</p>
                      <h4>GearBlocks locations</h4>
                    </div>
                  </div>
                  <div className="game-data-location-list">
                    {GAME_DATA_LOCATION_OPTIONS.map((option) => {
                      const location = dataLocations.find(
                        (item) => item.locationType === option.type
                      );
                      const isUpdating = updatingDataLocationType === option.type;

                      return (
                        <article className="game-data-location-row" key={option.type}>
                          <div>
                            <strong>{option.title}</strong>
                            <span>{option.description}</span>
                            <code>{location?.directoryPath || "Not set"}</code>
                          </div>
                          <div className="game-data-location-actions">
                            <button
                              className="primary-button"
                              disabled={isUpdating}
                              onClick={() => void browseGameDataLocation(selectedGame, option.type)}
                              type="button"
                            >
                              Browse
                            </button>
                            <button
                              className="ghost-button"
                              disabled={isUpdating || !location}
                              onClick={() => void clearGameDataLocation(selectedGame, option.type)}
                              type="button"
                            >
                              Clear
                            </button>
                          </div>
                        </article>
                      );
                    })}
                  </div>
                </section>
              )}
              {selectedGame.slug === "gearblocks" && (
                <section
                  className="game-construction-decode-panel"
                  aria-label="GearBlocks construction decoder"
                >
                  <div className="game-data-locations-head">
                    <div>
                      <p>Construction Decoder</p>
                      <h4>Saved constructions</h4>
                    </div>
                    <div className="game-construction-head-actions">
                      <button
                        className="ghost-button"
                        disabled={isInstallingLuaExporter}
                        onClick={() => void installGearBlocksExporter(selectedGame)}
                        type="button"
                      >
                        Install Exporter
                      </button>
                      <button
                        className="ghost-button"
                        onClick={() => void refreshGearBlocksConstructionFiles(selectedGame.id)}
                        type="button"
                      >
                        Refresh
                      </button>
                    </div>
                  </div>

                  {luaExporterPath && (
                    <p className="game-construction-exporter-path">
                      Lua exporter installed at {luaExporterPath}
                    </p>
                  )}

                  {constructionFiles.length === 0 ? (
                    <p>No saved construction folders found.</p>
                  ) : (
                    <div className="game-construction-file-list">
                      {constructionFiles.map((construction) => (
                        <article
                          className={
                            decodedConstruction?.constructionPath === construction.constructionPath
                              ? "game-construction-file-row active"
                              : "game-construction-file-row"
                          }
                          key={construction.constructionPath}
                        >
                          <div>
                            <strong>{construction.name}</strong>
                            <span>{formatBytes(construction.byteSize)}</span>
                          </div>
                          <button
                            className="primary-button"
                            disabled={decodingConstructionPath === construction.constructionPath}
                            onClick={() => void decodeGearBlocksConstruction(construction)}
                            type="button"
                          >
                            Decode
                          </button>
                        </article>
                      ))}
                    </div>
                  )}

                  {decodedConstruction && (
                    <div className="game-construction-summary">
                      <div>
                        <p>Decoded BSON</p>
                        <h4>{decodedConstruction.name}</h4>
                      </div>
                      <div className="game-construction-summary-grid">
                        <span>{decodedConstruction.summary.compositeCount} composites</span>
                        <span>{decodedConstruction.summary.partCount} parts</span>
                        <span>
                          {decodedConstruction.summary.uniqueAssetGuidCount} unique asset GUIDs
                        </span>
                        <span>{decodedConstruction.summary.attachmentCount} attachments</span>
                        <span>{decodedConstruction.summary.linkCount} links</span>
                        <span>{formatBytes(decodedConstruction.decodedByteSize)} decoded</span>
                      </div>
                      <p>
                        Part display names require GearBlocks runtime API resolution after spawning
                        the save; this local decoder exposes asset GUIDs, transforms, dimensions,
                        behaviours, attachments, links, and raw BSON JSON.
                      </p>
                      <pre>{JSON.stringify(decodedConstruction.document, null, 2)}</pre>
                    </div>
                  )}

                  {runtimeExports.length > 0 && (
                    <div className="game-construction-summary">
                      <div>
                        <p>Runtime Exports</p>
                        <h4>{runtimeExports.length} Player.log export(s)</h4>
                      </div>
                      <div className="game-runtime-export-list">
                        {runtimeExports.map((runtimeExport) => (
                          <button
                            className={
                              runtimeExport.id === selectedRuntimeExportId
                                ? "game-runtime-export-row active"
                                : "game-runtime-export-row"
                            }
                            key={runtimeExport.id}
                            onClick={() => setSelectedRuntimeExportId(runtimeExport.id)}
                            type="button"
                          >
                            <span>{runtimeExport.name}</span>
                            <small>{formatBytes(runtimeExport.byteSize)}</small>
                          </button>
                        ))}
                      </div>
                      {selectedRuntimeExport && (
                        <>
                          <p>
                            Reconstructed from {selectedRuntimeExport.sourceLogPath}; intended
                            export path was {selectedRuntimeExport.intendedPath}.
                          </p>
                          <pre>{JSON.stringify(selectedRuntimeExport.document, null, 2)}</pre>
                        </>
                      )}
                    </div>
                  )}
                </section>
              )}
            </section>
          )}

          {gameView === "constructions" && selectedGame.slug === "gearblocks" && (
            <section
              className="game-constructions-panel"
              aria-label="GearBlocks constructions catalog"
            >
              <div className="game-view-head">
                <div>
                  <p>Catalog</p>
                  <h3>Constructions</h3>
                </div>
                <button
                  className="ghost-button"
                  disabled={isSyncingGameConstructions}
                  onClick={() => void syncGearBlocksConstructions(selectedGame.id)}
                  type="button"
                >
                  Refresh
                </button>
              </div>
              {gameConstructions.length === 0 ? (
                <p>No saved constructions indexed yet.</p>
              ) : (
                <div className="game-construction-catalog-list">
                  {gameConstructions.map((construction) => (
                    <article className="game-construction-catalog-row" key={construction.id}>
                      <div>
                        <strong>{construction.name}</strong>
                        <code>{construction.folderPath}</code>
                      </div>
                      <span>{construction.partCount} parts</span>
                      <span>{construction.compositeCount} composites</span>
                      <span>{formatBytes(construction.byteSize)}</span>
                    </article>
                  ))}
                </div>
              )}
            </section>
          )}

          {gameView === "chat" && (
            <ChatWorkspace
              conversations={chatConversations}
              chatOverlayMode={chatOverlayMode}
              contextSlot={
                <section className="chat-screenshot-context" aria-label="Screenshot context">
                  <div className="chat-screenshot-summary">
                    <div>
                      <p>Prompt context</p>
                      <strong>{selectedPromptScreenshotIds.length} screenshot(s) selected</strong>
                    </div>
                    <button
                      className="ghost-button"
                      onClick={() => setIsChatScreenshotPickerOpen((current) => !current)}
                      type="button"
                    >
                      {isChatScreenshotPickerOpen ? "Hide Screenshots" : "Add Screenshots"}
                    </button>
                  </div>

                  {isChatScreenshotPickerOpen && (
                    <>
                      <div className="chat-screenshot-actions">
                        <button
                          className="ghost-button"
                          disabled={chatScreenshotPage === 0 || screenshots.length === 0}
                          onClick={() =>
                            setChatScreenshotPage((current) => Math.max(0, current - 1))
                          }
                          type="button"
                        >
                          Prev
                        </button>
                        <span>
                          {screenshots.length === 0
                            ? "0 / 0"
                            : `${chatScreenshotPage + 1} / ${chatScreenshotPageCount}`}
                        </span>
                        <button
                          className="ghost-button"
                          disabled={
                            chatScreenshotPage >= chatScreenshotPageCount - 1 ||
                            screenshots.length === 0
                          }
                          onClick={() =>
                            setChatScreenshotPage((current) =>
                              Math.min(chatScreenshotPageCount - 1, current + 1)
                            )
                          }
                          type="button"
                        >
                          Next
                        </button>
                        <button
                          className="primary-button"
                          disabled={isRequestingScreenshot}
                          onClick={() => void requestScreenshotCapture(selectedGame)}
                          type="button"
                        >
                          Capture
                        </button>
                      </div>
                      {screenshots.length === 0 ? (
                        <p>No screenshots captured yet.</p>
                      ) : (
                        <div className="chat-screenshot-context-grid">
                          {visibleChatScreenshots.map((screenshot) => (
                            <label
                              className={
                                selectedPromptScreenshotIds.includes(screenshot.id)
                                  ? "chat-screenshot-option selected"
                                  : "chat-screenshot-option"
                              }
                              key={screenshot.id}
                            >
                              <input
                                checked={selectedPromptScreenshotIds.includes(screenshot.id)}
                                onChange={() => togglePromptScreenshot(screenshot.id)}
                                type="checkbox"
                              />
                              <img
                                alt={`Screenshot captured ${formatCapturedAt(screenshot)}`}
                                src={convertFileSrc(screenshot.filePath)}
                              />
                              <span>{formatCapturedAt(screenshot)}</span>
                            </label>
                          ))}
                        </div>
                      )}
                    </>
                  )}
                </section>
              }
              draft={chatDraft}
              emptyMainSlot={<GameChatDefaultsPane game={selectedGame} />}
              emptyConversationLabel="No game chats yet."
              hideSidebarWhenSelected
              inputPlaceholder={`Ask about ${selectedGame.name}...`}
              isSending={isSendingChat}
              messages={chatMessages}
              newConversationTitle={newChatTitle}
              onEnterChatOverlayMode={() => {
                if (selectedChatConversationId) {
                  onEnterChatOverlayMode?.(selectedGame, selectedChatConversationId);
                }
              }}
              onCaptureScreenshot={() => void capturePromptScreenshot(selectedGame)}
              onCreateConversation={() => void createGameChat()}
              onDeleteConversation={(conversation) => void removeGameChat(conversation)}
              onDraftChange={setChatDraft}
              onExitChatOverlayMode={onExitChatOverlayMode}
              onNewConversationTitleChange={setNewChatTitle}
              onSelectConversation={(conversationId) => {
                setSelectedChatConversationId(conversationId);
                setSelectedPromptScreenshotIds([]);
                setIsChatScreenshotPickerOpen(false);
              }}
              onSendMessage={() => void sendGameChat()}
              promptContextSummary={
                isCapturingPromptScreenshot
                  ? "Capturing screenshot..."
                  : selectedPromptScreenshotIds.length > 0
                    ? `${selectedPromptScreenshotIds.length} screenshot(s) attached`
                    : "No screenshots attached"
              }
              selectedConversationId={selectedChatConversationId}
              showFocusedToolbar={false}
              status={status}
              title={`${selectedGame.name} chat`}
            />
          )}

          {gameView === "parts" && (
            <section className="game-parts-panel" aria-label="Cataloged parts">
              <div className="game-view-head">
                <div>
                  <p>Catalog</p>
                  <h3>Parts</h3>
                </div>
                {selectedGame.slug !== "gearblocks" && (
                  <button
                    className="primary-button"
                    disabled={isCatalogingParts}
                    onClick={() => void catalogParts(selectedGame)}
                    type="button"
                  >
                    Catalog Parts
                  </button>
                )}
              </div>
              {selectedGame.slug === "gearblocks" && (
                <div className="game-runtime-parts-panel">
                  <div className="game-runtime-parts-head">
                    <div>
                      <p>GearBlocks Runtime API Index</p>
                      <h4>{runtimeParts.length} unique part(s)</h4>
                      <div className="game-runtime-catalog-metadata">
                        <span>Game {GEARBLOCKS_PARTS_CATALOG_METADATA.gameVersion}</span>
                        <span>{GEARBLOCKS_PARTS_CATALOG_METADATA.completeness}</span>
                        <span>{GEARBLOCKS_PARTS_CATALOG_METADATA.validation}</span>
                      </div>
                    </div>
                    <div className="game-runtime-parts-actions">
                      <button
                        className="ghost-button"
                        disabled={isImportingRuntimeExports}
                        onClick={() => void importGearBlocksRuntimeExports(selectedGame)}
                        type="button"
                      >
                        Import Runtime Log
                      </button>
                      {SHOW_GEARBLOCKS_CATALOG_MAINTENANCE_CONTROLS && (
                        <>
                          <button
                            className="ghost-button"
                            disabled={isImportingCatalogScreenshot || selectedPartCategory === "all"}
                            onClick={() => void importCatalogScreenshotImages(selectedGame)}
                            title={
                              selectedPartCategory === "all"
                                ? "Select a specific category before importing catalog icons"
                                : `Import ${selectedPartCategory} catalog icons`
                            }
                            type="button"
                          >
                            Import Catalog Screenshot
                          </button>
                          <button
                            className="ghost-button"
                            disabled={
                              isClearingRuntimeCategoryImages || selectedPartCategory === "all"
                            }
                            onClick={() => void clearRuntimeCategoryImages(selectedGame)}
                            title={
                              selectedPartCategory === "all"
                                ? "Select a specific category before clearing images"
                                : `Clear ${selectedPartCategory} image associations`
                            }
                            type="button"
                          >
                            Clear Category Images
                          </button>
                        </>
                      )}
                    </div>
                  </div>
                  {runtimeParts.length === 0 ? (
                    <p>
                      Export scene parts from the GearBlocks Lua script, then run the maintenance
                      Player.log import.
                    </p>
                  ) : (
                    <div className="game-runtime-catalog-layout">
                      <nav
                        className="game-runtime-category-rail"
                        aria-label="GearBlocks part categories"
                      >
                        {partCategories.map((category) => (
                          <button
                            className={
                              selectedPartCategory === category.name
                                ? "game-runtime-category-button active"
                                : "game-runtime-category-button"
                            }
                            key={category.name}
                            onClick={() => {
                              setSelectedPartCategory(category.name);
                              setSelectedRuntimePartId(null);
                            }}
                            title={`${category.name} (${category.count})`}
                            type="button"
                          >
                            {category.iconPath ? (
                              <img alt="" src={convertFileSrc(category.iconPath)} />
                            ) : (
                              <span>{category.fallbackIcon}</span>
                            )}
                          </button>
                        ))}
                      </nav>

                      <div className="game-runtime-catalog-main">
                        {selectedRuntimePart ? (
                          <div className="game-runtime-part-detail">
                            <div className="game-runtime-part-detail-head">
                              <button
                                className="ghost-button compact-button"
                                onClick={() => setSelectedRuntimePartId(null)}
                                type="button"
                              >
                                Back to {selectedPartCategory} icons
                              </button>
                              <button
                                className="ghost-button compact-button"
                                disabled={updatingRuntimePartImageId === selectedRuntimePart.id}
                                onClick={() =>
                                  void setRuntimePartImage(selectedGame, selectedRuntimePart)
                                }
                                type="button"
                              >
                                {selectedRuntimePart.displayImagePath
                                  ? "Replace Image"
                                  : "Set Image"}
                              </button>
                            </div>

                            <div className="game-runtime-part-detail-layout">
                              <div className="game-runtime-part-detail-image">
                                {selectedRuntimePart.displayImagePath ? (
                                  <img
                                    alt={`${
                                      selectedRuntimePart.displayName ||
                                      selectedRuntimePart.assetName ||
                                      "Part"
                                    } catalog icon`}
                                    src={convertFileSrc(selectedRuntimePart.displayImagePath)}
                                  />
                                ) : (
                                  <span>No image</span>
                                )}
                              </div>

                              <div className="game-runtime-part-detail-main">
                                <div>
                                  <p>Part Detail</p>
                                  <h4>
                                    {selectedRuntimePart.displayName ||
                                      selectedRuntimePart.assetName ||
                                      selectedRuntimePart.partKey}
                                  </h4>
                                </div>
                                <dl className="game-runtime-part-definition-grid">
                                  {runtimePartDetailRows(selectedRuntimePart).map(
                                    ([label, value]) => (
                                      <div key={label}>
                                        <dt>{label}</dt>
                                        <dd>{value}</dd>
                                      </div>
                                    )
                                  )}
                                </dl>
                              </div>
                            </div>

                            <section
                              className="game-runtime-part-attributes"
                              aria-label="Available API attributes"
                            >
                              <div>
                                <p>Catalog Attributes</p>
                                <h4>
                                  {runtimePartAvailableAttributes(selectedRuntimePart).length}{" "}
                                  available member(s)
                                </h4>
                              </div>
                              <div className="game-runtime-part-attribute-list">
                                {runtimePartAvailableAttributes(selectedRuntimePart).length > 0 ? (
                                  runtimePartAvailableAttributes(selectedRuntimePart).map(
                                    (attribute) => <span key={attribute}>{attribute}</span>
                                  )
                                ) : (
                                  <span>No runtime API attributes indexed yet</span>
                                )}
                              </div>
                            </section>

                            <label className="game-runtime-part-notes">
                              <span>Notes</span>
                              <textarea
                                onChange={(event) => setRuntimePartNotesDraft(event.target.value)}
                                placeholder="Add practical notes about this part."
                                value={runtimePartNotesDraft}
                              />
                            </label>
                            <div className="game-runtime-part-detail-actions">
                              <button
                                className="primary-button"
                                disabled={
                                  isSavingRuntimePartNotes ||
                                  runtimePartNotesDraft === selectedRuntimePart.notes
                                }
                                onClick={() =>
                                  void saveRuntimePartNotes(selectedGame, selectedRuntimePart)
                                }
                                type="button"
                              >
                                Save Notes
                              </button>
                            </div>

                            <section
                              className="game-runtime-part-properties"
                              aria-label="Part properties"
                            >
                              <div>
                                <p>Runtime API Properties</p>
                                <h4>DB Definitions</h4>
                              </div>
                              <pre>
                                {formatRuntimePartProperties(selectedRuntimePart.propertiesJson)}
                              </pre>
                            </section>
                          </div>
                        ) : selectedPartCategory === "all" ? (
                          <p>Select a part category to view the in-game icon layout.</p>
                        ) : displayedRuntimeParts.length === 0 ? (
                          <p>No runtime API parts found for this category.</p>
                        ) : (
                          <div
                            className="game-runtime-icon-grid-shell"
                            aria-label={`${selectedPartCategory} catalog icons`}
                          >
                            <div className="game-runtime-icon-grid">
                              {displayedRuntimeParts.map((part) => (
                                <button
                                  className="game-runtime-icon-tile"
                                  key={part.id}
                                  onClick={() => setSelectedRuntimePartId(part.id)}
                                  title={part.displayName || part.assetName || part.partKey}
                                  type="button"
                                >
                                  {part.displayImagePath ? (
                                    <img
                                      alt={`${
                                        part.displayName || part.assetName || "Part"
                                      } catalog icon`}
                                      src={convertFileSrc(part.displayImagePath)}
                                    />
                                  ) : (
                                    <span className="game-runtime-icon-placeholder">
                                      No image
                                    </span>
                                  )}
                                </button>
                              ))}
                            </div>
                          </div>
                        )}
                      </div>
                    </div>
                  )}
                </div>
              )}
              {selectedGame.slug !== "gearblocks" && (
                <div className="game-parts-chart">
                  {parts.length === 0 ? (
                    <p>No parts cataloged yet.</p>
                  ) : displayedParts.length === 0 ? (
                    <p>No parts cataloged for this category yet.</p>
                  ) : (
                    <>
                      <div className="game-parts-chart-head">
                        <span>Thumbnail</span>
                        <span>Part name</span>
                        <span>Practical physics use</span>
                      </div>
                      {displayedParts.map((part) => (
                        <article className="game-part-row" key={part.id}>
                          <div className="game-part-thumb">
                            {part.thumbnailPath ? (
                              <img
                                alt={`${part.name} source screenshot`}
                                src={convertFileSrc(part.thumbnailPath)}
                              />
                            ) : (
                              <span>No image</span>
                            )}
                          </div>
                          <strong className="game-part-name">{part.name}</strong>
                          <p>{part.description}</p>
                        </article>
                      ))}
                    </>
                  )}
                </div>
              )}
            </section>
          )}

          {gameView === "screenshots" && (
            <section className="game-screenshots-panel" aria-label="Screenshots">
              <div className="game-view-head">
                <div>
                  <p>Gallery</p>
                  <h3>Screenshots</h3>
                </div>
                <button
                  className="primary-button"
                  disabled={isRequestingScreenshot}
                  onClick={() => void requestScreenshotCapture(selectedGame)}
                  type="button"
                >
                  Capture Screenshot
                </button>
              </div>
              <div className="game-screenshot-list">
                {screenshots.length === 0 ? (
                  <p>No screenshots captured yet.</p>
                ) : (
                  screenshots.map((screenshot) => (
                    <article
                      className={
                        deletingScreenshotId === screenshot.id
                          ? "game-screenshot-card deleting"
                          : "game-screenshot-card"
                      }
                      key={screenshot.id}
                      onContextMenu={(event) => openScreenshotContextMenu(event, screenshot)}
                    >
                      <img
                        alt={`Screenshot captured ${formatCapturedAt(screenshot)}`}
                        src={convertFileSrc(screenshot.filePath)}
                      />
                      <div>
                        <strong>{formatCapturedAt(screenshot)}</strong>
                        <span>{screenshot.captureStatus}</span>
                      </div>
                    </article>
                  ))
                )}
              </div>
            </section>
          )}
        </div>

        {screenshotContextMenu && (
          <div className="game-context-menu-layer" onMouseDown={closeScreenshotContextMenu}>
            <div
              aria-label="Screenshot actions"
              className="game-context-menu"
              onMouseDown={(event) => event.stopPropagation()}
              role="menu"
              style={{
                left: screenshotContextMenu.x,
                top: screenshotContextMenu.y
              }}
            >
              <div className="game-context-menu-title">
                <strong>{formatCapturedAt(screenshotContextMenu.screenshot)}</strong>
                <span>{screenshotContextMenu.screenshot.captureStatus}</span>
              </div>
              <button
                onClick={() => runScreenshotMenuPlaceholder("Open preview")}
                role="menuitem"
                type="button"
              >
                Open preview
              </button>
              <button
                onClick={() => runScreenshotMenuPlaceholder("Create object from screenshot")}
                role="menuitem"
                type="button"
              >
                Create object from screenshot
              </button>
              <button
                onClick={() => runScreenshotMenuPlaceholder("Attach reference")}
                role="menuitem"
                type="button"
              >
                Attach reference
              </button>
              <button
                onClick={() => runScreenshotMenuPlaceholder("Mark reviewed")}
                role="menuitem"
                type="button"
              >
                Mark reviewed
              </button>
              <div className="game-context-menu-separator" />
              <button
                className="danger"
                disabled={deletingScreenshotId === screenshotContextMenu.screenshot.id}
                onClick={() => void removeScreenshot(screenshotContextMenu.screenshot)}
                role="menuitem"
                type="button"
              >
                Delete screenshot and references
              </button>
            </div>
          </div>
        )}
      </section>
    );
  }

  return (
    <section className="feature-panel">
      <div className="panel-heading">
        <div>
          <p>Game Workspaces</p>
          <h3>Gaming</h3>
        </div>
        <span className="save-pill">{status}</span>
      </div>

      <div className="split-feature-body">
        <aside className="sub-list" aria-label="Game section list">
          {gameSections.map((game) => (
            <button
              className={game.id === selectedGameId ? "sub-list-item active" : "sub-list-item"}
              key={game.id}
              onClick={() => {
                onSelectGame(game.id);
                setStatus("Game section selected");
              }}
              type="button"
            >
              <strong>{game.name}</strong>
              <span>Game section</span>
            </button>
          ))}
        </aside>

        <div className="gaming-detail-panel">
          <div className="inline-form">
            <input
              aria-label="New game section name"
              className="text-input"
              onChange={(event) => setNewGameName(event.target.value)}
              placeholder="Game section name"
              value={newGameName}
            />
            <button className="primary-button" onClick={() => void addGameSection()} type="button">
              Add Game
            </button>
            <button
              className="ghost-button"
              disabled={!selectedGame}
              onClick={() => void removeSelectedGame()}
              type="button"
            >
              Remove
            </button>
          </div>

          <div className="empty-editor-state">
            <p>Add a game section to begin.</p>
          </div>
        </div>
      </div>
    </section>
  );
}

function formatError(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function parseJsonOrFallback(value: string, fallback: unknown) {
  try {
    return JSON.parse(value);
  } catch {
    return fallback;
  }
}

function screenshotTimestampLabel(date: Date) {
  const year = date.getFullYear();
  const month = padDatePart(date.getMonth() + 1);
  const day = padDatePart(date.getDate());
  const hours = padDatePart(date.getHours());
  const minutes = padDatePart(date.getMinutes());
  const seconds = padDatePart(date.getSeconds());

  return `${year}${month}${day}_${hours}${minutes}${seconds}`;
}

function padDatePart(value: number) {
  return value.toString().padStart(2, "0");
}

function formatCapturedAt(screenshot: GameScreenshotCaptureRequest) {
  const value = screenshot.capturedAt || screenshot.createdAt;
  const match = /^(\d{4})(\d{2})(\d{2})_(\d{2})(\d{2})(\d{2})/.exec(value);

  if (match) {
    const [, year, month, day, hour, minute, second] = match;
    return `${year}-${month}-${day} ${hour}:${minute}:${second}`;
  }

  return value || "Unknown capture time";
}

function gameDataLocationTitle(locationType: GameDataLocationType) {
  return (
    GAME_DATA_LOCATION_OPTIONS.find((option) => option.type === locationType)?.title ??
    "Data Location"
  );
}

function upsertGameDataLocation(
  locations: GameDataLocation[],
  nextLocation: GameDataLocation
) {
  const withoutExisting = locations.filter(
    (location) => location.locationType !== nextLocation.locationType
  );
  return [...withoutExisting, nextLocation].sort(
    (left, right) =>
      gameDataLocationSortValue(left.locationType) - gameDataLocationSortValue(right.locationType)
  );
}

function gameDataLocationSortValue(locationType: GameDataLocationType) {
  return locationType === "save" ? 0 : 1;
}

function formatBytes(value: number) {
  if (value < 1024) {
    return `${value} B`;
  }
  if (value < 1024 * 1024) {
    return `${(value / 1024).toFixed(1)} KB`;
  }
  return `${(value / (1024 * 1024)).toFixed(1)} MB`;
}

function runtimePartDetailRows(part: GameRuntimePart): Array<[string, string]> {
  return [
    ["ID", String(part.id)],
    ["Part Key", part.partKey],
    ["Display Name", part.displayName],
    ["Full Display Name", part.fullDisplayName],
    ["Asset Name", part.assetName],
    ["Asset GUID", part.assetGuid],
    ["Category", part.category || "Uncategorized"],
    ["Mass", part.mass.toFixed(2)],
    ["Source Export ID", part.sourceExportId],
    ["Source Construction ID", part.sourceConstructionId],
    ["Last Seen", part.lastSeenAt],
    ["Display Image Path", part.displayImagePath],
    ["Source Image Path", part.sourceImagePath],
    ["Created", part.createdAt],
    ["Updated", part.updatedAt]
  ].map(([label, value]) => [label, value || "Not set"]);
}

function runtimePartAvailableAttributes(part: GameRuntimePart) {
  const parsed = parseJsonOrFallback(part.propertiesJson, null);
  const attributes = new Set<string>();
  collectRuntimePartAvailableAttributes(parsed, attributes);
  return Array.from(attributes).sort((left, right) => left.localeCompare(right));
}

function collectRuntimePartAvailableAttributes(value: unknown, attributes: Set<string>) {
  if (!value || typeof value !== "object") {
    return;
  }

  if (Array.isArray(value)) {
    for (const item of value) {
      collectRuntimePartAvailableAttributes(item, attributes);
    }
    return;
  }

  const object = value as Record<string, unknown>;
  const apiAttributes = object.apiAttributes;
  if (Array.isArray(apiAttributes)) {
    for (const item of apiAttributes) {
      if (!item || typeof item !== "object") {
        continue;
      }
      const attribute = item as Record<string, unknown>;
      const interfaceName =
        typeof attribute.interface === "string" ? attribute.interface.trim() : "";
      const attributeName = typeof attribute.name === "string" ? attribute.name.trim() : "";
      if (interfaceName && attributeName) {
        attributes.add(`${interfaceName}.${attributeName}`);
      }
    }
  }

  for (const [key, child] of Object.entries(object)) {
    if (key !== "apiAttributes") {
      collectRuntimePartAvailableAttributes(child, attributes);
    }
  }
}

function formatRuntimePartProperties(propertiesJson: string) {
  if (!propertiesJson.trim()) {
    return "{}";
  }

  try {
    return JSON.stringify(JSON.parse(propertiesJson), null, 2);
  } catch {
    return propertiesJson;
  }
}

function GameChatDefaultsPane({ game }: { game: Game }) {
  const isGearBlocks = game.slug === "gearblocks";

  return (
    <section className="game-chat-defaults-panel" aria-label={`${game.name} chat defaults`}>
      <div className="game-view-head">
        <div>
          <p>Chat Defaults</p>
          <h3>{game.name}</h3>
        </div>
      </div>

      <div className="game-chat-defaults-grid">
        <article className={isGearBlocks ? "game-chat-default-item enabled" : "game-chat-default-item"}>
          <div>
            <strong>{isGearBlocks ? "Parts catalog context" : "Game context"}</strong>
            <span>
              {isGearBlocks
                ? "Enabled for every GearBlocks chat"
                : "No game-specific context configured yet"}
            </span>
          </div>
          <span>{isGearBlocks ? "On" : "Off"}</span>
        </article>

        {isGearBlocks && (
          <article className="game-chat-default-item enabled">
            <div>
              <strong>Runtime construction context</strong>
              <span>Latest Overlay Forge exporter data from Player.log is summarized for chat</span>
            </div>
            <span>Auto</span>
          </article>
        )}

        <article className="game-chat-default-item enabled">
          <div>
            <strong>Screenshot context</strong>
            <span>Select screenshots after opening or creating a chat</span>
          </div>
          <span>Per prompt</span>
        </article>

        <article className="game-chat-default-item enabled">
          <div>
            <strong>Response speed</strong>
            <span>Low reasoning, concise output, low-detail images</span>
          </div>
          <span>Fast</span>
        </article>
      </div>

      <p className="game-chat-defaults-note">
        Select an existing chat or create a new chat to open the AI interface.
      </p>
    </section>
  );
}
