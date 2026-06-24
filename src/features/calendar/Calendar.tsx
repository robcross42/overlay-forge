import { useEffect, useMemo, useState } from "react";
import {
  createCalendarEvent,
  deleteCalendarEvent,
  listCalendarEvents,
  updateCalendarEvent
} from "../../services/calendar";
import type { CalendarEvent, CalendarEventInput } from "../../services/calendar";
import { formatUnknownError as formatError } from "../../utils/errors";

const emptyEvent: CalendarEventInput = {
  title: "",
  startDate: new Date().toISOString().slice(0, 10),
  startTime: "09:00",
  endDate: new Date().toISOString().slice(0, 10),
  endTime: "10:00",
  notes: ""
};

type CalendarMode = "idle" | "create" | "view" | "edit";

export function Calendar() {
  const [events, setEvents] = useState<CalendarEvent[]>([]);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [calendarMode, setCalendarMode] = useState<CalendarMode>("idle");
  const [form, setForm] = useState<CalendarEventInput>(emptyEvent);
  const [status, setStatus] = useState("Loading events");

  const selectedEvent = useMemo(
    () => events.find((event) => event.id === selectedId) ?? null,
    [events, selectedId]
  );

  useEffect(() => {
    listCalendarEvents()
      .then((nextEvents) => {
        setEvents(nextEvents);
        setStatus(nextEvents.length === 0 ? "No events yet" : `${nextEvents.length} event(s)`);
      })
      .catch((error) => setStatus(formatError(error)));
  }, []);

  function selectEvent(event: CalendarEvent) {
    setSelectedId(event.id);
    setCalendarMode("view");
    setForm({
      title: event.title,
      startDate: event.startDate,
      startTime: event.startTime,
      endDate: event.endDate,
      endTime: event.endTime,
      notes: event.notes
    });
  }

  function resetForm() {
    setSelectedId(null);
    setCalendarMode("create");
    setForm({ ...emptyEvent });
    setStatus("New event");
  }

  async function onSaveEvent() {
    try {
      if (selectedEvent) {
        const updated = await updateCalendarEvent(selectedEvent.id, form);
        setEvents((current) =>
          current.map((item) => (item.id === updated.id ? updated : item)).sort(sortEvents)
        );
        selectEvent(updated);
        setStatus("Event updated");
      } else {
        const created = await createCalendarEvent(form);
        setEvents((current) => [...current, created].sort(sortEvents));
        setStatus("Event added");
        selectEvent(created);
      }
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function onDeleteEvent() {
    if (!selectedEvent) {
      return;
    }

    try {
      await deleteCalendarEvent(selectedEvent.id);
      setEvents((current) => current.filter((event) => event.id !== selectedEvent.id));
      setSelectedId(null);
      setCalendarMode("idle");
      setForm({ ...emptyEvent });
      setStatus("Event deleted");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  function updateField<K extends keyof CalendarEventInput>(field: K, value: CalendarEventInput[K]) {
    setForm((current) => {
      if (field === "startDate") {
        return { ...current, startDate: value, endDate: value };
      }

      if (field === "startTime") {
        const nextEnd = addOneHour(current.startDate, value);
        return {
          ...current,
          startTime: value,
          endDate: nextEnd.date,
          endTime: nextEnd.time
        };
      }

      return { ...current, [field]: value };
    });
  }

  return (
    <section className="feature-panel">
      <div className="panel-heading">
        <div>
          <p>Local Organizer</p>
          <h3>Calendar</h3>
        </div>
        <span className="save-pill">{status}</span>
      </div>

      <div className="split-feature-body">
        <div className="sub-list" aria-label="Calendar events">
          <button className="primary-button full-width" onClick={resetForm} type="button">
            New Event
          </button>
          {events.map((event) => (
            <button
              className={event.id === selectedId ? "sub-list-item active" : "sub-list-item"}
              key={event.id}
              onClick={() => selectEvent(event)}
              type="button"
            >
              <strong>{event.title}</strong>
              <span>
                {event.startDate} {event.startTime}
              </span>
            </button>
          ))}
        </div>

        {selectedEvent || calendarMode === "create" ? (
          <form className="editor-form">
            <input
              aria-label="Event title"
              className="text-input"
              readOnly={calendarMode === "view"}
              onChange={(event) => updateField("title", event.target.value)}
              placeholder="Event title"
              value={form.title}
            />

            <div className="field-grid">
              <label>
                <span>Start date</span>
                <input
                  className="text-input"
                  readOnly={calendarMode === "view"}
                  onClick={(event) => {
                    if (calendarMode !== "view") {
                      showNativePicker(event.currentTarget);
                    }
                  }}
                  onFocus={(event) => {
                    if (calendarMode !== "view") {
                      showNativePicker(event.currentTarget);
                    }
                  }}
                  onChange={(event) => updateField("startDate", event.target.value)}
                  type="date"
                  value={form.startDate}
                />
              </label>
              <label>
                <span>Start time</span>
                <input
                  className="text-input"
                  readOnly={calendarMode === "view"}
                  onClick={(event) => {
                    if (calendarMode !== "view") {
                      showNativePicker(event.currentTarget);
                    }
                  }}
                  onFocus={(event) => {
                    if (calendarMode !== "view") {
                      showNativePicker(event.currentTarget);
                    }
                  }}
                  onChange={(event) => updateField("startTime", event.target.value)}
                  type="time"
                  value={form.startTime}
                />
              </label>
              <label>
                <span>End date</span>
                <input
                  className="text-input"
                  readOnly={calendarMode === "view"}
                  onClick={(event) => {
                    if (calendarMode !== "view") {
                      showNativePicker(event.currentTarget);
                    }
                  }}
                  onFocus={(event) => {
                    if (calendarMode !== "view") {
                      showNativePicker(event.currentTarget);
                    }
                  }}
                  onChange={(event) => updateField("endDate", event.target.value)}
                  type="date"
                  value={form.endDate}
                />
              </label>
              <label>
                <span>End time</span>
                <input
                  className="text-input"
                  readOnly={calendarMode === "view"}
                  onClick={(event) => {
                    if (calendarMode !== "view") {
                      showNativePicker(event.currentTarget);
                    }
                  }}
                  onFocus={(event) => {
                    if (calendarMode !== "view") {
                      showNativePicker(event.currentTarget);
                    }
                  }}
                  onChange={(event) => updateField("endTime", event.target.value)}
                  type="time"
                  value={form.endTime}
                />
              </label>
            </div>

            <textarea
              aria-label="Event notes"
              className="body-input compact"
              readOnly={calendarMode === "view"}
              onChange={(event) => updateField("notes", event.target.value)}
              placeholder="Event notes"
              value={form.notes}
            />

            <div className="form-actions">
              {calendarMode === "view" ? (
                <button className="primary-button" onClick={() => setCalendarMode("edit")} type="button">
                  Edit
                </button>
              ) : (
                <button className="primary-button" onClick={() => void onSaveEvent()} type="button">
                  {selectedEvent ? "Save" : "Add"}
                </button>
              )}
              {selectedEvent && (
                <button className="primary-button" onClick={() => void onDeleteEvent()} type="button">
                  Delete
                </button>
              )}
            </div>
          </form>
        ) : (
          <div className="empty-editor-state">
            <p>Create or select an event to begin.</p>
          </div>
        )}
      </div>
    </section>
  );
}

function sortEvents(first: CalendarEvent, second: CalendarEvent) {
  return `${first.startDate} ${first.startTime}`.localeCompare(`${second.startDate} ${second.startTime}`);
}

function showNativePicker(input: HTMLInputElement) {
  try {
    input.showPicker?.();
  } catch {
    input.focus();
  }
}

function addOneHour(date: string, time: string) {
  const [hours, minutes] = time.split(":").map(Number);
  if (!date || Number.isNaN(hours) || Number.isNaN(minutes)) {
    return { date, time };
  }

  const nextDate = new Date(`${date}T${time}`);
  if (Number.isNaN(nextDate.getTime())) {
    return { date, time };
  }

  nextDate.setHours(nextDate.getHours() + 1);

  return {
    date: formatDateInput(nextDate),
    time: nextDate.toTimeString().slice(0, 5)
  };
}

function formatDateInput(date: Date) {
  return [
    date.getFullYear(),
    String(date.getMonth() + 1).padStart(2, "0"),
    String(date.getDate()).padStart(2, "0")
  ].join("-");
}
