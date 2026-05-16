import { useEffect, useMemo, useState } from "react";
import { createNote, deleteNote, listNotes, updateNote } from "../../services/notes";
import type { Note } from "../../services/notes";

type NoteMode = "idle" | "create" | "view" | "edit";

export function Notes() {
  const [notes, setNotes] = useState<Note[]>([]);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [noteMode, setNoteMode] = useState<NoteMode>("idle");
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [status, setStatus] = useState("Loading notes");

  const selectedNote = useMemo(
    () => notes.find((note) => note.id === selectedId) ?? null,
    [notes, selectedId]
  );

  useEffect(() => {
    listNotes()
      .then((nextNotes) => {
        setNotes(nextNotes);
        if (nextNotes[0]) {
          selectNote(nextNotes[0]);
        }
        setStatus(nextNotes.length === 0 ? "No notes yet" : `${nextNotes.length} note(s)`);
      })
      .catch((error) => setStatus(formatError(error)));
  }, []);

  function selectNote(note: Note) {
    setSelectedId(note.id);
    setNoteMode("view");
    setTitle(note.title);
    setBody(note.body);
  }

  function onNewNote() {
    setSelectedId(null);
    setNoteMode("create");
    setTitle("Untitled note");
    setBody("");
    setStatus("New note");
  }

  async function onSaveNote() {
    const nextTitle = title.trim();
    if (!nextTitle) {
      setStatus("Note title is required");
      return;
    }

    try {
      if (selectedNote) {
        const updated = await updateNote(selectedNote.id, nextTitle, body);
        setNotes((current) => current.map((note) => (note.id === updated.id ? updated : note)));
        selectNote(updated);
        setStatus("Note saved");
      } else {
        const note = await createNote(nextTitle, body);
        setNotes((current) => [note, ...current]);
        selectNote(note);
        setStatus("Note added");
      }
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function onDeleteNote() {
    if (!selectedNote) {
      return;
    }

    try {
      await deleteNote(selectedNote.id);
      const remaining = notes.filter((note) => note.id !== selectedNote.id);
      setNotes(remaining);
      if (remaining[0]) {
        selectNote(remaining[0]);
      } else {
        setSelectedId(null);
        setNoteMode("idle");
        setTitle("");
        setBody("");
      }
      setStatus("Note deleted");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  return (
    <section className="feature-panel">
      <div className="panel-heading">
        <div>
          <p>Local Organizer</p>
          <h3>Notes</h3>
        </div>
        <span className="save-pill">{status}</span>
      </div>

      <div className="split-feature-body">
        <aside className="sub-list" aria-label="Notes list">
          <button className="primary-button full-width" onClick={onNewNote} type="button">
            New Note
          </button>
          {notes.map((note) => (
            <button
              className={note.id === selectedId ? "sub-list-item active" : "sub-list-item"}
              key={note.id}
              onClick={() => selectNote(note)}
              type="button"
            >
              <strong>{note.title}</strong>
              <span>{note.updatedAt}</span>
            </button>
          ))}
        </aside>

        {selectedNote || noteMode === "create" ? (
          <form className="editor-form">
            <input
              aria-label="Note title"
              className="text-input"
              readOnly={noteMode === "view"}
              onChange={(event) => setTitle(event.target.value)}
              placeholder="Note title"
              value={title}
            />
            <textarea
              aria-label="Note body"
              className="body-input"
              readOnly={noteMode === "view"}
              onChange={(event) => setBody(event.target.value)}
              placeholder="Note body"
              value={body}
            />
            <div className="form-actions">
              {noteMode === "view" ? (
                <button className="primary-button" onClick={() => setNoteMode("edit")} type="button">
                  Edit
                </button>
              ) : (
                <button className="primary-button" onClick={() => void onSaveNote()} type="button">
                  {selectedNote ? "Save" : "Add"}
                </button>
              )}
              {selectedNote && (
                <button className="primary-button" onClick={() => void onDeleteNote()} type="button">
                  Delete
                </button>
              )}
            </div>
          </form>
        ) : (
          <div className="empty-editor-state">
            <p>Create a note to begin.</p>
          </div>
        )}
      </div>
    </section>
  );
}

function formatError(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
