import { invoke } from "@tauri-apps/api/core";

export type YouTubeReference = {
  id: number;
  title: string;
  url: string;
  videoId: string;
  channelName: string;
  notes: string;
  tags: string;
  createdAt: string;
  updatedAt: string;
};

export type YouTubeReferenceInput = {
  title: string;
  url: string;
  channelName: string;
  notes: string;
  tags: string;
};

export function listYouTubeReferences() {
  return invoke<YouTubeReference[]>("list_youtube_references");
}

export function getYouTubeReference(id: number) {
  return invoke<YouTubeReference>("get_youtube_reference", { id });
}

export function createYouTubeReference(input: YouTubeReferenceInput) {
  return invoke<YouTubeReference>("create_youtube_reference", input);
}

export function updateYouTubeReference(id: number, input: YouTubeReferenceInput) {
  return invoke<YouTubeReference>("update_youtube_reference", { id, ...input });
}

export function deleteYouTubeReference(id: number) {
  return invoke<void>("delete_youtube_reference", { id });
}

export function openYouTubeReference(id: number) {
  return invoke<void>("open_youtube_reference", { id });
}
