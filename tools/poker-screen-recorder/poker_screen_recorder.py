"""Passive full-screen screenshot recorder for post-session poker review.

This utility only captures PNG images when the user clicks Capture now or enables
the configured interval. It does not inspect images, detect cards, read client
state, control another application, or provide recommendations.
"""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass
from datetime import datetime
from pathlib import Path
from tkinter import BooleanVar, StringVar, Tk, messagebox, ttk

from PIL import ImageGrab


APP_DIRECTORY = Path(__file__).resolve().parent
CONFIG_PATH = APP_DIRECTORY / "recorder_config.json"


@dataclass
class RecorderConfig:
    """User-configurable, non-sensitive recorder settings."""

    output_directory: str = "captures"
    interval_seconds: int = 300

    @classmethod
    def load(cls) -> "RecorderConfig":
        if not CONFIG_PATH.exists():
            config = cls()
            config.save()
            return config
        try:
            payload = json.loads(CONFIG_PATH.read_text(encoding="utf-8"))
            interval = int(payload.get("interval_seconds", cls.interval_seconds))
            if interval < 1:
                raise ValueError("interval_seconds must be at least 1")
            output = str(payload.get("output_directory", cls.output_directory)).strip()
            if not output:
                raise ValueError("output_directory cannot be empty")
            return cls(output_directory=output, interval_seconds=interval)
        except (OSError, ValueError, json.JSONDecodeError) as error:
            raise RuntimeError(f"Could not read {CONFIG_PATH.name}: {error}") from error

    def save(self) -> None:
        CONFIG_PATH.write_text(json.dumps(asdict(self), indent=2) + "\n", encoding="utf-8")

    def resolved_output_directory(self) -> Path:
        path = Path(self.output_directory)
        return path if path.is_absolute() else APP_DIRECTORY / path


class PassiveScreenRecorder:
    """Owns capture file creation; it intentionally has no image-analysis behavior."""

    def __init__(self, config: RecorderConfig) -> None:
        self._config = config

    def capture(self) -> Path:
        output_directory = self._config.resolved_output_directory()
        output_directory.mkdir(parents=True, exist_ok=True)
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S_%f")
        destination = output_directory / f"poker_screen_{timestamp}.png"
        # all_screens=True records visible displays only; it does not target or control a client.
        image = ImageGrab.grab(all_screens=True)
        image.save(destination, format="PNG")
        return destination


class RecorderWindow:
    """Small local UI for explicit manual capture and opt-in interval capture."""

    def __init__(self, root: Tk, config: RecorderConfig) -> None:
        self._root = root
        self._config = config
        self._recorder = PassiveScreenRecorder(config)
        self._interval_enabled = BooleanVar(value=False)
        self._status = StringVar(value="Ready. Captures are for post-session review only.")
        self._timer_id: str | None = None

        root.title("Poker Screen Recorder")
        root.resizable(False, False)
        root.protocol("WM_DELETE_WINDOW", self.close)

        frame = ttk.Frame(root, padding=16)
        frame.grid(sticky="nsew")
        ttk.Label(frame, text="Passive screenshot recorder", font=("Segoe UI", 12, "bold")).grid(
            row=0, column=0, columnspan=2, sticky="w"
        )
        ttk.Label(
            frame,
            text="Does not read cards, analyze hands, control a poker client, or give live advice.",
            wraplength=420,
        ).grid(row=1, column=0, columnspan=2, pady=(5, 12), sticky="w")
        ttk.Button(frame, text="Capture now", command=self.capture_now).grid(row=2, column=0, sticky="w")
        ttk.Checkbutton(
            frame,
            text=f"Capture every {config.interval_seconds} seconds",
            variable=self._interval_enabled,
            command=self.toggle_interval,
        ).grid(row=2, column=1, padx=(16, 0), sticky="w")
        ttk.Label(frame, textvariable=self._status, wraplength=420).grid(
            row=3, column=0, columnspan=2, pady=(14, 0), sticky="w"
        )

    def capture_now(self) -> None:
        try:
            saved_file = self._recorder.capture()
            self._status.set(f"Saved {saved_file.name}")
        except (OSError, ValueError) as error:
            self._status.set("Capture failed.")
            messagebox.showerror("Poker Screen Recorder", str(error))

    def toggle_interval(self) -> None:
        self.cancel_interval()
        if self._interval_enabled.get():
            self._status.set(f"Interval capture enabled every {self._config.interval_seconds} seconds.")
            self._schedule_next_capture()
        else:
            self._status.set("Interval capture disabled.")

    def _schedule_next_capture(self) -> None:
        self._timer_id = self._root.after(self._config.interval_seconds * 1000, self.capture_on_interval)

    def capture_on_interval(self) -> None:
        self._timer_id = None
        if not self._interval_enabled.get():
            return
        self.capture_now()
        if self._interval_enabled.get():
            self._schedule_next_capture()

    def cancel_interval(self) -> None:
        if self._timer_id is not None:
            self._root.after_cancel(self._timer_id)
            self._timer_id = None

    def close(self) -> None:
        self.cancel_interval()
        self._root.destroy()


def main() -> None:
    try:
        config = RecorderConfig.load()
    except RuntimeError as error:
        root = Tk()
        root.withdraw()
        messagebox.showerror("Poker Screen Recorder", str(error))
        return
    root = Tk()
    RecorderWindow(root, config)
    root.mainloop()


if __name__ == "__main__":
    main()
