import { useEffect, useMemo, useState } from "react";
import {
  clearOpenAiApiKey,
  getOpenAiApiKeyStatus,
  listKeybinds,
  resetKeybinds,
  saveKeybinds,
  saveOpenAiApiKey
} from "../../services/settings";
import type { ApiKeyStatus, KeybindRecord } from "../../services/settings";

type CaptureTarget = {
  action: string;
  slot: number;
};

const MODIFIER_LABELS = ["Ctrl", "Shift", "Alt"] as const;
const RESERVED_KEYS = new Set(["Escape", "Tab", "CapsLock", "PrintScreen", "Meta", "OS"]);
const COMMON_CTRL_KEYS = new Set(["A", "C", "F", "N", "P", "R", "S", "T", "V", "W", "X", "Y", "Z"]);
const MOUSE_KEYS = new Set(["MouseLeft", "MouseRight", "MouseMiddle", "Mouse4", "Mouse5"]);

export function Settings() {
  const [status, setStatus] = useState<ApiKeyStatus | null>(null);
  const [apiKey, setApiKey] = useState("");
  const [message, setMessage] = useState("Loading settings");
  const [isSaving, setIsSaving] = useState(false);
  const [keybinds, setKeybinds] = useState<KeybindRecord[]>([]);
  const [captureTarget, setCaptureTarget] = useState<CaptureTarget | null>(null);

  const duplicateMap = useMemo(() => {
    const counts = new Map<string, number>();
    for (const keybind of keybinds) {
      const combo = shortcutFromParts(keybind.keys);
      if (combo) {
        counts.set(combo.toLowerCase(), (counts.get(combo.toLowerCase()) ?? 0) + 1);
      }
    }
    return counts;
  }, [keybinds]);

  useEffect(() => {
    getOpenAiApiKeyStatus()
      .then((nextStatus) => {
        setStatus(nextStatus);
        setMessage(
          nextStatus.isConfigured ? `OpenAI key: ${nextStatus.source}` : "OpenAI key not configured"
        );
      })
      .catch((error) => setMessage(formatError(error)));

    listKeybinds()
      .then(setKeybinds)
      .catch((error) => setMessage(formatError(error)));
  }, []);

  useEffect(() => {
    if (!captureTarget) {
      return;
    }
    const target = captureTarget;

    function assignKey(nextKey: string, hasMeta = false) {
      if (hasMeta || nextKey === "Meta" || nextKey === "OS") {
        setMessage("Windows/Super key shortcuts are reserved for the operating system");
        return;
      }
      if (!nextKey) {
        setMessage("Press a key to assign");
        return;
      }

      const nextKeybinds = replaceKeyPart(keybinds, target.action, target.slot, nextKey);
      const edited = nextKeybinds.find((keybind) => keybind.action === target.action);
      const shortcut = edited ? shortcutFromParts(edited.keys) : "";
      const validationError = shortcut ? validateKeybind(shortcut) : "";
      if (validationError) {
        setMessage(validationError);
        return;
      }
      const duplicate = keybinds.some(
        (keybind) => keybind.action !== target.action && shortcutFromParts(keybind.keys).toLowerCase() === shortcut.toLowerCase()
      );

      if (duplicate) {
        setMessage(`Duplicate keybind is not allowed: ${shortcut}`);
        return;
      }

      void saveUpdatedKeybinds(nextKeybinds, shortcut ? `Saved keybind: ${shortcut}` : "Keybind updated");
      setCaptureTarget(null);
    }

    function handleKeyDown(event: KeyboardEvent) {
      event.preventDefault();
      event.stopPropagation();

      if (event.key === "Escape") {
        setCaptureTarget(null);
        setMessage("Keybind capture cancelled");
        return;
      }

      assignKey(normalizeKey(event), event.metaKey);
    }

    function handleMouseDown(event: MouseEvent) {
      event.preventDefault();
      event.stopPropagation();

      const nextKey = normalizeMouseButton(event);
      if (!nextKey) {
        setMessage("That mouse button cannot be assigned");
        return;
      }

      assignKey(nextKey);
    }

    window.addEventListener("keydown", handleKeyDown, true);
    window.addEventListener("mousedown", handleMouseDown, true);
    return () => {
      window.removeEventListener("keydown", handleKeyDown, true);
      window.removeEventListener("mousedown", handleMouseDown, true);
    };
  }, [captureTarget, keybinds]);

  async function saveKey() {
    const trimmedKey = apiKey.trim();
    if (!trimmedKey) {
      setMessage("OpenAI API key is required");
      return;
    }

    setIsSaving(true);
    try {
      const nextStatus = await saveOpenAiApiKey(trimmedKey);
      setStatus(nextStatus);
      setApiKey("");
      setMessage("OpenAI API key saved in plaintext local settings");
    } catch (error) {
      setMessage(formatError(error));
    } finally {
      setIsSaving(false);
    }
  }

  async function clearKey() {
    setIsSaving(true);
    try {
      const nextStatus = await clearOpenAiApiKey();
      setStatus(nextStatus);
      setApiKey("");
      setMessage(nextStatus.isConfigured ? `Using ${nextStatus.source}` : "OpenAI API key cleared");
    } catch (error) {
      setMessage(formatError(error));
    } finally {
      setIsSaving(false);
    }
  }

  async function saveUpdatedKeybinds(nextKeybinds: KeybindRecord[], successMessage: string) {
    try {
      const saved = await saveKeybinds(nextKeybinds);
      setKeybinds(saved);
      setMessage(successMessage);
    } catch (error) {
      setMessage(formatError(error));
    }
  }

  async function clearKeybind(action: string, slot: number) {
    const nextKeybinds = replaceKeyPart(keybinds, action, slot, "");
    await saveUpdatedKeybinds(nextKeybinds, "Key unmapped");
  }

  async function resetAllKeybinds() {
    try {
      const defaults = await resetKeybinds();
      setKeybinds(defaults);
      setMessage("Keybinds reset to defaults");
    } catch (error) {
      setMessage(formatError(error));
    }
  }

  return (
    <section className="feature-panel">
      <div className="panel-heading">
        <div>
          <p>Local Configuration</p>
          <h3>Settings</h3>
        </div>
        <span className="save-pill">{message}</span>
      </div>

      <div className="settings-panel">
        <section className="settings-card" aria-label="OpenAI API key">
          <div>
            <p>OpenAI</p>
            <h4>{status?.isConfigured ? "Configured" : "Not configured"}</h4>
          </div>
          <span>{status?.source ?? "Checking configuration"}</span>
          <input
            aria-label="OpenAI API key"
            className="text-input"
            onChange={(event) => setApiKey(event.target.value)}
            placeholder="Paste OpenAI API key"
            type="password"
            value={apiKey}
          />
          <div className="settings-actions">
            <button
              className="primary-button"
              disabled={isSaving || apiKey.trim().length === 0}
              onClick={() => void saveKey()}
              type="button"
            >
              Save Key
            </button>
            <button
              className="ghost-button"
              disabled={isSaving || !status?.isConfigured}
              onClick={() => void clearKey()}
              type="button"
            >
              Clear Saved Key
            </button>
          </div>
          <p>
            The key is stored locally in plaintext SQLite settings. The Tauri backend uses it for
            OpenAI requests and falls back to `OPENAI_API_KEY` if no saved key exists.
          </p>
        </section>

        <section className="settings-card settings-keybinds-card" aria-label="Keybinds">
          <div className="settings-card-heading-row">
            <div>
              <p>Controls</p>
              <h4>Keybinds</h4>
            </div>
            <button className="ghost-button" onClick={() => void resetAllKeybinds()} type="button">
              Reset Defaults
            </button>
          </div>

          <div className="keybind-table" role="table" aria-label="Configurable keybinds">
            <div className="keybind-row keybind-row-head" role="row">
              <span role="columnheader">Function</span>
              <span role="columnheader">key1</span>
              <span role="columnheader">key2</span>
              <span role="columnheader">key3</span>
            </div>
            {keybinds.map((keybind) => (
              <div className="keybind-row" role="row" key={keybind.action}>
                <strong role="cell">{keybind.label}</strong>
                {[0, 1, 2].map((slot) => {
                  const key = keybind.keys[slot] ?? "";
                  const shortcut = shortcutFromParts(keybind.keys);
                  const duplicate = shortcut && (duplicateMap.get(shortcut.toLowerCase()) ?? 0) > 1;
                  return (
                    <button
                      className={duplicate ? "keybind-cell duplicate" : "keybind-cell"}
                      key={slot}
                      onClick={() => setCaptureTarget({ action: keybind.action, slot })}
                      onContextMenu={(event) => {
                        event.preventDefault();
                        if (key) {
                          void clearKeybind(keybind.action, slot);
                        }
                      }}
                      role="cell"
                      title={key ? "Left click to remap this key. Right click to unmap." : "Left click to map this key."}
                      type="button"
                    >
                      {key || "Unmapped"}
                    </button>
                  );
                })}
              </div>
            ))}
          </div>
        </section>
      </div>

      {captureTarget && (
        <div
          className="keybind-capture-layer"
          onContextMenu={(event) => {
            event.preventDefault();
            if (event.target === event.currentTarget) {
              setCaptureTarget(null);
              setMessage("Keybind capture cancelled");
            }
          }}
        >
          <section className="keybind-capture-dialog" aria-label="Press keybind">
            <p>Press a key or mouse button to assign.</p>
            <strong>Esc cancels. Right click assigns MouseRight.</strong>
          </section>
        </div>
      )}
    </section>
  );
}

function normalizeKey(event: KeyboardEvent) {
  if (event.key === "Control") {
    return "Ctrl";
  }
  if (event.key === "Shift") {
    return "Shift";
  }
  if (event.key === "Alt") {
    return "Alt";
  }
  if (event.key === "Meta" || event.key === "OS") {
    return event.key;
  }
  if (/^Key[A-Z]$/.test(event.code)) {
    return event.code.replace("Key", "");
  }
  if (/^Digit[0-9]$/.test(event.code)) {
    return event.code.replace("Digit", "");
  }
  if (event.code === "Space") {
    return "Space";
  }
  if (event.code.startsWith("Arrow")) {
    return event.code;
  }
  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(event.code)) {
    return event.code;
  }
  if (event.code.startsWith("Numpad")) {
    return event.code;
  }
  return event.key.length === 1 ? event.key.toUpperCase() : event.key;
}

function normalizeMouseButton(event: MouseEvent) {
  switch (event.button) {
    case 0:
      return "MouseLeft";
    case 1:
      return "MouseMiddle";
    case 2:
      return "MouseRight";
    case 3:
      return "Mouse4";
    case 4:
      return "Mouse5";
    default:
      return "";
  }
}

function replaceKeyPart(
  keybinds: KeybindRecord[],
  action: string,
  slot: number,
  key: string
) {
  return keybinds.map((keybind) => {
    if (keybind.action !== action) {
      return keybind;
    }

    const nextKeys = [...keybind.keys];
    while (nextKeys.length < 3) {
      nextKeys.push("");
    }
    nextKeys[slot] = key;
    return {
      ...keybind,
      keys: nextKeys.slice(0, 3)
    };
  });
}

function shortcutFromParts(keys: string[]) {
  return keys.filter(Boolean).join("+");
}

function validateKeybind(value: string) {
  const parts = value.split("+").filter(Boolean);
  const mainKey = parts[parts.length - 1] ?? "";
  const modifiers = parts.slice(0, -1);

  if (!mainKey || MODIFIER_LABELS.includes(mainKey as (typeof MODIFIER_LABELS)[number])) {
    return "Press a non-modifier key to complete the keybind";
  }
  if (parts.length > 3) {
    return "Use at most two modifier keys plus one normal key";
  }
  if (RESERVED_KEYS.has(mainKey)) {
    return `${mainKey} is reserved and cannot be mapped`;
  }
  if (modifiers.includes("Meta") || mainKey === "Meta" || mainKey === "OS") {
    return "Windows/Super key shortcuts are reserved for the operating system";
  }
  if (value === "Alt+F4") {
    return "Alt+F4 is reserved for closing windows";
  }
  if (modifiers.length === 0 && isPrintableKey(mainKey)) {
    return "Single printable keys are not allowed as global shortcuts";
  }
  if (modifiers.length === 0 && (mainKey === "MouseLeft" || mainKey === "MouseRight")) {
    return `${mainKey} requires Ctrl, Shift, or Alt so normal clicks are not intercepted`;
  }
  if (MOUSE_KEYS.has(mainKey)) {
    return "";
  }
  if (modifiers.length === 1 && modifiers[0] === "Ctrl" && COMMON_CTRL_KEYS.has(mainKey)) {
    return `Ctrl+${mainKey} is reserved for common app editing commands`;
  }
  return "";
}

function isPrintableKey(key: string) {
  return /^[A-Z0-9]$/.test(key) || key === "Space" || /^[,./;'[\]\\`=-]$/.test(key);
}

function formatError(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
