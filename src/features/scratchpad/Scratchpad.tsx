import { useEffect, useRef, useState } from "react";
import { loadScratchpad, saveScratchpad } from "../../services/scratchpad";

type SaveState = "idle" | "loading" | "saving" | "saved" | "error";

export function Scratchpad() {
  const [content, setContent] = useState("");
  const [saveState, setSaveState] = useState<SaveState>("loading");
  const saveTimer = useRef<number | undefined>(undefined);
  const didLoad = useRef(false);

  useEffect(() => {
    loadScratchpad()
      .then((value) => {
        setContent(value);
        setSaveState("idle");
        didLoad.current = true;
      })
      .catch(() => setSaveState("error"));
  }, []);

  useEffect(() => {
    if (!didLoad.current) {
      return;
    }

    setSaveState("saving");
    window.clearTimeout(saveTimer.current);
    saveTimer.current = window.setTimeout(() => {
      saveScratchpad(content)
        .then(() => setSaveState("saved"))
        .catch(() => setSaveState("error"));
    }, 350);

    return () => window.clearTimeout(saveTimer.current);
  }, [content]);

  return (
    <section className="scratchpad-panel">
      <div className="panel-heading">
        <div>
          <p>Placeholder Component</p>
          <h3>Scratchpad</h3>
        </div>
        <span className={`save-pill save-pill-${saveState}`}>{formatSaveState(saveState)}</span>
      </div>

      <textarea
        aria-label="Scratchpad"
        className="scratchpad-input"
        onChange={(event) => setContent(event.target.value)}
        placeholder="Capture the current thought, plan, or handoff note."
        spellCheck
        value={content}
      />
    </section>
  );
}

function formatSaveState(state: SaveState) {
  switch (state) {
    case "loading":
      return "Loading";
    case "saving":
      return "Saving";
    case "saved":
      return "Saved";
    case "error":
      return "Storage error";
    default:
      return "Ready";
  }
}

