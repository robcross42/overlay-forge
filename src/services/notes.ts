import { invoke } from "@tauri-apps/api/core";

export type Note = {
  id: number;
  title: string;
  body: string;
  createdAt: string;
  updatedAt: string;
};

export function listNotes() {
  return invoke<Note[]>("list_notes");
}

export function createNote(title: string, body: string) {
  return invoke<Note>("create_note", { title, body });
}

export function updateNote(id: number, title: string, body: string) {
  return invoke<Note>("update_note", { id, title, body });
}

export function deleteNote(id: number) {
  return invoke<void>("delete_note", { id });
}

