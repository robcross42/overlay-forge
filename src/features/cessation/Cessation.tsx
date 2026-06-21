import { listen } from "@tauri-apps/api/event";
import { useEffect, useMemo, useState } from "react";
import {
  deleteSmokingEvent,
  exportSmokingCessationChatGptContext,
  getSmokingCessationSettings,
  listSmokingEvents,
  recordSmokingEvent,
  updateSmokingCigaretteCount
} from "../../services/smokingCessation";
import type { SmokingCessationSettings, SmokingEvent } from "../../services/smokingCessation";

type GraphRange = "day" | "week" | "month" | "year";

export function Cessation() {
  const [events, setEvents] = useState<SmokingEvent[]>([]);
  const [settings, setSettings] = useState<SmokingCessationSettings | null>(null);
  const [range, setRange] = useState<GraphRange>("day");
  const [status, setStatus] = useState("Loading cessation data");
  const [exportPath, setExportPath] = useState("");
  const [cigaretteCountInput, setCigaretteCountInput] = useState("0");
  const [clockTick, setClockTick] = useState(() => Date.now());

  const grouped = useMemo(() => groupSmokingEvents(events, range), [events, range]);
  const todayCount = useMemo(() => {
    const today = localDateKey(new Date());
    return events.filter((event) => event.smokedAt.slice(0, 10) === today).length;
  }, [events]);
  const maxCount = Math.max(1, ...grouped.map((item) => item.count));
  const lastEvent = events[0] ?? null;
  const intervalSeries = useMemo(() => buildIntervalSeries(events), [events]);
  const averageIntervalMinutes = useMemo(() => averageRecentIntervalMinutes(events), [events]);
  const predictedRunOut = useMemo(
    () => predictRunOut(settings?.currentCigaretteCount ?? 0, averageIntervalMinutes, clockTick),
    [averageIntervalMinutes, clockTick, settings?.currentCigaretteCount]
  );

  useEffect(() => {
    void refresh();
    let isMounted = true;
    let cleanup: (() => void) | null = null;

    listen<SmokingEvent>("smoking-event-recorded", (event) => {
      if (!isMounted) {
        return;
      }
      setEvents((current) => [event.payload, ...current]);
      setSettings((current) =>
        current
          ? {
              ...current,
              currentCigaretteCount: Math.max(current.currentCigaretteCount - 1, 0)
            }
          : current
      );
      setCigaretteCountInput((current) => String(Math.max((Number(current) || 0) - 1, 0)));
      setStatus(`Recorded cigarette at ${formatDateTime(event.payload.smokedAt)}`);
      void exportSmokingCessationChatGptContext()
        .then((exportRecord) => setExportPath(exportRecord.exportPath))
        .catch((error) => setStatus(formatError(error)));
    }).then((nextCleanup) => {
      if (isMounted) {
        cleanup = nextCleanup;
      } else {
        nextCleanup();
      }
    });

    return () => {
      isMounted = false;
      cleanup?.();
    };
  }, []);

  useEffect(() => {
    const intervalId = window.setInterval(() => setClockTick(Date.now()), 60000);
    return () => window.clearInterval(intervalId);
  }, []);

  async function refresh() {
    try {
      const [nextEvents, nextSettings] = await Promise.all([
        listSmokingEvents(),
        getSmokingCessationSettings()
      ]);
      const exportRecord = await exportSmokingCessationChatGptContext();
      setEvents(nextEvents);
      setSettings(nextSettings);
      setCigaretteCountInput(String(nextSettings.currentCigaretteCount));
      setExportPath(exportRecord.exportPath);
      setStatus(nextEvents.length === 0 ? "No cigarettes recorded" : `${nextEvents.length} record(s)`);
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function recordNow() {
    try {
      const event = await recordSmokingEvent(undefined, "Recorded from Cessation module");
      const nextSettings = await getSmokingCessationSettings();
      const exportRecord = await exportSmokingCessationChatGptContext();
      setEvents((current) => [event, ...current]);
      setSettings(nextSettings);
      setCigaretteCountInput(String(nextSettings.currentCigaretteCount));
      setExportPath(exportRecord.exportPath);
      setStatus(`Recorded cigarette at ${formatDateTime(event.smokedAt)}`);
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function saveCigaretteCount() {
    try {
      const nextCount = Math.max(0, Math.floor(Number(cigaretteCountInput) || 0));
      const nextSettings = await updateSmokingCigaretteCount(nextCount);
      const exportRecord = await exportSmokingCessationChatGptContext();
      setSettings(nextSettings);
      setCigaretteCountInput(String(nextSettings.currentCigaretteCount));
      setExportPath(exportRecord.exportPath);
      setStatus(`Current cigarettes set to ${nextSettings.currentCigaretteCount}`);
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function removeEvent(event: SmokingEvent) {
    try {
      await deleteSmokingEvent(event.id);
      const exportRecord = await exportSmokingCessationChatGptContext();
      setEvents((current) => current.filter((item) => item.id !== event.id));
      setExportPath(exportRecord.exportPath);
      setStatus("Record deleted");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  return (
    <section className="feature-panel cessation-panel">
      <div className="panel-heading">
        <div>
          <p>Personal Tracking</p>
          <h3>Smoking Cessation</h3>
        </div>
        <span className="save-pill">{status}</span>
      </div>

      <div className="cessation-dashboard">
        <section className="cessation-summary" aria-label="Cessation summary">
          <article>
            <span>Today</span>
            <strong>{todayCount}</strong>
          </article>
          <article>
            <span>Total</span>
            <strong>{events.length}</strong>
          </article>
          <article>
            <span>Last recorded</span>
            <strong>{lastEvent ? formatDateTime(lastEvent.smokedAt) : "None"}</strong>
          </article>
          <article>
            <span>Remaining</span>
            <strong>{settings?.currentCigaretteCount ?? 0}</strong>
          </article>
          <article>
            <span>Average spacing</span>
            <strong>{formatIntervalMinutes(averageIntervalMinutes)}</strong>
          </article>
          <article>
            <span>Predicted run-out</span>
            <strong>{predictedRunOut}</strong>
          </article>
        </section>

        {settings && (
          <section className="cessation-patch-card" aria-label="Nicotine patch">
            <div>
              <p>Patch</p>
              <h4>{settings.patchLabel}</h4>
            </div>
            <strong>
              Started {formatDateTime(settings.patchStartedAt)} {settings.patchTimezone}
            </strong>
          </section>
        )}

        <section className="cessation-actions" aria-label="Record cigarette">
          <button className="primary-button" onClick={() => void recordNow()} type="button">
            Record Cigarette
          </button>
          <label className="cessation-count-input">
            <span>Current cigarettes</span>
            <input
              className="text-input"
              min="0"
              onChange={(event) => setCigaretteCountInput(event.target.value)}
              type="number"
              value={cigaretteCountInput}
            />
          </label>
          <button className="ghost-button" onClick={() => void saveCigaretteCount()} type="button">
            Save Count
          </button>
          <button className="ghost-button" onClick={() => void refresh()} type="button">
            Refresh
          </button>
        </section>

        <section className="cessation-export-card" aria-label="ChatGPT export">
          <div>
            <p>ChatGPT Context</p>
            <h4>Auto-export</h4>
          </div>
          <code>{exportPath || "Export path will appear after refresh."}</code>
        </section>

        <section className="cessation-chart-card" aria-label="Smoking interval graph">
          <div className="settings-card-heading-row">
            <div>
              <p>Rate</p>
              <h4>Minutes Between Cigarettes</h4>
            </div>
            <strong>{formatIntervalMinutes(averageIntervalMinutes)} avg</strong>
          </div>

          {intervalSeries.length < 2 ? (
            <p>Record at least two cigarettes to calculate spacing.</p>
          ) : (
            <div className="cessation-line-chart">
              <svg viewBox="0 0 100 48" preserveAspectRatio="none" aria-hidden="true">
                <polyline points={lineChartPoints(intervalSeries)} />
              </svg>
              <span>
                Latest: {intervalSeries[intervalSeries.length - 1]?.label} -{" "}
                {Math.round(intervalSeries[intervalSeries.length - 1]?.minutes ?? 0)}m
              </span>
            </div>
          )}
        </section>

        <section className="cessation-chart-card" aria-label="Smoking chart">
          <div className="settings-card-heading-row">
            <div>
              <p>Usage</p>
              <h4>{rangeLabel(range)}</h4>
            </div>
            <div className="segmented-control">
              {(["day", "week", "month", "year"] as GraphRange[]).map((item) => (
                <button
                  className={item === range ? "active" : ""}
                  key={item}
                  onClick={() => setRange(item)}
                  type="button"
                >
                  {item}
                </button>
              ))}
            </div>
          </div>

          <div className="cessation-chart">
            {grouped.length === 0 ? (
              <p>No records for this range.</p>
            ) : (
              grouped.map((item) => (
                <div className="cessation-chart-row" key={item.key}>
                  <span>{item.label}</span>
                  <div>
                    <i style={{ width: `${Math.max(4, (item.count / maxCount) * 100)}%` }} />
                  </div>
                  <strong>{item.count}</strong>
                </div>
              ))
            )}
          </div>
        </section>

        <section className="cessation-history" aria-label="Smoking event history">
          <div>
            <p>History</p>
            <h4>Recent Records</h4>
          </div>
          {events.length === 0 ? (
            <p>No smoking records yet.</p>
          ) : (
            <div className="cessation-event-list">
              {events.slice(0, 20).map((event) => (
                <div className="cessation-event-row" key={event.id}>
                  <div>
                    <strong>{formatDateTime(event.smokedAt)}</strong>
                    <span>{event.source}</span>
                  </div>
                  <button
                    className="ghost-button"
                    onClick={() => void removeEvent(event)}
                    type="button"
                  >
                    Delete
                  </button>
                </div>
              ))}
            </div>
          )}
        </section>
      </div>
    </section>
  );
}

function groupSmokingEvents(events: SmokingEvent[], range: GraphRange) {
  const counts = new Map<string, number>();
  for (const event of events) {
    const date = parseLocalDateTime(event.smokedAt);
    const key = groupKey(date, range);
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }

  return [...counts.entries()]
    .sort(([left], [right]) => right.localeCompare(left))
    .slice(0, range === "day" ? 14 : 12)
    .reverse()
    .map(([key, count]) => ({
      key,
      count,
      label: groupLabel(key, range)
    }));
}

function groupKey(date: Date, range: GraphRange) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  if (range === "day") {
    return `${year}-${month}-${day}`;
  }
  if (range === "month") {
    return `${year}-${month}`;
  }
  if (range === "year") {
    return `${year}`;
  }
  return `${year}-W${String(weekNumber(date)).padStart(2, "0")}`;
}

function groupLabel(key: string, range: GraphRange) {
  if (range === "day") {
    return key.slice(5);
  }
  return key;
}

function weekNumber(date: Date) {
  const copy = new Date(Date.UTC(date.getFullYear(), date.getMonth(), date.getDate()));
  const dayNum = copy.getUTCDay() || 7;
  copy.setUTCDate(copy.getUTCDate() + 4 - dayNum);
  const yearStart = new Date(Date.UTC(copy.getUTCFullYear(), 0, 1));
  return Math.ceil(((copy.getTime() - yearStart.getTime()) / 86400000 + 1) / 7);
}

function parseLocalDateTime(value: string) {
  return new Date(value.replace(" ", "T"));
}

function localDateKey(date: Date) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function formatDateTime(value: string) {
  return value.replace("T", " ").slice(0, 16);
}

function rangeLabel(range: GraphRange) {
  if (range === "day") return "By day";
  if (range === "week") return "By week";
  if (range === "month") return "By month";
  return "By year";
}

function buildIntervalSeries(events: SmokingEvent[]) {
  const ordered = [...events]
    .map((event) => parseLocalDateTime(event.smokedAt))
    .filter((date) => !Number.isNaN(date.getTime()))
    .sort((left, right) => left.getTime() - right.getTime());
  const intervals = [];
  for (let index = 1; index < ordered.length; index += 1) {
    const minutes = (ordered[index].getTime() - ordered[index - 1].getTime()) / 60000;
    if (minutes >= 0) {
      intervals.push({
        label: formatTimeOnly(ordered[index]),
        minutes
      });
    }
  }
  return intervals.slice(-12);
}

function averageRecentIntervalMinutes(events: SmokingEvent[]) {
  const intervals = buildIntervalSeries(events).map((item) => item.minutes).filter((value) => value > 0);
  if (intervals.length === 0) {
    return null;
  }
  const recent = intervals.slice(-8);
  return recent.reduce((sum, value) => sum + value, 0) / recent.length;
}

function predictRunOut(currentCount: number, averageMinutes: number | null, now: number) {
  if (currentCount <= 0) {
    return "Now";
  }
  if (!averageMinutes || averageMinutes <= 0) {
    return "Need 2 records";
  }
  return formatDateTimeFromDate(new Date(now + currentCount * averageMinutes * 60000));
}

function lineChartPoints(series: { minutes: number }[]) {
  const maxMinutes = Math.max(1, ...series.map((item) => item.minutes));
  const lastIndex = Math.max(1, series.length - 1);
  return series
    .map((item, index) => {
      const x = (index / lastIndex) * 100;
      const y = 44 - (item.minutes / maxMinutes) * 40;
      return `${x.toFixed(2)},${Math.max(4, y).toFixed(2)}`;
    })
    .join(" ");
}

function formatIntervalMinutes(value: number | null) {
  if (!value || value <= 0) {
    return "Need 2 records";
  }
  if (value < 60) {
    return `${Math.round(value)}m`;
  }
  const hours = Math.floor(value / 60);
  const minutes = Math.round(value % 60);
  return `${hours}h ${minutes}m`;
}

function formatTimeOnly(date: Date) {
  return `${String(date.getHours()).padStart(2, "0")}:${String(date.getMinutes()).padStart(2, "0")}`;
}

function formatDateTimeFromDate(date: Date) {
  return `${localDateKey(date)} ${formatTimeOnly(date)}`;
}

function formatError(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
