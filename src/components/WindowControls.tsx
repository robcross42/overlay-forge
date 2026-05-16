import { useEffect, useState } from "react";
import {
  hideOverlayWindow,
  minimizeOverlayWindow,
  shutdownOverlayApp,
  startOverlayDrag,
  startOverlayResize,
  toggleOverlayMaximize
} from "../services/windowControls";

const resizeHandles = [
  { className: "resize-handle resize-n", direction: "North" },
  { className: "resize-handle resize-e", direction: "East" },
  { className: "resize-handle resize-s", direction: "South" },
  { className: "resize-handle resize-w", direction: "West" },
  { className: "resize-handle resize-ne", direction: "NorthEast" },
  { className: "resize-handle resize-nw", direction: "NorthWest" },
  { className: "resize-handle resize-se", direction: "SouthEast" },
  { className: "resize-handle resize-sw", direction: "SouthWest" }
] as const;

export function WindowTitlebar() {
  const [isMaximized, setIsMaximized] = useState(false);

  useEffect(() => {
    toggleOverlayMaximize(false)
      .then(setIsMaximized)
      .catch(() => setIsMaximized(false));
  }, []);

  async function onToggleMaximize() {
    const nextState = await toggleOverlayMaximize(true);
    setIsMaximized(nextState);
  }

  return (
    <header
      aria-label="Window titlebar"
      className="window-titlebar"
      onDoubleClick={onToggleMaximize}
      onMouseDown={(event) => {
        if (event.detail === 1 && event.button === 0) {
          void startOverlayDrag();
        }
      }}
    >
      <div className="window-title">
        <span className="window-title-mark">OF</span>
        <span>Overlay Forge</span>
      </div>

      <div
        className="window-controls"
        onDoubleClick={(event) => event.stopPropagation()}
        onMouseDown={(event) => event.stopPropagation()}
      >
        <button
          aria-label="Minimize"
          className="window-control"
          onClick={() => void minimizeOverlayWindow()}
          type="button"
        >
          <span aria-hidden="true">-</span>
        </button>
        <button
          aria-label={isMaximized ? "Restore" : "Maximize"}
          className="window-control"
          onClick={() => void onToggleMaximize()}
          type="button"
        >
          <span aria-hidden="true">{isMaximized ? "[ ]" : "[]"}</span>
        </button>
        <button
          aria-label="Hide overlay"
          className="window-control window-control-close"
          onClick={() => void hideOverlayWindow()}
          type="button"
        >
          <span aria-hidden="true">x</span>
        </button>
        <button
          aria-label="Shut down app"
          className="window-control window-control-shutdown"
          onClick={() => void shutdownOverlayApp()}
          type="button"
        >
          <span aria-hidden="true">!</span>
        </button>
      </div>
    </header>
  );
}

export function WindowResizeHandles() {
  return (
    <>
      {resizeHandles.map((handle) => (
        <button
          aria-label={`Resize ${handle.direction}`}
          className={handle.className}
          key={handle.direction}
          onMouseDown={(event) => {
            if (event.button === 0) {
              event.preventDefault();
              void startOverlayResize(handle.direction);
            }
          }}
          tabIndex={-1}
          type="button"
        />
      ))}
    </>
  );
}
