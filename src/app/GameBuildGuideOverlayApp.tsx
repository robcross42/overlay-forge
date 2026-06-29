import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useMemo, useState } from "react";
import type { MouseEvent, WheelEvent } from "react";
import { WindowResizeHandles } from "../components/WindowControls";
import {
  createBuildStepVisualModel,
  type BuildStepVisualElement,
  type BuildStepVisualModel
} from "../features/gaming/buildGuideVisuals";
import {
  getActiveGameBuildGuideOverlay,
  getGameBuildGuide,
  listGames
} from "../services/gaming";
import type {
  Game,
  GameBuildGuideOverlaySelection,
  GameBuildGuidePayload
} from "../services/gaming";
import { applyStandaloneOverlayFocusState, startOverlayDrag } from "../services/windowControls";
import { formatUnknownError as formatError } from "../utils/errors";

const ACTIVE_BUILD_GUIDE_STORAGE_KEY = "overlayForgeActiveBuildGuide";
const BUILD_GUIDE_FONT_SIZE_STORAGE_KEY = "overlayForgeBuildGuideFontSize";
const BUILD_GUIDE_DIAGRAM_ZOOM_STORAGE_KEY = "overlayForgeBuildGuideDiagramZoom";
const FONT_SIZES = ["small", "medium", "large"] as const;
const MIN_DIAGRAM_ZOOM = 1;
const MAX_DIAGRAM_ZOOM = 3;
const DIAGRAM_ZOOM_STEP = 0.25;
type BuildGuideFontSize = (typeof FONT_SIZES)[number];
type BuildGuideViewMode = "info" | "steps";

export default function GameBuildGuideOverlayApp() {
  const [game, setGame] = useState<Game | null>(null);
  const [payload, setPayload] = useState<GameBuildGuidePayload | null>(null);
  const [status, setStatus] = useState("Loading build guide");
  const [fontSize, setFontSize] = useState<BuildGuideFontSize>(loadStoredFontSize);
  const [viewMode, setViewMode] = useState<BuildGuideViewMode>("info");
  const [diagramZoom, setDiagramZoom] = useState(loadStoredDiagramZoom);

  const visualModels = useMemo(() => {
    if (!payload) {
      return [];
    }
    return payload.steps.map((step, index) =>
      createBuildStepVisualModel(step, payload.parts, index)
    );
  }, [payload]);

  useEffect(() => {
    setViewMode("info");
  }, [payload?.guide.id]);

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

  function changeDiagramZoom(direction: -1 | 1) {
    setDiagramZoom((current) => {
      const next = clampDiagramZoom(current + direction * DIAGRAM_ZOOM_STEP);
      window.localStorage.setItem(BUILD_GUIDE_DIAGRAM_ZOOM_STORAGE_KEY, String(next));
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
              <div className="build-guide-view-toolbar">
                {viewMode === "steps" && (
                  <div className="build-guide-zoom-controls" aria-label="Diagram zoom controls">
                    <button
                      aria-label="Zoom diagram out"
                      className="ghost-button build-guide-zoom-button"
                      disabled={diagramZoom <= MIN_DIAGRAM_ZOOM}
                      onClick={() => changeDiagramZoom(-1)}
                      type="button"
                    >
                      -
                    </button>
                    <span>{Math.round(diagramZoom * 100)}%</span>
                    <button
                      aria-label="Zoom diagram in"
                      className="ghost-button build-guide-zoom-button"
                      disabled={diagramZoom >= MAX_DIAGRAM_ZOOM}
                      onClick={() => changeDiagramZoom(1)}
                      type="button"
                    >
                      +
                    </button>
                  </div>
                )}
                <button
                  type="button"
                  className="ghost-button build-guide-view-toggle"
                  onClick={() => setViewMode((current) => (current === "info" ? "steps" : "info"))}
                >
                  {viewMode === "info" ? "Steps" : "Info"}
                </button>
              </div>

              {viewMode === "info" ? (
                <BuildGuideInfoView payload={payload} status={status} />
              ) : (
                <BuildGuideStepsView
                  diagramZoom={diagramZoom}
                  payload={payload}
                  visualModels={visualModels}
                />
              )}
            </>
          )}
        </section>
      </div>
    </main>
  );
}

function BuildGuideInfoView({
  payload,
  status
}: {
  payload: GameBuildGuidePayload;
  status: string;
}) {
  return (
    <div className="build-guide-info-view">
      <div className="build-guide-meta-row" aria-label="Build guide summary">
        <span>{payload.parts.length} parts</span>
        <span>{payload.steps.length} steps</span>
        <span>{status}</span>
      </div>

      <section className="build-guide-context-panel" aria-label="Guide context">
        <GuideTextSection title="Goal" value={payload.guide.buildGoal} />
        <GuideTextSection title="Scale" value={payload.guide.scaleReference} />
        <GuideTextSection title="Geometry" value={payload.guide.geometryNotes} />
        <GuideTextSection title="Glossary" value={payload.guide.glossaryText} />
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
    </div>
  );
}

function BuildGuideStepsView({
  diagramZoom,
  payload,
  visualModels
}: {
  diagramZoom: number;
  payload: GameBuildGuidePayload;
  visualModels: BuildStepVisualModel[];
}) {
  if (payload.steps.length === 0) {
    return (
      <section className="build-guide-section" aria-label="Assembly instructions">
        <h2>Steps</h2>
        <p>No structured assembly steps found in this guide.</p>
      </section>
    );
  }

  return (
    <section className="build-guide-steps-gallery" aria-label="Build step diagrams">
      {payload.steps.map((step, index) => {
        const model = visualModels[index];
        if (!model) {
          return null;
        }
        return (
          <article className="build-guide-step-card" key={step.id}>
            <section className="build-guide-visual-panel" aria-label={`Step ${step.stepNumber} diagram`}>
              <BuildStepDiagram
                diagramZoom={diagramZoom}
                model={model}
                stepNumber={step.stepNumber}
                totalSteps={payload.steps.length}
              />
            </section>
            <section className="build-guide-step-caption" aria-label={`Step ${step.stepNumber} caption`}>
              <div className="build-guide-step-head">
                <span>{step.stepNumber}</span>
                <h2>{step.title || `Step ${step.stepNumber}`}</h2>
              </div>
              {model.captionLines.map((line) => (
                <p key={line}>{line}</p>
              ))}
            </section>
          </article>
        );
      })}
    </section>
  );
}

function BuildStepDiagram({
  diagramZoom,
  model,
  stepNumber,
  totalSteps
}: {
  diagramZoom: number;
  model: BuildStepVisualModel;
  stepNumber: number;
  totalSteps: number;
}) {
  const elementById = new Map(model.elements.map((element) => [element.id, element]));
  const sortedElements = [...model.elements].sort(
    (left, right) => left.x + left.z + left.y - (right.x + right.z + right.y)
  );
  const projector = createIsoProjector(model.grid);
  const callouts = createDiagramCallouts(sortedElements, projector);
  const groundPlanePoints = points([
    [model.grid.xMin, 0, model.grid.zMin],
    [model.grid.xMax, 0, model.grid.zMin],
    [model.grid.xMax, 0, model.grid.zMax],
    [model.grid.xMin, 0, model.grid.zMax]
  ], projector);

  return (
    <div className="build-guide-diagram-frame">
      <div className="build-guide-diagram-title">
        <strong>{model.subtitle}</strong>
        <span>
          Step {stepNumber} / {totalSteps}
        </span>
      </div>
      <svg
        className="build-guide-isometric-svg"
        style={{
          width: `${diagramZoom * 100}%`,
          height: `${diagramZoom * 100}%`
        }}
        viewBox="0 0 640 360"
        role="img"
        aria-label={`${model.title} placement diagram`}
      >
        <defs>
          {(["new", "existing", "reference"] as const).map((role) => (
            <marker
              key={`build-guide-arrow-${role}`}
              id={`build-guide-arrow-${role}`}
              viewBox="0 0 10 10"
              refX="8.2"
              refY="5"
              markerWidth="8"
              markerHeight="8"
              orient="auto-start-reverse"
            >
              <path
                className={`build-guide-arrow-head build-guide-link-arrow-head build-guide-arrow-head-${role}`}
                d="M 1 1 L 9 5 L 1 9 z"
              />
            </marker>
          ))}
          {(["new", "existing", "reference"] as const).map((role) => (
            <marker
              key={`build-guide-callout-arrow-${role}`}
              id={`build-guide-callout-arrow-${role}`}
              viewBox="0 0 10 10"
              refX="8.2"
              refY="5"
              markerWidth="7"
              markerHeight="7"
              orient="auto"
            >
              <path
                className={`build-guide-arrow-head build-guide-callout-arrow-head build-guide-arrow-head-${role}`}
                d="M 1 1 L 9 5 L 1 9 z"
              />
            </marker>
          ))}
          <linearGradient id="build-guide-ground-gradient" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="rgba(105, 142, 177, 0.26)" />
            <stop offset="100%" stopColor="rgba(34, 48, 64, 0.12)" />
          </linearGradient>
        </defs>
        <polygon
          className="build-guide-ground"
          points={groundPlanePoints}
          fill="url(#build-guide-ground-gradient)"
        />
        <g className="build-guide-grid-lines" aria-hidden="true">
          {integerRange(model.grid.zMin, model.grid.zMax).map((offset) => {
            const a = projector.project(model.grid.xMin, 0, offset);
            const b = projector.project(model.grid.xMax, 0, offset);
            return (
              <line key={`z-${offset}`} x1={a.x} y1={a.y} x2={b.x} y2={b.y} />
            );
          })}
          {integerRange(model.grid.xMin, model.grid.xMax).map((offset) => {
            const a = projector.project(offset, 0, model.grid.zMin);
            const b = projector.project(offset, 0, model.grid.zMax);
            return (
              <line key={`x-${offset}`} x1={a.x} y1={a.y} x2={b.x} y2={b.y} />
            );
          })}
        </g>
        <g>
          {sortedElements.map((element) => (
            <BuildStepElement key={element.id} element={element} projector={projector} />
          ))}
        </g>
        <g className="build-guide-link-lines">
          {model.links.map((link) => {
            const from = elementById.get(link.from);
            const to = elementById.get(link.to);
            if (!from || !to) {
              return null;
            }
            const start = projector.project(from.x, from.y + from.height + 0.15, from.z);
            const end = projector.project(to.x, to.y + to.height + 0.15, to.z);
            return (
              <g key={link.id} className={`build-guide-link-role-${to.role}`}>
                <line
                  x1={start.x}
                  y1={start.y}
                  x2={end.x}
                  y2={end.y}
                  markerEnd={arrowMarkerUrl("build-guide-arrow", to.role)}
                />
              </g>
            );
          })}
        </g>
        <g className="build-guide-callout-lines">
          {callouts.map((callout) => (
            <g key={callout.id} className={`build-guide-callout-role-${callout.role}`}>
              <line
                x1={callout.lineStartX}
                y1={callout.lineStartY}
                x2={callout.targetX}
                y2={callout.targetY}
                markerEnd={arrowMarkerUrl("build-guide-callout-arrow", callout.role)}
              />
              <circle cx={callout.targetX} cy={callout.targetY} r="3.5" />
            </g>
          ))}
        </g>
        <g className="build-guide-diagram-callouts">
          {callouts.map((callout) => (
            <text
              key={callout.id}
              className={`build-guide-callout-role-${callout.role}`}
              textAnchor={callout.textAnchor}
              x={callout.labelX}
              y={callout.labelY}
            >
              {callout.label}
            </text>
          ))}
        </g>
        <g className="build-guide-diagram-legend">
          <circle cx="38" cy="292" r="9" className="legend-new" />
          <text x="58" y="301">
            new placement
          </text>
          <circle cx="38" cy="326" r="9" className="legend-existing" />
          <text x="58" y="335">
            existing/reference
          </text>
        </g>
      </svg>
    </div>
  );
}

function BuildStepElement({
  element,
  projector
}: {
  element: BuildStepVisualElement;
  projector: IsoProjector;
}) {
  if (element.shape === "engine-driven-crank") {
    return (
      <g className={`build-guide-element build-guide-role-${element.role}`}>
        <BuildStepEngineDrivenCrank element={element} projector={projector} />
      </g>
    );
  }
  if (element.shape === "wheel" || element.shape === "gear") {
    return (
      <g className={`build-guide-element build-guide-role-${element.role}`}>
        <BuildStepDisc element={element} projector={projector} />
      </g>
    );
  }
  if (element.shape === "spring") {
    return (
      <g className={`build-guide-element build-guide-role-${element.role}`}>
        <BuildStepSpring element={element} projector={projector} />
      </g>
    );
  }
  if (element.shape === "cylinder") {
    return (
      <g className={`build-guide-element build-guide-role-${element.role}`}>
        <BuildStepCylinder element={element} projector={projector} />
      </g>
    );
  }
  return (
    <g className={`build-guide-element build-guide-role-${element.role}`}>
      <BuildStepCuboid element={element} projector={projector} />
    </g>
  );
}

function BuildStepEngineDrivenCrank({
  element,
  projector
}: {
  element: BuildStepVisualElement;
  projector: IsoProjector;
}) {
  const x0 = element.x - element.width / 2;
  const crankThickness = Math.min(1, element.width);
  const axleLength = Math.max(0.5, element.width - crankThickness);
  const crankCenterX = x0 + crankThickness / 2;
  const axleCenterX = x0 + crankThickness + axleLength / 2;
  const shaftSize = 0.5;
  const axle: BuildStepVisualElement = {
    ...element,
    id: `${element.id}-axle`,
    shape: "beam",
    axis: "x",
    x: axleCenterX,
    y: element.y + (element.height - shaftSize) / 2,
    z: element.z,
    width: axleLength,
    depth: shaftSize,
    height: shaftSize
  };
  const crank: BuildStepVisualElement = {
    ...element,
    id: `${element.id}-crank`,
    shape: "cylinder",
    axis: "x",
    x: crankCenterX,
    width: crankThickness,
    depth: element.depth,
    height: element.height
  };
  const hub: BuildStepVisualElement = {
    ...element,
    id: `${element.id}-hub`,
    shape: "cylinder",
    axis: "x",
    x: crankCenterX,
    y: element.y + element.height * 0.31,
    width: crankThickness + 0.02,
    depth: element.depth * 0.45,
    height: element.height * 0.45
  };

  return (
    <>
      <BuildStepCuboid element={axle} projector={projector} />
      <BuildStepCylinder element={crank} projector={projector} />
      <BuildStepCylinder element={hub} projector={projector} />
    </>
  );
}

function BuildStepCylinder({
  element,
  projector
}: {
  element: BuildStepVisualElement;
  projector: IsoProjector;
}) {
  const center = projector.project(element.x, element.y + element.height / 2, element.z);
  const rx = Math.max(6, element.depth * projector.scale * 0.5);
  const ry = Math.max(5, element.height * projector.scale * 0.38);
  const thickness = Math.max(3, element.width * projector.scale * 0.18);
  return (
    <>
      <ellipse className="iso-disc-shadow" cx={center.x + thickness} cy={center.y + thickness} rx={rx} ry={ry} />
      <ellipse className="iso-cylinder-back" cx={center.x + thickness} cy={center.y + thickness * 0.48} rx={rx} ry={ry} />
      <path
        className="iso-cylinder-body"
        d={`M ${center.x - rx} ${center.y} L ${center.x - rx + thickness} ${center.y + thickness * 0.48} A ${rx} ${ry} 0 0 0 ${center.x + rx + thickness} ${center.y + thickness * 0.48} L ${center.x + rx} ${center.y} A ${rx} ${ry} 0 0 1 ${center.x - rx} ${center.y} Z`}
      />
      <ellipse className="iso-disc" cx={center.x} cy={center.y} rx={rx} ry={ry} />
      <ellipse className="iso-disc-inner" cx={center.x} cy={center.y} rx={rx * 0.28} ry={ry * 0.28} />
    </>
  );
}

function BuildStepCuboid({
  element,
  projector
}: {
  element: BuildStepVisualElement;
  projector: IsoProjector;
}) {
  const x0 = element.x - element.width / 2;
  const x1 = element.x + element.width / 2;
  const y0 = element.y;
  const y1 = element.y + element.height;
  const z0 = element.z - element.depth / 2;
  const z1 = element.z + element.depth / 2;
  const top = points([
    [x0, y1, z0],
    [x1, y1, z0],
    [x1, y1, z1],
    [x0, y1, z1]
  ], projector);
  const left = points([
    [x0, y0, z1],
    [x0, y1, z1],
    [x0, y1, z0],
    [x0, y0, z0]
  ], projector);
  const right = points([
    [x0, y0, z1],
    [x1, y0, z1],
    [x1, y1, z1],
    [x0, y1, z1]
  ], projector);
  return (
    <>
      <polygon className="iso-face iso-face-left" points={left} />
      <polygon className="iso-face iso-face-right" points={right} />
      <polygon className="iso-face iso-face-top" points={top} />
    </>
  );
}

function BuildStepDisc({
  element,
  projector
}: {
  element: BuildStepVisualElement;
  projector: IsoProjector;
}) {
  const center = projector.project(element.x, element.y + element.height / 2, element.z);
  const rx = Math.max(8, element.depth * projector.scale * 0.67);
  const ry = Math.max(6, element.height * projector.scale * 0.47);
  return (
    <>
      <ellipse className="iso-disc-shadow" cx={center.x + 5} cy={center.y + 7} rx={rx} ry={ry} />
      <ellipse className="iso-disc" cx={center.x} cy={center.y} rx={rx} ry={ry} />
      <ellipse className="iso-disc-inner" cx={center.x} cy={center.y} rx={rx * 0.45} ry={ry * 0.45} />
      {element.shape === "gear" && (
        <>
          <line
            className="iso-disc-spoke"
            x1={center.x - rx * 0.75}
            y1={center.y}
            x2={center.x + rx * 0.75}
            y2={center.y}
          />
          <line
            className="iso-disc-spoke"
            x1={center.x}
            y1={center.y - ry * 0.75}
            x2={center.x}
            y2={center.y + ry * 0.75}
          />
        </>
      )}
    </>
  );
}

function BuildStepSpring({
  element,
  projector
}: {
  element: BuildStepVisualElement;
  projector: IsoProjector;
}) {
  const springPoints = Array.from({ length: 10 }, (_, index) => {
    const t = index / 9;
    const x = element.x + (index % 2 === 0 ? -element.width / 2 : element.width / 2);
    return projector.project(x, element.y + element.height * t, element.z);
  });
  const base = projector.project(element.x, element.y, element.z);
  const top = projector.project(element.x, element.y + element.height, element.z);
  return (
    <>
      <line className="iso-spring-guide" x1={base.x} y1={base.y} x2={top.x} y2={top.y} />
      <polyline
        className="iso-spring"
        points={springPoints.map((point) => `${point.x},${point.y}`).join(" ")}
      />
    </>
  );
}

type IsoProjector = {
  scale: number;
  project: (x: number, y: number, z: number) => { x: number; y: number };
};

function createIsoProjector(grid: BuildStepVisualModel["grid"]): IsoProjector {
  const width = Math.max(1, grid.xMax - grid.xMin);
  const depth = Math.max(1, grid.zMax - grid.zMin);
  const scale = Math.min(30, 500 / ((width + depth) * 0.9), 205 / ((width + depth) * 0.42));
  const centerX = (grid.xMin + grid.xMax) / 2;
  const centerZ = (grid.zMin + grid.zMax) / 2;
  return {
    scale,
    project: (x, y, z) => ({
      x: 320 + (x - centerX - (z - centerZ)) * scale * 0.9,
      y: 224 + (x - centerX + z - centerZ) * scale * 0.42 - y * scale
    })
  };
}

function points(values: Array<[number, number, number]>, projector: IsoProjector) {
  return values
    .map(([x, y, z]) => {
      const point = projector.project(x, y, z);
      return `${point.x},${point.y}`;
    })
    .join(" ");
}

function arrowMarkerUrl(prefix: string, role: BuildStepVisualElement["role"]) {
  return `url(#${prefix}-${role})`;
}

function integerRange(min: number, max: number) {
  return Array.from({ length: Math.max(0, Math.floor(max) - Math.ceil(min) + 1) }, (_, index) =>
    Math.ceil(min) + index
  );
}

type DiagramCallout = {
  id: string;
  label: string;
  role: BuildStepVisualElement["role"];
  labelX: number;
  labelY: number;
  lineStartX: number;
  lineStartY: number;
  targetX: number;
  targetY: number;
  textAnchor: "start" | "end";
};

function createDiagramCallouts(
  elements: BuildStepVisualElement[],
  projector: IsoProjector
): DiagramCallout[] {
  const slots: Array<{ x: number; y: number; textAnchor: "start" | "end" }> = [
    { x: 34, y: 44, textAnchor: "start" },
    { x: 34, y: 72, textAnchor: "start" },
    { x: 606, y: 44, textAnchor: "end" },
    { x: 606, y: 72, textAnchor: "end" },
    { x: 606, y: 252, textAnchor: "end" },
    { x: 606, y: 280, textAnchor: "end" },
    { x: 606, y: 308, textAnchor: "end" },
    { x: 34, y: 100, textAnchor: "start" }
  ];

  return elements.slice(0, slots.length).map((element, index) => {
    const slot = slots[index];
    const target = projector.project(element.x, element.y + element.height + 0.18, element.z);
    const label = truncateVisualLabel(element.label);
    const labelWidth = Math.min(178, Math.max(56, label.length * 11));
    const lineStartX =
      slot.textAnchor === "start" ? slot.x + labelWidth + 8 : slot.x - labelWidth - 8;
    return {
      id: element.id,
      label,
      role: element.role,
      labelX: slot.x,
      labelY: slot.y,
      lineStartX,
      lineStartY: slot.y - 6,
      targetX: target.x,
      targetY: target.y,
      textAnchor: slot.textAnchor
    };
  });
}

function truncateVisualLabel(value: string) {
  return cleanMarkdownLine(value);
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

function loadStoredDiagramZoom() {
  const value = Number(window.localStorage.getItem(BUILD_GUIDE_DIAGRAM_ZOOM_STORAGE_KEY));
  return clampDiagramZoom(Number.isFinite(value) ? value : MIN_DIAGRAM_ZOOM);
}

function clampDiagramZoom(value: number) {
  return Math.min(MAX_DIAGRAM_ZOOM, Math.max(MIN_DIAGRAM_ZOOM, value));
}

function storeBuildGuideSelection(selection: GameBuildGuideOverlaySelection) {
  window.localStorage.setItem(ACTIVE_BUILD_GUIDE_STORAGE_KEY, JSON.stringify(selection));
}

async function prepareBuildGuideOverlayWindow() {
  const window = getCurrentWindow();
  await window.setIgnoreCursorEvents(false).catch(() => {});
  return applyStandaloneOverlayFocusState(window);
}
