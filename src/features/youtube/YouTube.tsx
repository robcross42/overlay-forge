import { useEffect, useMemo, useState } from "react";
import {
  createYouTubeReference,
  deleteYouTubeReference,
  listYouTubeReferences,
  openYouTubeReference,
  updateYouTubeReference
} from "../../services/youtube";
import type { YouTubeReference, YouTubeReferenceInput } from "../../services/youtube";
import { formatUnknownError as formatError } from "../../utils/errors";

const emptyReference: YouTubeReferenceInput = {
  title: "",
  url: "",
  channelName: "",
  notes: "",
  tags: ""
};

type YouTubeMode = "idle" | "create" | "view" | "edit";

export function YouTube() {
  const [references, setReferences] = useState<YouTubeReference[]>([]);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [mode, setMode] = useState<YouTubeMode>("idle");
  const [form, setForm] = useState<YouTubeReferenceInput>(emptyReference);
  const [status, setStatus] = useState("Loading YouTube references");

  const selectedReference = useMemo(
    () => references.find((reference) => reference.id === selectedId) ?? null,
    [references, selectedId]
  );

  useEffect(() => {
    listYouTubeReferences()
      .then((nextReferences) => {
        setReferences(nextReferences);
        setStatus(
          nextReferences.length === 0
            ? "No YouTube references yet"
            : `${nextReferences.length} reference(s)`
        );
      })
      .catch((error) => setStatus(formatError(error)));
  }, []);

  function selectReference(reference: YouTubeReference) {
    setSelectedId(reference.id);
    setMode("view");
    setForm({
      title: reference.title,
      url: reference.url,
      channelName: reference.channelName,
      notes: reference.notes,
      tags: reference.tags
    });
    setStatus("Reference selected");
  }

  function newReference() {
    setSelectedId(null);
    setMode("create");
    setForm(emptyReference);
    setStatus("New YouTube reference");
  }

  function updateField<K extends keyof YouTubeReferenceInput>(
    field: K,
    value: YouTubeReferenceInput[K]
  ) {
    setForm((current) => ({ ...current, [field]: value }));
  }

  async function onSaveReference() {
    const title = form.title.trim();
    const url = form.url.trim();

    if (!title) {
      setStatus("YouTube reference title is required");
      return;
    }

    if (!url) {
      setStatus("YouTube URL is required");
      return;
    }

    try {
      if (selectedReference) {
        const updated = await updateYouTubeReference(selectedReference.id, {
          ...form,
          title,
          url
        });
        setReferences((current) =>
          current.map((reference) => (reference.id === updated.id ? updated : reference))
        );
        selectReference(updated);
        setStatus("Reference saved");
      } else {
        const created = await createYouTubeReference({
          ...form,
          title,
          url
        });
        setReferences((current) => [created, ...current]);
        selectReference(created);
        setStatus("Reference added");
      }
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function onOpenReference() {
    if (!selectedReference) {
      return;
    }

    try {
      await openYouTubeReference(selectedReference.id);
      setStatus("Opened externally");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function onDeleteReference() {
    if (!selectedReference) {
      return;
    }

    try {
      await deleteYouTubeReference(selectedReference.id);
      setReferences((current) =>
        current.filter((reference) => reference.id !== selectedReference.id)
      );
      setSelectedId(null);
      setMode("idle");
      setForm(emptyReference);
      setStatus("Reference deleted");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  return (
    <section className="feature-panel">
      <div className="panel-heading">
        <div>
          <p>Controlled References</p>
          <h3>YouTube</h3>
        </div>
        <span className="save-pill">{status}</span>
      </div>

      <div className="split-feature-body">
        <aside className="sub-list" aria-label="YouTube reference list">
          <button className="primary-button full-width" onClick={newReference} type="button">
            New Reference
          </button>
          {references.map((reference) => (
            <button
              className={reference.id === selectedId ? "sub-list-item active" : "sub-list-item"}
              key={reference.id}
              onClick={() => selectReference(reference)}
              type="button"
            >
              <strong>{reference.title}</strong>
              <span>{reference.channelName || reference.videoId || "Saved YouTube URL"}</span>
            </button>
          ))}
        </aside>

        {selectedReference || mode === "create" ? (
          <form className="editor-form youtube-editor-form">
            <input
              aria-label="YouTube reference title"
              className="text-input"
              onChange={(event) => updateField("title", event.target.value)}
              placeholder="Title"
              readOnly={mode === "view"}
              value={form.title}
            />

            <input
              aria-label="YouTube URL"
              className="text-input"
              onChange={(event) => updateField("url", event.target.value)}
              placeholder="https://www.youtube.com/watch?v=VIDEO_ID"
              readOnly={mode === "view"}
              value={form.url}
            />

            <div className="field-grid">
              <label>
                <span>Channel</span>
                <input
                  aria-label="Channel name"
                  className="text-input"
                  onChange={(event) => updateField("channelName", event.target.value)}
                  placeholder="Channel name"
                  readOnly={mode === "view"}
                  value={form.channelName}
                />
              </label>

              <label>
                <span>Tags</span>
                <input
                  aria-label="Tags"
                  className="text-input"
                  onChange={(event) => updateField("tags", event.target.value)}
                  placeholder="planning, reference"
                  readOnly={mode === "view"}
                  value={form.tags}
                />
              </label>
            </div>

            <textarea
              aria-label="YouTube reference notes"
              className="body-input"
              onChange={(event) => updateField("notes", event.target.value)}
              placeholder="Notes"
              readOnly={mode === "view"}
              value={form.notes}
            />

            {selectedReference && (
              <dl className="metadata-grid compact-metadata">
                <div>
                  <dt>Video ID</dt>
                  <dd>{selectedReference.videoId || "Not parsed"}</dd>
                </div>
                <div>
                  <dt>Updated</dt>
                  <dd>{selectedReference.updatedAt}</dd>
                </div>
              </dl>
            )}

            <div className="form-actions">
              {mode === "view" ? (
                <button
                  className="primary-button"
                  onClick={() => {
                    setMode("edit");
                    setStatus("Editing reference");
                  }}
                  type="button"
                >
                  Edit
                </button>
              ) : (
                <button
                  className="primary-button"
                  onClick={() => void onSaveReference()}
                  type="button"
                >
                  {selectedReference ? "Save" : "Add"}
                </button>
              )}
              {selectedReference && (
                <button
                  className="primary-button"
                  onClick={() => void onOpenReference()}
                  type="button"
                >
                  Open
                </button>
              )}
              {selectedReference && (
                <button
                  className="primary-button"
                  onClick={() => void onDeleteReference()}
                  type="button"
                >
                  Delete
                </button>
              )}
            </div>
          </form>
        ) : (
          <div className="empty-editor-state">
            <p>Create or select a YouTube reference to begin.</p>
          </div>
        )}
      </div>
    </section>
  );
}
