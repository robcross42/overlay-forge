import { invoke } from "@tauri-apps/api/core";

export type CalendarEvent = {
  id: number;
  title: string;
  startDate: string;
  startTime: string;
  endDate: string;
  endTime: string;
  notes: string;
  createdAt: string;
  updatedAt: string;
};

export type CalendarEventInput = Omit<CalendarEvent, "id" | "createdAt" | "updatedAt">;

export function listCalendarEvents() {
  return invoke<CalendarEvent[]>("list_calendar_events");
}

export function createCalendarEvent(input: CalendarEventInput) {
  return invoke<CalendarEvent>("create_calendar_event", input);
}

export function updateCalendarEvent(id: number, input: CalendarEventInput) {
  return invoke<CalendarEvent>("update_calendar_event", { id, ...input });
}

export function deleteCalendarEvent(id: number) {
  return invoke<void>("delete_calendar_event", { id });
}

