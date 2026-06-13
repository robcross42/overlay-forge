import { useEffect, useState } from "react";
import {
  clearOpenAiApiKey,
  getOpenAiApiKeyStatus,
  saveOpenAiApiKey
} from "../../services/settings";
import type { ApiKeyStatus } from "../../services/settings";

export function Settings() {
  const [status, setStatus] = useState<ApiKeyStatus | null>(null);
  const [apiKey, setApiKey] = useState("");
  const [message, setMessage] = useState("Loading settings");
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    getOpenAiApiKeyStatus()
      .then((nextStatus) => {
        setStatus(nextStatus);
        setMessage(nextStatus.isConfigured ? `OpenAI key: ${nextStatus.source}` : "OpenAI key not configured");
      })
      .catch((error) => setMessage(formatError(error)));
  }, []);

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
      </div>
    </section>
  );
}

function formatError(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
