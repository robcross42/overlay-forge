import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useMemo, useRef, useState } from "react";
import { ChatWorkspace } from "../../components/ChatWorkspace";
import {
  catalogGamePartsFromScreenshots,
  clearGameRuntimePartImagesForCategory,
  clearGearBlocksMarkers,
  createGame,
  createGameBuildGuideFromChat,
  createGameCharacterBuild,
  createGameChatConversation,
  createGameChatScreenshotCapture,
  createGameScreenshotCaptureRequest,
  decodeGearBlocksConstructionFile,
  deleteGameBuildGuide,
  deleteGameCharacterBuild,
  getGearBlocksThirdPartyDependencyStatus,
  importGameBuildGuideMarkdown,
  importGameBuildGuideUrl,
  importGearBlocksOfficialApiDocs,
  installGearBlocksLuaExporter,
  importGearBlocksCatalogScreenshotImages,
  importGearBlocksRuntimePartIndex,
  deleteGameChatConversation,
  deleteGameDataLocation,
  deleteGameScreenshot,
  deleteGame,
  listGameChatConversations,
  listGameCharacterBuilds,
  listGameDataLocations,
  listGameChatMessages,
  listGameCatalogObjects,
  listGameConstructions,
  listGameRuntimePartApiMembers,
  listGearBlocksApiCatalog,
  listGearBlocksConstructionFiles,
  listGameBuildGuides,
  listGamePartCategories,
  listGameRuntimeParts,
  listGameScreenshots,
  openGameBuildGuideOverlayWindow,
  saveGameDataLocation,
  setActiveGameCharacterBuild,
  setGameRuntimePartDisplayImage,
  sendGameChatMessage,
  sendGearBlocksMarkerCommands,
  syncGearBlocksRuntimeContext,
  syncGearBlocksSavedConstructions,
  updateGameCharacterBuild,
  updateGameRuntimePartNotes
} from "../../services/gaming";
import type {
  Game,
  GameBuildGuide,
  GameCharacterBuild,
  GameCharacterBuildInput,
  GameChatConversation,
  GameChatMessage,
  GameCatalogObject,
  GameConstruction,
  GameDataLocation,
  GameDataLocationType,
  GameRuntimePart,
  GameRuntimePartApiMember,
  GearBlocksApiCatalog,
  GearBlocksApiMember,
  GearBlocksConstructionDecode,
  GearBlocksConstructionFile,
  GearBlocksThirdPartyDependencyStatusPayload,
  GearBlocksRuntimeExport,
  GearBlocksMarkerInput,
  GamePartCategory,
  GameScreenshotCaptureRequest
} from "../../services/gaming";
import { timestampLabel } from "../../utils/datetime";
import { formatUnknownError as formatError } from "../../utils/errors";

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
  type: "home" | "newChat" | "chat" | "screenshots" | "parts" | "build-guides";
  gameId?: number;
  conversationId?: number;
  nonce: number;
};

type ScreenshotContextMenu = {
  screenshot: GameScreenshotCaptureRequest;
  x: number;
  y: number;
};

type PathOfExile2BuildDraft = {
  title: string;
  characterClass: string;
  ascendancy: string;
  buildRole: string;
  status: string;
  sourceLabel: string;
  sourceUrl: string;
  patch: string;
  summary: string;
  tags: string;
  notes: string;
  isActive: boolean;
};

type GameView =
  | "home"
  | "chat"
  | "screenshots"
  | "parts"
  | "constructions"
  | "api"
  | "tools"
  | "build-guides"
  | "builds"
  | "skill-tree"
  | "items"
  | "skill-gems"
  | "support-gems"
  | "loot-filter"
  | "trade";
const CHAT_SCREENSHOTS_PER_PAGE = 8;
const PATH_OF_EXILE_2_SLUG = "path-of-exile-2";
const EMPTY_PATH_OF_EXILE_2_BUILD_DRAFT: PathOfExile2BuildDraft = {
  title: "",
  characterClass: "",
  ascendancy: "",
  buildRole: "",
  status: "planned",
  sourceLabel: "",
  sourceUrl: "",
  patch: "",
  summary: "",
  tags: "",
  notes: "",
  isActive: false
};
const ENABLE_GEARBLOCKS_MARKERS = false;
const ENABLE_GEARBLOCKS_PLUGIN_STATUS = false;
const PATH_OF_EXILE_2_SECTIONS: Array<{
  view: GameView;
  label: string;
  eyebrow: string;
  description: string;
}> = [
  {
    view: "builds",
    label: "Builds",
    eyebrow: "Character Planning",
    description: "Planned location for character builds, ascendancy choices, campaign notes, and endgame goals."
  },
  {
    view: "skill-tree",
    label: "Skill Tree",
    eyebrow: "Passive Planning",
    description: "Planned location for passive tree routes, keystones, weapon swaps, and respec notes."
  },
  {
    view: "items",
    label: "Items",
    eyebrow: "Equipment",
    description: "Planned location for gear targets, rare item notes, uniques, affixes, and upgrade priorities."
  },
  {
    view: "skill-gems",
    label: "Skill Gems",
    eyebrow: "Active Skills",
    description: "Planned location for active skill gems, gem levels, quality notes, and socket groups."
  },
  {
    view: "support-gems",
    label: "Support Gems",
    eyebrow: "Links",
    description: "Planned location for support gem combinations, damage conversions, and utility links."
  },
  {
    view: "loot-filter",
    label: "Loot Filter",
    eyebrow: "Drops",
    description: "Planned location for loot filter rules, strictness profiles, currency visibility, and leveling presets."
  },
  {
    view: "trade",
    label: "Trade",
    eyebrow: "Market",
    description: "Planned location for trade searches, price notes, acquisition targets, and economy references."
  }
];
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
  const [gearBlocksDependencyStatus, setGearBlocksDependencyStatus] =
    useState<GearBlocksThirdPartyDependencyStatusPayload | null>(null);
  const [isLoadingGearBlocksDependencyStatus, setIsLoadingGearBlocksDependencyStatus] =
    useState(false);
  const [runtimeExports, setRuntimeExports] = useState<GearBlocksRuntimeExport[]>([]);
  const [selectedRuntimeExportId, setSelectedRuntimeExportId] = useState("");
  const [gearBlocksApiCatalog, setGearBlocksApiCatalog] = useState<GearBlocksApiCatalog | null>(
    null
  );
  const [selectedGearBlocksApiTypeId, setSelectedGearBlocksApiTypeId] = useState<number | null>(
    null
  );
  const [isLoadingGearBlocksApiCatalog, setIsLoadingGearBlocksApiCatalog] = useState(false);
  const [isImportingGearBlocksApiDocs, setIsImportingGearBlocksApiDocs] = useState(false);
  const [isImportingRuntimeExports, setIsImportingRuntimeExports] = useState(false);
  const [isImportingCatalogScreenshot, setIsImportingCatalogScreenshot] = useState(false);
  const [isClearingRuntimeCategoryImages, setIsClearingRuntimeCategoryImages] = useState(false);
  const [updatingRuntimePartImageId, setUpdatingRuntimePartImageId] = useState<number | null>(null);
  const [selectedRuntimePartId, setSelectedRuntimePartId] = useState<number | null>(null);
  const [selectedRuntimePartApiMembers, setSelectedRuntimePartApiMembers] = useState<
    GameRuntimePartApiMember[]
  >([]);
  const [isLoadingRuntimePartApiMembers, setIsLoadingRuntimePartApiMembers] = useState(false);
  const [runtimePartNotesDraft, setRuntimePartNotesDraft] = useState("");
  const [isSavingRuntimePartNotes, setIsSavingRuntimePartNotes] = useState(false);
  const [chatConversations, setChatConversations] = useState<GameChatConversation[]>([]);
  const [selectedChatConversationId, setSelectedChatConversationId] = useState<number | null>(null);
  const [chatMessages, setChatMessages] = useState<GameChatMessage[]>([]);
  const [buildGuides, setBuildGuides] = useState<GameBuildGuide[]>([]);
  const [isImportingBuildGuide, setIsImportingBuildGuide] = useState(false);
  const [isImportingBuildGuideUrl, setIsImportingBuildGuideUrl] = useState(false);
  const [buildGuideImportProgressFrame, setBuildGuideImportProgressFrame] = useState(0);
  const [isGeneratingBuildGuide, setIsGeneratingBuildGuide] = useState(false);
  const [isOpeningBuildGuide, setIsOpeningBuildGuide] = useState(false);
  const [deletingBuildGuideId, setDeletingBuildGuideId] = useState<number | null>(null);
  const [pathOfExile2Builds, setPathOfExile2Builds] = useState<GameCharacterBuild[]>([]);
  const [pathOfExile2BuildDraft, setPathOfExile2BuildDraft] =
    useState<PathOfExile2BuildDraft>(EMPTY_PATH_OF_EXILE_2_BUILD_DRAFT);
  const [editingPathOfExile2BuildId, setEditingPathOfExile2BuildId] = useState<number | null>(
    null
  );
  const [isSavingPathOfExile2Build, setIsSavingPathOfExile2Build] = useState(false);
  const [deletingPathOfExile2BuildId, setDeletingPathOfExile2BuildId] = useState<number | null>(
    null
  );
  const [newChatTitle, setNewChatTitle] = useState("");
  const [chatDraft, setChatDraft] = useState("");
  const [isSendingChat, setIsSendingChat] = useState(false);
  const [isSendingGearBlocksMarkers, setIsSendingGearBlocksMarkers] = useState(false);
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
  const selectedGearBlocksApiType = useMemo(
    () =>
      gearBlocksApiCatalog?.types.find((type) => type.id === selectedGearBlocksApiTypeId) ?? null,
    [gearBlocksApiCatalog, selectedGearBlocksApiTypeId]
  );
  const selectedGearBlocksApiMembers = useMemo(
    () =>
      selectedGearBlocksApiType && gearBlocksApiCatalog
        ? gearBlocksApiCatalog.members.filter(
            (member) => member.typeId === selectedGearBlocksApiType.id
          )
        : [],
    [gearBlocksApiCatalog, selectedGearBlocksApiType]
  );
  const selectedGearBlocksApiEnumValues = useMemo(
    () =>
      selectedGearBlocksApiType && gearBlocksApiCatalog
        ? gearBlocksApiCatalog.enumValues.filter(
            (value) => value.typeId === selectedGearBlocksApiType.id
          )
        : [],
    [gearBlocksApiCatalog, selectedGearBlocksApiType]
  );
  const selectedGearBlocksApiMemberParameters = useMemo(() => {
    if (!gearBlocksApiCatalog) {
      return new Map<number, string[]>();
    }
    const parameters = new Map<number, string[]>();
    for (const parameter of gearBlocksApiCatalog.parameters) {
      const label = `${parameter.parameterType} ${parameter.parameterName}${
        parameter.defaultValue ? `=${parameter.defaultValue}` : ""
      }`;
      const current = parameters.get(parameter.memberId) ?? [];
      current.push(label.trim());
      parameters.set(parameter.memberId, current);
    }
    return parameters;
  }, [gearBlocksApiCatalog]);

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
    if (
      selectedGame &&
      selectedGame.slug !== "gearblocks" &&
      (gameView === "constructions" || gameView === "api" || gameView === "tools")
    ) {
      setGameView("home");
    }
  }, [gameView, selectedGame?.id, selectedGame?.slug]);

  useEffect(() => {
    if (
      selectedGame &&
      selectedGame.slug !== PATH_OF_EXILE_2_SLUG &&
      PATH_OF_EXILE_2_SECTIONS.some((section) => section.view === gameView)
    ) {
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
      setGearBlocksApiCatalog(null);
      setSelectedGearBlocksApiTypeId(null);
      setGearBlocksDependencyStatus(null);
      setIsLoadingGearBlocksDependencyStatus(false);
      setIsLoadingGearBlocksApiCatalog(false);
      setIsImportingRuntimeExports(false);
      setIsImportingCatalogScreenshot(false);
      setIsClearingRuntimeCategoryImages(false);
      setUpdatingRuntimePartImageId(null);
      setSelectedRuntimePartId(null);
      setSelectedRuntimePartApiMembers([]);
      setIsLoadingRuntimePartApiMembers(false);
      setRuntimePartNotesDraft("");
      setIsSavingRuntimePartNotes(false);
      setChatConversations([]);
      setSelectedChatConversationId(null);
      setChatMessages([]);
      setBuildGuides([]);
      setIsImportingBuildGuide(false);
      setIsImportingBuildGuideUrl(false);
      setBuildGuideImportProgressFrame(0);
      setIsGeneratingBuildGuide(false);
      setIsOpeningBuildGuide(false);
      setDeletingBuildGuideId(null);
      setPathOfExile2Builds([]);
      setPathOfExile2BuildDraft(EMPTY_PATH_OF_EXILE_2_BUILD_DRAFT);
      setEditingPathOfExile2BuildId(null);
      setIsSavingPathOfExile2Build(false);
      setDeletingPathOfExile2BuildId(null);
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
          void refreshGearBlocksApiCatalog();
          if (ENABLE_GEARBLOCKS_PLUGIN_STATUS) {
            void refreshGearBlocksDependencyStatus(selectedGame.id);
          }
        } else {
          setConstructionFiles([]);
          setGameConstructions([]);
          setDecodedConstruction(null);
          setGearBlocksApiCatalog(null);
          setSelectedGearBlocksApiTypeId(null);
          setGearBlocksDependencyStatus(null);
        }
      })
      .catch((error) => setStatus(formatError(error)));
    listGameBuildGuides(selectedGame.id)
      .then(setBuildGuides)
      .catch((error) => setStatus(formatError(error)));
    if (selectedGame.slug === PATH_OF_EXILE_2_SLUG) {
      listGameCharacterBuilds(selectedGame.id)
        .then(setPathOfExile2Builds)
        .catch((error) => setStatus(formatError(error)));
    } else {
      setPathOfExile2Builds([]);
      setPathOfExile2BuildDraft(EMPTY_PATH_OF_EXILE_2_BUILD_DRAFT);
      setEditingPathOfExile2BuildId(null);
      setIsSavingPathOfExile2Build(false);
      setDeletingPathOfExile2BuildId(null);
    }
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
    if (!isImportingBuildGuide && !isImportingBuildGuideUrl) {
      setBuildGuideImportProgressFrame(0);
      return;
    }

    const intervalId = window.setInterval(() => {
      setBuildGuideImportProgressFrame((current) => (current + 1) % 3);
    }, 450);

    return () => window.clearInterval(intervalId);
  }, [isImportingBuildGuide, isImportingBuildGuideUrl]);

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
    if (!selectedGame || selectedGame.slug !== "gearblocks" || !selectedRuntimePart) {
      setSelectedRuntimePartApiMembers([]);
      setIsLoadingRuntimePartApiMembers(false);
      return;
    }

    let isCancelled = false;
    setIsLoadingRuntimePartApiMembers(true);
    listGameRuntimePartApiMembers(selectedGame.id, selectedRuntimePart.id)
      .then((members) => {
        if (!isCancelled) {
          setSelectedRuntimePartApiMembers(members);
        }
      })
      .catch((error) => {
        if (!isCancelled) {
          setSelectedRuntimePartApiMembers([]);
          setStatus(formatError(error));
        }
      })
      .finally(() => {
        if (!isCancelled) {
          setIsLoadingRuntimePartApiMembers(false);
        }
      });

    return () => {
      isCancelled = true;
    };
  }, [selectedGame?.id, selectedGame?.slug, selectedRuntimePart?.id]);

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

  useEffect(() => {
    let isMounted = true;
    let cleanup: (() => void) | null = null;

    listen<GameBuildGuide>("game-build-guides-changed", (event) => {
      if (!isMounted || event.payload.gameId !== selectedGameId) {
        return;
      }
      listGameBuildGuides(event.payload.gameId)
        .then(setBuildGuides)
        .catch((error) => setStatus(formatError(error)));
    }).then((nextCleanup) => {
      if (isMounted) {
        cleanup = nextCleanup;
      } else {
        nextCleanup();
      }
    });

    return () => {
      isMounted = false;
      cleanup?.();
    };
  }, [selectedGameId]);

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
        timestampLabel(new Date())
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
        timestampLabel(new Date())
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

  async function sendGameChat(includeSceneDiff = false) {
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
    setStatus(includeSceneDiff ? "Including scene diff" : "Waiting for OpenAI");

    try {
      const nextMessages = await sendGameChatMessage(
        selectedChatConversationId,
        content,
        selectedPromptScreenshotIds,
        includeSceneDiff
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

  async function generateBuildGuideFromChat() {
    if (!selectedGame || selectedGame.slug !== "gearblocks") {
      setStatus("Build guide generation requires the GearBlocks game section");
      return;
    }
    if (!selectedChatConversationId) {
      setStatus("Select a GearBlocks chat before generating a build guide");
      return;
    }

    const latestUserMessage = [...chatMessages]
      .reverse()
      .find((message) => message.role === "user");
    const buildGoal = chatDraft.trim() || latestUserMessage?.content.trim() || "";
    if (!buildGoal) {
      setStatus("Type a build goal or select a chat with a previous user message");
      return;
    }

    setIsGeneratingBuildGuide(true);
    setStatus("Generating build guide");
    try {
      const generated = await createGameBuildGuideFromChat(
        selectedChatConversationId,
        buildGoal
      );
      const guides = await listGameBuildGuides(selectedGame.id);
      setBuildGuides(guides);
      setChatDraft("");
      setStatus(
        `Generated build guide "${generated.guide.title}" with ${generated.parts.length} part row(s) and ${generated.steps.length} step(s)`
      );
      showToast("Build guide created");
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setIsGeneratingBuildGuide(false);
    }
  }

  async function sendGearBlocksMarkersFromMessage(message: GameChatMessage) {
    if (!selectedGame || selectedGame.slug !== "gearblocks") {
      setStatus("GearBlocks markers require the GearBlocks game section");
      return;
    }

    const markers = extractGearBlocksMarkers(message.content);
    if (markers.length === 0) {
      setStatus("No marker plan found in that response");
      return;
    }

    setIsSendingGearBlocksMarkers(true);
    try {
      const result = await sendGearBlocksMarkerCommands(selectedGame.id, markers);
      setStatus(`Sent ${result.commandCount} marker command(s) to GearBlocks`);
    } catch (error) {
      setStatus(formatError(error));
    } finally {
      setIsSendingGearBlocksMarkers(false);
    }
  }

  async function clearGearBlocksChatMarkers() {
    if (!selectedGame || selectedGame.slug !== "gearblocks") {
      setStatus("GearBlocks markers require the GearBlocks game section");
      return;
    }

    setIsSendingGearBlocksMarkers(true);
    try {
      await clearGearBlocksMarkers(selectedGame.id);
      setStatus("Sent clear markers command to GearBlocks");
    } catch (error) {
      setStatus(formatError(error));
    } finally {
      setIsSendingGearBlocksMarkers(false);
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

  async function importBuildGuide(game: Game) {
    setIsImportingBuildGuide(true);
    try {
      const selected = await open({
        directory: false,
        multiple: false,
        title: `Import ${game.name} build guide`,
        filters: [
          {
            name: "Markdown",
            extensions: ["md", "markdown"]
          }
        ]
      });
      const markdownPath = Array.isArray(selected) ? selected[0] : selected;
      if (!markdownPath) {
        setStatus("Build guide import cancelled");
        return;
      }

      const imported = await importGameBuildGuideMarkdown(game.id, markdownPath);
      const guides = await listGameBuildGuides(game.id);
      setBuildGuides(guides);
      setStatus(
        `Imported "${imported.guide.title}" with ${imported.parts.length} part row(s) and ${imported.steps.length} step(s)`
      );
      showToast("Successful");
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setIsImportingBuildGuide(false);
    }
  }

  async function importBuildGuideFromUrl(game: Game) {
    const guideUrl = window.prompt("Steam build guide URL");
    if (!guideUrl?.trim()) {
      setStatus("Build guide URL import cancelled");
      return;
    }

    setIsImportingBuildGuideUrl(true);
    setStatus("Importing build guide URL");
    try {
      const imported = await importGameBuildGuideUrl(game.id, guideUrl.trim());
      const guides = await listGameBuildGuides(game.id);
      setBuildGuides(guides);
      setStatus(
        `Imported URL guide "${imported.guide.title}" with ${imported.parts.length} part row(s), ${imported.steps.length} step(s), and ${imported.imageReferenceCount} image reference(s)`
      );
      showToast("Successful");
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setIsImportingBuildGuideUrl(false);
    }
  }

  async function openBuildGuideOverlay(game: Game, guide: GameBuildGuide) {
    setIsOpeningBuildGuide(true);
    try {
      window.localStorage.setItem(
        "overlayForgeActiveBuildGuide",
        JSON.stringify({
          gameId: game.id,
          guideId: guide.id,
          openedAt: new Date().toISOString()
        })
      );
      await openGameBuildGuideOverlayWindow(game.id, guide.id);
      setStatus(`Opened build guide overlay: ${guide.title}`);
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setIsOpeningBuildGuide(false);
    }
  }

  async function removeBuildGuide(guide: GameBuildGuide) {
    if (!window.confirm(`Delete build guide "${guide.title}"?`)) {
      return;
    }

    setDeletingBuildGuideId(guide.id);
    try {
      await deleteGameBuildGuide(guide.id);
      setBuildGuides((current) => current.filter((item) => item.id !== guide.id));
      clearStoredBuildGuideSelection(guide.id);
      setStatus("Build guide deleted");
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setDeletingBuildGuideId(null);
    }
  }

  async function refreshPathOfExile2Builds(gameId: number) {
    try {
      const builds = await listGameCharacterBuilds(gameId);
      setPathOfExile2Builds(builds);
      return builds;
    } catch (error) {
      setStatus(formatError(error));
      return null;
    }
  }

  function startNewPathOfExile2Build() {
    setEditingPathOfExile2BuildId(null);
    setPathOfExile2BuildDraft({
      ...EMPTY_PATH_OF_EXILE_2_BUILD_DRAFT,
      status: "planned",
      isActive: pathOfExile2Builds.length === 0
    });
  }

  function editPathOfExile2Build(build: GameCharacterBuild) {
    setEditingPathOfExile2BuildId(build.id);
    setPathOfExile2BuildDraft(gameCharacterBuildToDraft(build));
  }

  async function savePathOfExile2Build(game: Game) {
    if (game.slug !== PATH_OF_EXILE_2_SLUG) {
      return;
    }
    if (!pathOfExile2BuildDraft.title.trim()) {
      setStatus("Build title is required");
      return;
    }

    setIsSavingPathOfExile2Build(true);
    try {
      const input = gameCharacterBuildDraftToInput(game.id, pathOfExile2BuildDraft);
      const saved =
        editingPathOfExile2BuildId === null
          ? await createGameCharacterBuild(input)
          : await updateGameCharacterBuild(editingPathOfExile2BuildId, input);
      await refreshPathOfExile2Builds(game.id);
      setEditingPathOfExile2BuildId(saved.id);
      setPathOfExile2BuildDraft(gameCharacterBuildToDraft(saved));
      setStatus(`Saved POE2 build "${saved.title}"`);
      showToast("Build saved");
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setIsSavingPathOfExile2Build(false);
    }
  }

  async function activatePathOfExile2Build(game: Game, build: GameCharacterBuild) {
    if (game.slug !== PATH_OF_EXILE_2_SLUG) {
      return;
    }
    try {
      const active = await setActiveGameCharacterBuild(build.id);
      await refreshPathOfExile2Builds(game.id);
      setStatus(`Active POE2 build set to "${active.title}"`);
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function removePathOfExile2Build(game: Game, build: GameCharacterBuild) {
    if (!window.confirm(`Delete POE2 build "${build.title}"?`)) {
      return;
    }

    setDeletingPathOfExile2BuildId(build.id);
    try {
      await deleteGameCharacterBuild(build.id);
      await refreshPathOfExile2Builds(game.id);
      if (editingPathOfExile2BuildId === build.id) {
        setEditingPathOfExile2BuildId(null);
        setPathOfExile2BuildDraft(EMPTY_PATH_OF_EXILE_2_BUILD_DRAFT);
      }
      setStatus("POE2 build deleted");
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setDeletingPathOfExile2BuildId(null);
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

  async function refreshGearBlocksApiCatalog() {
    setIsLoadingGearBlocksApiCatalog(true);
    try {
      const catalog = await listGearBlocksApiCatalog();
      setGearBlocksApiCatalog(catalog);
      setSelectedGearBlocksApiTypeId((current) =>
        current && catalog.types.some((type) => type.id === current)
          ? current
          : catalog.types[0]?.id ?? null
      );
    } catch (error) {
      setStatus(formatError(error));
    } finally {
      setIsLoadingGearBlocksApiCatalog(false);
    }
  }

  async function importOfficialGearBlocksApiDocs() {
    setIsImportingGearBlocksApiDocs(true);
    try {
      const result = await importGearBlocksOfficialApiDocs();
      setStatus(
        `Imported ${result.importedTypeCount} GearBlocks API types, ${result.importedMemberCount} members, ${result.importedParameterCount} parameters, and ${result.importedEnumValueCount} enum values from ${result.fetchedPages} official docs pages.`
      );
      await refreshGearBlocksApiCatalog();
    } catch (error) {
      setStatus(formatError(error));
    } finally {
      setIsImportingGearBlocksApiDocs(false);
    }
  }

  async function refreshGearBlocksDependencyStatus(gameId: number) {
    setIsLoadingGearBlocksDependencyStatus(true);
    try {
      const dependencyStatus = await getGearBlocksThirdPartyDependencyStatus(gameId);
      setGearBlocksDependencyStatus(dependencyStatus);
    } catch (error) {
      setStatus(formatError(error));
      setGearBlocksDependencyStatus(null);
    } finally {
      setIsLoadingGearBlocksDependencyStatus(false);
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
        `Refreshed GearBlocks scene context and indexed ${indexedParts.length} unique part reference(s)`
      );
      showToast("Successful");
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setIsImportingRuntimeExports(false);
    }
  }

  async function refreshGearBlocksSceneContext(game: Game) {
    setIsImportingRuntimeExports(true);
    try {
      const sync = await syncGearBlocksRuntimeContext(game.id);
      const categories = await listGamePartCategories(game.id);
      setPartCategories(categories);
      setStatus(
        `Refreshed GearBlocks scene context: ${sync.runtimePartCount} runtime part reference(s), ${sync.runtimeExportCount} scene export(s)`
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
          {selectedGame.slug === "gearblocks" && (
            <button
              className={gameView === "api" ? "primary-button" : "ghost-button"}
              onClick={() => setGameView("api")}
              type="button"
            >
              API
              <span className="button-count">{gearBlocksApiCatalog?.types.length ?? 0}</span>
            </button>
          )}
          {selectedGame.slug === "gearblocks" && (
            <button
              className={gameView === "tools" ? "primary-button" : "ghost-button"}
              onClick={() => setGameView("tools")}
              type="button"
            >
              Tools
            </button>
          )}
          {selectedGame.slug === "gearblocks" && (
            <button
              className={gameView === "build-guides" ? "primary-button" : "ghost-button"}
              onClick={() => setGameView("build-guides")}
              type="button"
            >
              Build Guides
              <span className="button-count">{buildGuides.length}</span>
            </button>
          )}
          {selectedGame.slug === PATH_OF_EXILE_2_SLUG ? (
            PATH_OF_EXILE_2_SECTIONS.map((section) => (
              <button
                className={gameView === section.view ? "primary-button" : "ghost-button"}
                key={section.view}
                onClick={() => setGameView(section.view)}
                type="button"
              >
                {section.label}
              </button>
            ))
          ) : (
            <>
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
            </>
          )}
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
              {selectedGame.slug === "gearblocks" && ENABLE_GEARBLOCKS_PLUGIN_STATUS && (
                <section
                  className="game-third-party-panel"
                  aria-label="GearBlocks third-party dependencies"
                >
                  <div className="game-data-locations-head">
                    <div>
                      <p>Third-party Mods</p>
                      <h4>BepInEx and GearLib</h4>
                    </div>
                    <button
                      className="ghost-button"
                      disabled={isLoadingGearBlocksDependencyStatus}
                      onClick={() => void refreshGearBlocksDependencyStatus(selectedGame.id)}
                      type="button"
                    >
                      Refresh
                    </button>
                  </div>
                  <p>
                    Overlay Forge does not bundle or install these third-party dependencies. Install
                    them separately from their official sources before using GearLib-based features.
                  </p>
                  <code>{gearBlocksDependencyStatus?.gameRoot || "GearBlocks install root not detected"}</code>
                  <div className="game-third-party-list">
                    {gearBlocksDependencyStatus?.dependencies.map((dependency) => (
                      <article
                        className={
                          dependency.isDetected
                            ? "game-third-party-row detected"
                            : "game-third-party-row"
                        }
                        key={dependency.name}
                      >
                        <div>
                          <strong>{dependency.name}</strong>
                          <span>{dependency.detail}</span>
                          {dependency.installedVersion && (
                            <span>Version: {dependency.installedVersion}</span>
                          )}
                          {dependency.isInstalledCorrectly !== null && (
                            <span>
                              Install check:{" "}
                              {dependency.isInstalledCorrectly ? "Correct" : "Needs attention"}
                            </span>
                          )}
                          {dependency.isActivated !== null && (
                            <span>
                              Activation:{" "}
                              {dependency.isActivated ? "Successful" : "Not confirmed"}
                            </span>
                          )}
                          <code>{dependency.expectedPath}</code>
                          {dependency.logPaths.map((logPath) => (
                            <code key={logPath}>{logPath}</code>
                          ))}
                        </div>
                        <span>{dependency.isDetected ? "Detected" : "Not detected"}</span>
                      </article>
                    ))}
                    {!gearBlocksDependencyStatus && (
                      <p>
                        {isLoadingGearBlocksDependencyStatus
                          ? "Checking third-party dependency status."
                          : "Dependency status has not been checked."}
                      </p>
                    )}
                  </div>
                </section>
              )}
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
              {selectedGame.slug === PATH_OF_EXILE_2_SLUG && <PathOfExile2HomeScaffold />}
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

          {gameView === "api" && selectedGame.slug === "gearblocks" && (
            <section className="game-api-panel" aria-label="GearBlocks API catalog">
              <div className="game-view-head">
                <div>
                  <p>GearBlocks</p>
                  <h3>API Catalog</h3>
                </div>
                <div className="game-view-actions">
                  <button
                    className="ghost-button"
                    disabled={isLoadingGearBlocksApiCatalog || isImportingGearBlocksApiDocs}
                    onClick={() => void refreshGearBlocksApiCatalog()}
                    type="button"
                  >
                    Refresh
                  </button>
                  <button
                    className="ghost-button"
                    disabled={isLoadingGearBlocksApiCatalog || isImportingGearBlocksApiDocs}
                    onClick={() => void importOfficialGearBlocksApiDocs()}
                    type="button"
                  >
                    {isImportingGearBlocksApiDocs ? "Importing" : "Import Official Docs"}
                  </button>
                </div>
              </div>

              {!gearBlocksApiCatalog ? (
                <p>Loading GearBlocks API catalog.</p>
              ) : (
                <>
                  <div className="game-api-summary-grid" aria-label="GearBlocks API summary">
                    <span>{gearBlocksApiCatalog.types.length} types</span>
                    <span>{gearBlocksApiCatalog.members.length} members</span>
                    <span>{gearBlocksApiCatalog.parameters.length} parameters</span>
                    <span>{gearBlocksApiCatalog.enumValues.length} enum values</span>
                  </div>

                  <div className="game-api-layout">
                    <nav className="game-api-type-list" aria-label="GearBlocks API types">
                      {gearBlocksApiCatalog.types.map((apiType) => (
                        <button
                          className={
                            selectedGearBlocksApiType?.id === apiType.id
                              ? "game-api-type-row active"
                              : "game-api-type-row"
                          }
                          key={apiType.id}
                          onClick={() => setSelectedGearBlocksApiTypeId(apiType.id)}
                          type="button"
                        >
                          <span>{apiType.typeName}</span>
                          <small>
                            {apiType.typeKind}
                            {apiType.memberCount > 0 ? ` · ${apiType.memberCount}` : ""}
                            {apiType.enumValueCount > 0 ? ` · ${apiType.enumValueCount}` : ""}
                          </small>
                        </button>
                      ))}
                    </nav>

                    <div className="game-api-detail">
                      {selectedGearBlocksApiType ? (
                        <>
                          <div className="game-api-detail-head">
                            <div>
                              <p>{selectedGearBlocksApiType.namespace}</p>
                              <h4>{selectedGearBlocksApiType.typeName}</h4>
                            </div>
                            <span>{selectedGearBlocksApiType.typeKind}</span>
                          </div>

                          {selectedGearBlocksApiType.notes && (
                            <p className="game-api-notes">{selectedGearBlocksApiType.notes}</p>
                          )}

                          {selectedGearBlocksApiMembers.length > 0 && (
                            <div className="game-api-member-list">
                              {selectedGearBlocksApiMembers.map((member) => (
                                <article className="game-api-member-row" key={member.id}>
                                  <div>
                                    <strong>{member.memberName}</strong>
                                    <code>{member.signature || member.memberKey}</code>
                                    {selectedGearBlocksApiMemberParameters.has(member.id) && (
                                      <small>
                                        {selectedGearBlocksApiMemberParameters
                                          .get(member.id)
                                          ?.join(", ")}
                                      </small>
                                    )}
                                  </div>
                                  <ApiMemberFlags member={member} />
                                </article>
                              ))}
                            </div>
                          )}

                          {selectedGearBlocksApiEnumValues.length > 0 && (
                            <div className="game-api-enum-list">
                              {selectedGearBlocksApiEnumValues.map((value) => (
                                <article className="game-api-enum-row" key={value.id}>
                                  <div>
                                    <strong>{value.valueName}</strong>
                                    <span>{value.description}</span>
                                  </div>
                                  <code>{value.luaName || value.numericValue || "value"}</code>
                                </article>
                              ))}
                            </div>
                          )}

                          {selectedGearBlocksApiMembers.length === 0 &&
                            selectedGearBlocksApiEnumValues.length === 0 && (
                              <p>No members or enum values indexed for this type yet.</p>
                            )}
                        </>
                      ) : (
                        <p>No GearBlocks API types indexed yet.</p>
                      )}
                    </div>
                  </div>
                </>
              )}
            </section>
          )}

          {gameView === "tools" && selectedGame.slug === "gearblocks" && (
            <section className="game-script-tools-panel" aria-label="GearBlocks overlay tools">
              <div className="game-view-head">
                <div>
                  <p>GearBlocks</p>
                  <h3>Tools</h3>
                </div>
                <button
                  className="ghost-button"
                  disabled={isInstallingLuaExporter}
                  onClick={() => void installGearBlocksExporter(selectedGame)}
                  type="button"
                >
                  Install Overlay Forge Script
                </button>
              </div>

              {luaExporterPath && (
                <div className="game-script-tools-status">
                  <span>Overlay Forge Script</span>
                  <code>{luaExporterPath}</code>
                </div>
              )}

              <div className="game-script-tools-grid">
                <article className="game-script-tool-card">
                  <div>
                    <p>Scene</p>
                    <h4>Scene Context</h4>
                  </div>
                  <p>Full live-scene export for chat context.</p>
                </article>

                <article className="game-script-tool-card">
                  <div>
                    <p>BuilderToolExt</p>
                    <h4>Builder Tool</h4>
                  </div>
                  <p>Orientation, step intervals, pivot transforms, sizing, and attachment visibility.</p>
                </article>

                <article className="game-script-tool-card">
                  <div>
                    <p>WeldTool</p>
                    <h4>Weld Tool</h4>
                  </div>
                  <p>Attachment type, start/complete weld, detach, and targeting feedback.</p>
                </article>
              </div>
            </section>
          )}

          {gameView === "build-guides" && selectedGame.slug === "gearblocks" && (
            <section className="game-build-guides-panel" aria-label="GearBlocks build guides">
              <div className="game-view-head">
                <div>
                  <p>GearBlocks</p>
                  <h3>Build Guides</h3>
                </div>
                <div className="game-build-guide-header-actions">
                  <button
                    className="primary-button"
                    disabled={isImportingBuildGuide || isImportingBuildGuideUrl}
                    onClick={() => void importBuildGuideFromUrl(selectedGame)}
                    type="button"
                  >
                    {isImportingBuildGuideUrl
                      ? buildGuideImportProgressLabel(buildGuideImportProgressFrame)
                      : "Import URL"}
                  </button>
                  <button
                    className="primary-button"
                    disabled={isImportingBuildGuide || isImportingBuildGuideUrl}
                    onClick={() => void importBuildGuide(selectedGame)}
                    type="button"
                  >
                    {isImportingBuildGuide
                      ? buildGuideImportProgressLabel(buildGuideImportProgressFrame)
                      : "Import Markdown"}
                  </button>
                </div>
              </div>

              <p>
                Build guides are rendered in a separate narrow overlay window for in-game assembly
                reference. Import a Markdown handoff, then open it and pin the overlay to the side
                of the screen.
              </p>

              {buildGuides.length === 0 ? (
                <div className="game-build-guides-empty">
                  <strong>No build guides imported yet.</strong>
                  <span>Use Import Markdown with a vehicle handoff or construction guide.</span>
                </div>
              ) : (
                <div className="game-build-guides-list">
                  {buildGuides.map((guide) => (
                    <article className="game-build-guide-row" key={guide.id}>
                      <div>
                        <strong>{guide.title}</strong>
                        <span>{guide.buildGoal || "No build goal summary found."}</span>
                        <code>{guide.sourcePath || "Imported Markdown"}</code>
                      </div>
                      <div className="game-build-guide-actions">
                        <button
                          className="primary-button"
                          disabled={isOpeningBuildGuide || deletingBuildGuideId === guide.id}
                          onClick={() => void openBuildGuideOverlay(selectedGame, guide)}
                          type="button"
                        >
                          Open
                        </button>
                        <button
                          aria-label={`Delete ${guide.title}`}
                          className="primary-button"
                          disabled={deletingBuildGuideId === guide.id}
                          onClick={() => void removeBuildGuide(guide)}
                          type="button"
                        >
                          Delete
                        </button>
                      </div>
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
                <>
                  {selectedGame.slug === "gearblocks" && (
                    <section className="chat-screenshot-context" aria-label="Scene context">
                      <div className="chat-screenshot-summary">
                        <div>
                          <p>Scene context</p>
                          <strong>Latest indexed GearBlocks export</strong>
                        </div>
                        <button
                          className="ghost-button"
                          disabled={isImportingRuntimeExports}
                          onClick={() => void refreshGearBlocksSceneContext(selectedGame)}
                          type="button"
                        >
                          Refresh Scene Context
                        </button>
                        <button
                          className="ghost-button"
                          disabled={
                            isGeneratingBuildGuide ||
                            isSendingChat ||
                            !selectedChatConversationId
                          }
                          onClick={() => void generateBuildGuideFromChat()}
                          type="button"
                        >
                          Generate Build Guide
                        </button>
                        {ENABLE_GEARBLOCKS_MARKERS && (
                          <button
                            className="ghost-button"
                            disabled={isSendingGearBlocksMarkers}
                            onClick={() => void clearGearBlocksChatMarkers()}
                            type="button"
                          >
                            Clear Markers
                          </button>
                        )}
                      </div>
                    </section>
                  )}
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
                </>
              }
              draft={chatDraft}
              emptyMainSlot={
                <GameChatDefaultsPane
                  game={selectedGame}
                  isRefreshingSceneContext={isImportingRuntimeExports}
                  onRefreshSceneContext={() => void refreshGearBlocksSceneContext(selectedGame)}
                />
              }
              emptyConversationLabel="No game chats yet."
              hideSidebarWhenSelected
              inputPlaceholder={`Ask about ${selectedGame.name}...`}
              inputActionSlot={
                selectedGame.slug === "gearblocks" ? (
                  <>
                    <button
                      aria-label="Generate build guide"
                      className="ghost-button chat-input-extra-action"
                      disabled={
                        isGeneratingBuildGuide ||
                        isSendingChat ||
                        !selectedChatConversationId
                      }
                      onClick={() => void generateBuildGuideFromChat()}
                      title="Guide"
                      type="button"
                    >
                      G
                    </button>
                    <button
                      aria-label="Send with latest scene diff"
                      className="ghost-button chat-input-extra-action"
                      disabled={
                        isGeneratingBuildGuide ||
                        isSendingChat ||
                        !selectedChatConversationId ||
                        chatDraft.trim().length === 0
                      }
                      onClick={() => void sendGameChat(true)}
                      title="Diff"
                      type="button"
                    >
                      D↑
                    </button>
                  </>
                ) : null
              }
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
              onSendMessage={() => void sendGameChat(false)}
              renderMessageContent={(message) => {
                const content =
                  selectedGame.slug === "gearblocks" && message.role === "assistant"
                    ? stripGearBlocksMarkerBlocks(message.content)
                    : message.content;
                return <p>{content}</p>;
              }}
              renderMessageActions={(message) => {
                if (
                  !ENABLE_GEARBLOCKS_MARKERS ||
                  selectedGame.slug !== "gearblocks" ||
                  message.role !== "assistant"
                ) {
                  return null;
                }

                const markers = extractGearBlocksMarkers(message.content);
                if (markers.length === 0) {
                  return null;
                }

                return (
                  <div className="chat-message-actions">
                    <button
                      className="ghost-button"
                      disabled={isSendingGearBlocksMarkers}
                      onClick={() => void sendGearBlocksMarkersFromMessage(message)}
                      type="button"
                    >
                      Send {markers.length} Marker{markers.length === 1 ? "" : "s"}
                    </button>
                  </div>
                );
              }}
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

          {selectedGame.slug === PATH_OF_EXILE_2_SLUG &&
            PATH_OF_EXILE_2_SECTIONS.map((section) =>
              gameView === section.view ? (
                <PathOfExile2SectionScaffold
                  builds={pathOfExile2Builds}
                  deletingBuildId={deletingPathOfExile2BuildId}
                  draft={pathOfExile2BuildDraft}
                  editingBuildId={editingPathOfExile2BuildId}
                  game={selectedGame}
                  isSaving={isSavingPathOfExile2Build}
                  key={section.view}
                  onActivateBuild={activatePathOfExile2Build}
                  onDeleteBuild={removePathOfExile2Build}
                  onDraftChange={setPathOfExile2BuildDraft}
                  onEditBuild={editPathOfExile2Build}
                  onNewBuild={startNewPathOfExile2Build}
                  onSaveBuild={savePathOfExile2Build}
                  section={section}
                />
              ) : null
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
                        Refresh Scene Context
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
                      Export the scene from the GearBlocks Lua script, then refresh the runtime log
                      to index the latest Player.log scene context.
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
                                <p>Canonical API Members</p>
                                <h4>
                                  {selectedRuntimePartApiMembers.length ||
                                    runtimePartAvailableAttributes(selectedRuntimePart).length}{" "}
                                  indexed member(s)
                                </h4>
                              </div>
                              {isLoadingRuntimePartApiMembers ? (
                                <p>Loading indexed API members.</p>
                              ) : selectedRuntimePartApiMembers.length > 0 ? (
                                <div className="game-runtime-part-api-member-list">
                                  {selectedRuntimePartApiMembers.map((member) => (
                                    <article
                                      className="game-runtime-part-api-member-row"
                                      key={member.id}
                                    >
                                      <div>
                                        <strong>
                                          {member.typeName}.{member.memberName}
                                        </strong>
                                        <code>{member.signature || member.memberKey}</code>
                                        <small>
                                          {member.availability || "available"} · last seen{" "}
                                          {member.lastSeenAt || "unknown"}
                                        </small>
                                      </div>
                                      <RuntimePartApiMemberFlags member={member} />
                                    </article>
                                  ))}
                                </div>
                              ) : (
                                <div className="game-runtime-part-attribute-list">
                                  {runtimePartAvailableAttributes(selectedRuntimePart).length > 0 ? (
                                    runtimePartAvailableAttributes(selectedRuntimePart).map(
                                      (attribute) => <span key={attribute}>{attribute}</span>
                                    )
                                  ) : (
                                    <span>No runtime API attributes indexed yet</span>
                                  )}
                                </div>
                              )}
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

function stripGearBlocksMarkerBlocks(content: string) {
  return content
    .replace(/```overlay-forge-markers\s*[\s\S]*?```/gi, "")
    .trim();
}

function extractGearBlocksMarkers(content: string): GearBlocksMarkerInput[] {
  const markers: GearBlocksMarkerInput[] = [];
  const blockPattern = /```overlay-forge-markers\s*([\s\S]*?)```/gi;
  let match: RegExpExecArray | null;

  while ((match = blockPattern.exec(content)) !== null) {
    try {
      const parsed = JSON.parse(match[1]) as unknown;
      const markerList = Array.isArray(parsed)
        ? parsed
        : isRecord(parsed) && Array.isArray(parsed.markers)
          ? parsed.markers
          : [];

      for (const marker of markerList) {
        if (!isRecord(marker)) {
          continue;
        }

        const x = Number(marker.x);
        const y = Number(marker.y);
        const z = Number(marker.z);
        if (!Number.isFinite(x) || !Number.isFinite(y) || !Number.isFinite(z)) {
          continue;
        }

        markers.push({
          label: typeof marker.label === "string" ? marker.label : undefined,
          reason: typeof marker.reason === "string" ? marker.reason : undefined,
          x,
          y,
          z,
          color: typeof marker.color === "string" ? marker.color : undefined,
          durationSeconds:
            Number.isFinite(Number(marker.durationSeconds)) && marker.durationSeconds !== undefined
              ? Number(marker.durationSeconds)
              : undefined,
          size:
            Number.isFinite(Number(marker.size)) && marker.size !== undefined
              ? Number(marker.size)
              : undefined
        });
      }
    } catch {
      continue;
    }
  }

  return markers;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function parseJsonOrFallback(value: string, fallback: unknown) {
  try {
    return JSON.parse(value);
  } catch {
    return fallback;
  }
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

function ApiMemberFlags({ member }: { member: GearBlocksApiMember }) {
  const flags = [
    member.memberKind,
    member.isReadable ? "read" : "",
    member.isWritable ? "write" : "",
    member.isInvokable ? "call" : "",
    member.isMutating ? "mutates" : ""
  ].filter(Boolean);

  return (
    <div className="game-api-member-flags">
      {flags.map((flag) => (
        <span key={flag}>{flag}</span>
      ))}
    </div>
  );
}

function RuntimePartApiMemberFlags({ member }: { member: GameRuntimePartApiMember }) {
  const flags = [
    member.memberKind,
    member.isReadable ? "read" : "",
    member.isWritable ? "write" : "",
    member.isInvokable ? "call" : "",
    member.isMutating ? "mutates" : ""
  ].filter(Boolean);

  return (
    <div className="game-api-member-flags">
      {flags.map((flag) => (
        <span key={flag}>{flag}</span>
      ))}
    </div>
  );
}

function PathOfExile2HomeScaffold() {
  return (
    <section className="poe2-home-scaffold" aria-label="Path of Exile 2 module sections">
      <div className="game-data-locations-head">
        <div>
          <p>Module Scaffold</p>
          <h4>Path of Exile 2</h4>
        </div>
      </div>
      <div className="poe2-section-grid">
        {PATH_OF_EXILE_2_SECTIONS.map((section) => (
          <article className="poe2-section-card" key={section.view}>
            <div>
              <p>{section.eyebrow}</p>
              <h4>{section.label}</h4>
            </div>
            <span>{section.description}</span>
          </article>
        ))}
      </div>
    </section>
  );
}

function PathOfExile2SectionScaffold({
  builds,
  deletingBuildId,
  draft,
  editingBuildId,
  game,
  isSaving,
  onActivateBuild,
  onDeleteBuild,
  onDraftChange,
  onEditBuild,
  onNewBuild,
  onSaveBuild,
  section
}: {
  builds: GameCharacterBuild[];
  deletingBuildId: number | null;
  draft: PathOfExile2BuildDraft;
  editingBuildId: number | null;
  game: Game;
  isSaving: boolean;
  onActivateBuild: (game: Game, build: GameCharacterBuild) => void | Promise<void>;
  onDeleteBuild: (game: Game, build: GameCharacterBuild) => void | Promise<void>;
  onDraftChange: (draft: PathOfExile2BuildDraft) => void;
  onEditBuild: (build: GameCharacterBuild) => void;
  onNewBuild: () => void;
  onSaveBuild: (game: Game) => void | Promise<void>;
  section: (typeof PATH_OF_EXILE_2_SECTIONS)[number];
}) {
  const isBuildsSection = section.view === "builds";

  return (
    <section className="poe2-section-panel" aria-label={`Path of Exile 2 ${section.label}`}>
      <div className="game-view-head">
        <div>
          <p>{section.eyebrow}</p>
          <h3>{section.label}</h3>
        </div>
      </div>
      {isBuildsSection ? (
        <PathOfExile2BuildsPanel
          builds={builds}
          deletingBuildId={deletingBuildId}
          draft={draft}
          editingBuildId={editingBuildId}
          game={game}
          isSaving={isSaving}
          onActivateBuild={onActivateBuild}
          onDeleteBuild={onDeleteBuild}
          onDraftChange={onDraftChange}
          onEditBuild={onEditBuild}
          onNewBuild={onNewBuild}
          onSaveBuild={onSaveBuild}
        />
      ) : (
        <article className="poe2-section-card large">
          <div>
            <p>Scaffold</p>
            <h4>{section.label}</h4>
          </div>
          <span>{section.description}</span>
        </article>
      )}
    </section>
  );
}

function PathOfExile2BuildsPanel({
  builds,
  deletingBuildId,
  draft,
  editingBuildId,
  game,
  isSaving,
  onActivateBuild,
  onDeleteBuild,
  onDraftChange,
  onEditBuild,
  onNewBuild,
  onSaveBuild
}: {
  builds: GameCharacterBuild[];
  deletingBuildId: number | null;
  draft: PathOfExile2BuildDraft;
  editingBuildId: number | null;
  game: Game;
  isSaving: boolean;
  onActivateBuild: (game: Game, build: GameCharacterBuild) => void | Promise<void>;
  onDeleteBuild: (game: Game, build: GameCharacterBuild) => void | Promise<void>;
  onDraftChange: (draft: PathOfExile2BuildDraft) => void;
  onEditBuild: (build: GameCharacterBuild) => void;
  onNewBuild: () => void;
  onSaveBuild: (game: Game) => void | Promise<void>;
}) {
  const updateDraft = <K extends keyof PathOfExile2BuildDraft>(
    key: K,
    value: PathOfExile2BuildDraft[K]
  ) => {
    onDraftChange({ ...draft, [key]: value });
  };

  return (
    <div className="poe2-build-workspace">
      <section className="poe2-build-list" aria-label="Path of Exile 2 builds">
        <div className="poe2-build-list-head">
          <div>
            <p>Local Builds</p>
            <h4>{builds.length} build{builds.length === 1 ? "" : "s"}</h4>
          </div>
          <button className="ghost-button" onClick={onNewBuild} type="button">
            New Build
          </button>
        </div>

        {builds.length === 0 ? (
          <article className="poe2-section-card large">
            <div>
              <p>Planner</p>
              <h4>No builds saved</h4>
            </div>
            <span>Local character builds will appear here.</span>
          </article>
        ) : (
          <div className="poe2-build-card-list">
            {builds.map((build) => (
              <article
                className={
                  build.id === editingBuildId ? "poe2-build-card selected" : "poe2-build-card"
                }
                key={build.id}
              >
                <div className="poe2-build-card-head">
                  <div>
                    <p>{build.isActive ? "Active" : formatPathOfExile2BuildStatus(build.status)}</p>
                    <h4>{build.title}</h4>
                  </div>
                  {build.patch && <span>{build.patch}</span>}
                </div>
                <div className="poe2-current-build-meta">
                  {build.characterClass && <span>{build.characterClass}</span>}
                  {build.ascendancy && <span>{build.ascendancy}</span>}
                  {build.buildRole && <span>{build.buildRole}</span>}
                </div>
                {build.summary && <p>{build.summary}</p>}
                {splitPathOfExile2BuildTags(build.tags).length > 0 && (
                  <div className="poe2-current-build-tags">
                    {splitPathOfExile2BuildTags(build.tags).map((tag) => (
                      <span key={tag}>{tag}</span>
                    ))}
                  </div>
                )}
                <div className="poe2-current-build-actions">
                  <button className="ghost-button" onClick={() => onEditBuild(build)} type="button">
                    Edit
                  </button>
                  <button
                    className="ghost-button"
                    disabled={build.isActive}
                    onClick={() => void onActivateBuild(game, build)}
                    type="button"
                  >
                    Set Active
                  </button>
                  {build.sourceUrl && (
                    <a href={build.sourceUrl} rel="noreferrer" target="_blank">
                      Open {build.sourceLabel || "Source"}
                    </a>
                  )}
                  <button
                    className="ghost-button"
                    disabled={deletingBuildId === build.id}
                    onClick={() => void onDeleteBuild(game, build)}
                    type="button"
                  >
                    Delete
                  </button>
                </div>
              </article>
            ))}
          </div>
        )}
      </section>

      <section className="poe2-build-editor" aria-label="Path of Exile 2 build editor">
        <div className="poe2-build-list-head">
          <div>
            <p>{editingBuildId === null ? "New Build" : "Build Details"}</p>
            <h4>{draft.title || "Untitled build"}</h4>
          </div>
          <label className="toggle-row compact">
            <input
              checked={draft.isActive}
              onChange={(event) => updateDraft("isActive", event.target.checked)}
              type="checkbox"
            />
            <span>Active</span>
          </label>
        </div>

        <div className="poe2-build-form-grid">
          <label>
            <span>Title</span>
            <input
              className="text-input"
              onChange={(event) => updateDraft("title", event.target.value)}
              value={draft.title}
            />
          </label>
          <label>
            <span>Status</span>
            <select
              className="text-input"
              onChange={(event) => updateDraft("status", event.target.value)}
              value={draft.status}
            >
              <option value="planned">Planned</option>
              <option value="currently_playing">Currently Playing</option>
              <option value="active">Active</option>
              <option value="archived">Archived</option>
            </select>
          </label>
          <label>
            <span>Class</span>
            <input
              className="text-input"
              onChange={(event) => updateDraft("characterClass", event.target.value)}
              value={draft.characterClass}
            />
          </label>
          <label>
            <span>Ascendancy</span>
            <input
              className="text-input"
              onChange={(event) => updateDraft("ascendancy", event.target.value)}
              value={draft.ascendancy}
            />
          </label>
          <label>
            <span>Role</span>
            <input
              className="text-input"
              onChange={(event) => updateDraft("buildRole", event.target.value)}
              value={draft.buildRole}
            />
          </label>
          <label>
            <span>Patch</span>
            <input
              className="text-input"
              onChange={(event) => updateDraft("patch", event.target.value)}
              value={draft.patch}
            />
          </label>
          <label>
            <span>Source</span>
            <input
              className="text-input"
              onChange={(event) => updateDraft("sourceLabel", event.target.value)}
              value={draft.sourceLabel}
            />
          </label>
          <label>
            <span>Source URL</span>
            <input
              className="text-input"
              onChange={(event) => updateDraft("sourceUrl", event.target.value)}
              value={draft.sourceUrl}
            />
          </label>
          <label className="wide">
            <span>Tags</span>
            <input
              className="text-input"
              onChange={(event) => updateDraft("tags", event.target.value)}
              value={draft.tags}
            />
          </label>
          <label className="wide">
            <span>Summary</span>
            <textarea
              className="text-area"
              onChange={(event) => updateDraft("summary", event.target.value)}
              value={draft.summary}
            />
          </label>
          <label className="wide">
            <span>Notes</span>
            <textarea
              className="text-area"
              onChange={(event) => updateDraft("notes", event.target.value)}
              value={draft.notes}
            />
          </label>
        </div>

        <div className="poe2-build-editor-actions">
          <button
            className="primary-button"
            disabled={isSaving || draft.title.trim().length === 0}
            onClick={() => void onSaveBuild(game)}
            type="button"
          >
            Save Build
          </button>
        </div>
      </section>
    </div>
  );
}

function gameCharacterBuildToDraft(build: GameCharacterBuild): PathOfExile2BuildDraft {
  return {
    title: build.title,
    characterClass: build.characterClass,
    ascendancy: build.ascendancy,
    buildRole: build.buildRole,
    status: build.status,
    sourceLabel: build.sourceLabel,
    sourceUrl: build.sourceUrl,
    patch: build.patch,
    summary: build.summary,
    tags: build.tags,
    notes: build.notes,
    isActive: build.isActive
  };
}

function gameCharacterBuildDraftToInput(
  gameId: number,
  draft: PathOfExile2BuildDraft
): GameCharacterBuildInput {
  return {
    gameId,
    title: draft.title,
    characterClass: draft.characterClass,
    ascendancy: draft.ascendancy,
    buildRole: draft.buildRole,
    status: draft.status,
    sourceLabel: draft.sourceLabel,
    sourceUrl: draft.sourceUrl,
    patch: draft.patch,
    summary: draft.summary,
    tags: draft.tags,
    notes: draft.notes,
    isActive: draft.isActive
  };
}

function splitPathOfExile2BuildTags(value: string) {
  return value
    .split(",")
    .map((tag) => tag.trim())
    .filter(Boolean);
}

function formatPathOfExile2BuildStatus(status: string) {
  return status
    .split("_")
    .map((part) => `${part.charAt(0).toUpperCase()}${part.slice(1)}`)
    .join(" ");
}

function buildGuideImportProgressLabel(frame: number) {
  return `Importing${".".repeat((frame % 3) + 1)}`;
}

function clearStoredBuildGuideSelection(deletedGuideId: number) {
  try {
    const value = window.localStorage.getItem("overlayForgeActiveBuildGuide");
    if (!value) {
      return;
    }
    const parsed = JSON.parse(value) as { guideId?: unknown };
    if (parsed.guideId === deletedGuideId) {
      window.localStorage.removeItem("overlayForgeActiveBuildGuide");
    }
  } catch {
    window.localStorage.removeItem("overlayForgeActiveBuildGuide");
  }
}

function GameChatDefaultsPane({
  game,
  isRefreshingSceneContext,
  onRefreshSceneContext
}: {
  game: Game;
  isRefreshingSceneContext: boolean;
  onRefreshSceneContext: () => void;
}) {
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
              <span>Refresh after running Export Scene to update chat context</span>
            </div>
            <button
              className="ghost-button"
              disabled={isRefreshingSceneContext}
              onClick={onRefreshSceneContext}
              type="button"
            >
              Refresh Scene Context
            </button>
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
