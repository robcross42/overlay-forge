import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useMemo, useState } from "react";
import type { MouseEvent, WheelEvent } from "react";
import { WindowResizeHandles } from "../components/WindowControls";
import {
  getActiveGameBuildGuideOverlay,
  getGameBuildGuide,
  listGames
} from "../services/gaming";
import type {
  Game,
  GameBuildGuideOverlaySelection,
  GameBuildGuidePart,
  GameBuildGuidePayload
} from "../services/gaming";
import { applyStandaloneOverlayFocusState, startOverlayDrag } from "../services/windowControls";
import { formatUnknownError as formatError } from "../utils/errors";

const ACTIVE_BUILD_GUIDE_STORAGE_KEY = "overlayForgeActiveBuildGuide";
const BUILD_GUIDE_FONT_SIZE_STORAGE_KEY = "overlayForgeBuildGuideFontSize";
const FONT_SIZES = ["small", "medium", "large"] as const;
type BuildGuideFontSize = (typeof FONT_SIZES)[number];

export default function GameBuildGuideOverlayApp() {
  const [game, setGame] = useState<Game | null>(null);
  const [payload, setPayload] = useState<GameBuildGuidePayload | null>(null);
  const [status, setStatus] = useState("Loading build guide");
  const [fontSize, setFontSize] = useState<BuildGuideFontSize>(loadStoredFontSize);

  const partsBySection = useMemo(() => {
    const groups = new Map<string, GameBuildGuidePart[]>();
    for (const part of payload?.parts ?? []) {
      if (!shouldDisplayPart(part)) {
        continue;
      }
      const section = part.section || "Parts";
      groups.set(section, [...(groups.get(section) ?? []), part]);
    }
    return Array.from(groups.entries());
  }, [payload?.parts]);

  useEffect(() => {
    let isMounted = true;
    let cleanup: (() => void) | null = null;
    void prepareBuildGuideOverlayWindow().then((nextCleanup) => {
      if (isMounted) {
        cleanup = nextCleanup;
      } else {
        nextCleanup?.();
      }
    });
    void loadActiveBuildGuide();

    return () => {
      isMounted = false;
      cleanup?.();
    };
  }, []);

  useEffect(() => {
    let isMounted = true;
    let cleanup: (() => void) | null = null;

    listen<GameBuildGuideOverlaySelection>("game-build-guide-overlay-selection-changed", (event) => {
      if (isMounted) {
        storeBuildGuideSelection(event.payload);
        void loadActiveBuildGuide();
      }
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
  }, []);

  async function loadActiveBuildGuide(attempt = 0) {
    try {
      const selection = loadStoredBuildGuideSelection() ?? (await getActiveGameBuildGuideOverlay());
      if (!selection) {
        setGame(null);
        setPayload(null);
        setStatus("Loading build guide");
        if (attempt < 8) {
          window.setTimeout(() => {
            void loadActiveBuildGuide(attempt + 1);
          }, 150);
        } else {
          setStatus("No selected build guide");
        }
        return;
      }

      const [games, nextPayload] = await Promise.all([
        listGames(),
        getGameBuildGuide(selection.guideId)
      ]);
      setGame(games.find((item) => item.id === selection.gameId) ?? null);
      setPayload(nextPayload);
      setStatus("Ready");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  function startGuideDrag(event: MouseEvent) {
    if (event.detail !== 1 || event.button !== 0) {
      return;
    }
    void startOverlayDrag();
  }

  function changeFontSize(event: WheelEvent) {
    if (!event.ctrlKey || !event.shiftKey) {
      return;
    }

    event.preventDefault();
    setFontSize((current) => {
      const currentIndex = FONT_SIZES.indexOf(current);
      const nextIndex =
        event.deltaY < 0
          ? Math.min(FONT_SIZES.length - 1, currentIndex + 1)
          : Math.max(0, currentIndex - 1);
      const next = FONT_SIZES[nextIndex];
      window.localStorage.setItem(BUILD_GUIDE_FONT_SIZE_STORAGE_KEY, next);
      return next;
    });
  }

  return (
    <main
      className={`overlay-frame overlay-frame-chat-mode standalone-overlay-window build-guide-overlay-frame build-guide-font-${fontSize}`}
      onMouseDownCapture={() => {
        void getCurrentWindow().setIgnoreCursorEvents(false);
      }}
      onWheel={changeFontSize}
    >
      <WindowResizeHandles />
      <div className="build-guide-overlay-shell">
        <div
          className="overlay-window-titlebar build-guide-title-drag-box"
          onMouseDown={startGuideDrag}
          role="presentation"
        >
          <h1>{payload?.guide.title ?? status}</h1>
          {game && <span>{game.name}</span>}
        </div>

        <section className="build-guide-overlay-content" aria-label="Build guide">
          {payload && (
            <>
              <div className="build-guide-meta-row">
                <span>{payload.parts.length} parts</span>
                <span>{payload.steps.length} steps</span>
                <span>{status}</span>
              </div>

              <GuideTextSection title="Goal" value={payload.guide.buildGoal} />
              <GuideTextSection title="Scale" value={payload.guide.scaleReference} />
              <GuideTextSection title="Geometry" value={payload.guide.geometryNotes} />
              <GuideTextSection title="Glossary" value={payload.guide.glossaryText} />

              <section className="build-guide-section" aria-label="Parts list">
                <h2>Parts</h2>
                {partsBySection.length === 0 ? (
                  <p>No structured parts found in this guide.</p>
                ) : (
                  partsBySection.map(([section, parts]) => (
                    <article className="build-guide-part-group" key={section}>
                      <h3>{section}</h3>
                      <div className="build-guide-part-list">
                        {parts.map((part) => (
                          <div className="build-guide-part-row" key={part.id}>
                            <span className="build-guide-qty">{part.quantity || "-"}</span>
                            <div>
                              <strong>{cleanMarkdownLine(part.partName)}</strong>
                              {cleanMarkdownLine(part.purpose) && (
                                <p>{cleanMarkdownLine(part.purpose)}</p>
                              )}
                            </div>
                          </div>
                        ))}
                      </div>
                    </article>
                  ))
                )}
              </section>

              <section className="build-guide-section" aria-label="Assembly instructions">
                <h2>Steps</h2>
                {payload.steps.length === 0 ? (
                  <p>No structured assembly steps found in this guide.</p>
                ) : (
                  <div className="build-guide-step-list">
                    {payload.steps.map((step) => (
                      <article className="build-guide-step" key={step.id}>
                        <div className="build-guide-step-head">
                          <span>{step.stepNumber}</span>
                          <h3>{step.title || `Step ${step.stepNumber}`}</h3>
                        </div>
                        {step.body && <MarkdownLines value={step.body} />}
                      </article>
                    ))}
                  </div>
                )}
              </section>

              {payload.checklist.length > 0 && (
                <section className="build-guide-section" aria-label="First test checklist">
                  <h2>Checklist</h2>
                  <ul className="build-guide-checklist">
                    {payload.checklist.map((item) => (
                      <li key={item}>{item}</li>
                    ))}
                  </ul>
                </section>
              )}
            </>
          )}
        </section>
      </div>
    </main>
  );
}

function GuideTextSection({ title, value }: { title: string; value: string }) {
  if (!value.trim()) {
    return null;
  }
  return (
    <section className="build-guide-section">
      <h2>{title}</h2>
      <MarkdownLines value={value} />
    </section>
  );
}

function MarkdownLines({ value }: { value: string }) {
  const lines = value
    .split(/\r?\n/)
    .map(cleanMarkdownLine)
    .filter(shouldDisplayMarkdownLine);

  return (
    <>
      {lines.map((line, index) => (
        <p key={`${index}:${line}`}>{line}</p>
      ))}
    </>
  );
}

function shouldDisplayPart(part: GameBuildGuidePart) {
  const partName = cleanMarkdownLine(part.partName);
  if (!partName) {
    return false;
  }
  return shouldDisplayMarkdownLine(partName) && !isMarkdownTableSeparator(partName);
}

function cleanMarkdownLine(value: string) {
  return value
    .trim()
    .replace(/^[-*]\s+/, "")
    .replace(/^`{3,}\s*[A-Za-z0-9_-]*\s*$/, "")
    .trim();
}

function shouldDisplayMarkdownLine(value: string) {
  const cleaned = value.trim();
  if (!cleaned) {
    return false;
  }
  if (cleaned === "---" || cleaned === "***" || cleaned === "```") {
    return false;
  }
  if (/^`{3,}/.test(cleaned)) {
    return false;
  }
  return !isMarkdownTableSeparator(cleaned);
}

function isMarkdownTableSeparator(value: string) {
  const cleaned = value.trim();
  if (!cleaned.includes("-")) {
    return false;
  }
  return /^[|:\-\s]+$/.test(cleaned);
}

function loadStoredBuildGuideSelection(): GameBuildGuideOverlaySelection | null {
  try {
    const value = window.localStorage.getItem(ACTIVE_BUILD_GUIDE_STORAGE_KEY);
    if (!value) {
      return null;
    }
    const parsed = JSON.parse(value) as Partial<GameBuildGuideOverlaySelection>;
    if (typeof parsed.gameId !== "number" || typeof parsed.guideId !== "number") {
      return null;
    }
    return {
      gameId: parsed.gameId,
      guideId: parsed.guideId
    };
  } catch {
    return null;
  }
}

function loadStoredFontSize(): BuildGuideFontSize {
  const value = window.localStorage.getItem(BUILD_GUIDE_FONT_SIZE_STORAGE_KEY);
  return FONT_SIZES.includes(value as BuildGuideFontSize)
    ? (value as BuildGuideFontSize)
    : "small";
}

function storeBuildGuideSelection(selection: GameBuildGuideOverlaySelection) {
  window.localStorage.setItem(ACTIVE_BUILD_GUIDE_STORAGE_KEY, JSON.stringify(selection));
}

async function prepareBuildGuideOverlayWindow() {
  const window = getCurrentWindow();
  await window.setIgnoreCursorEvents(false).catch(() => {});
  return applyStandaloneOverlayFocusState(window);
}
