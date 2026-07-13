import { useEffect, useRef, useState } from "react";
import type { Game } from "../../services/gaming";

type GameContextPickerProps = {
  games: Game[];
  onSelectGame: (gameId: number) => void;
  selectedGame: Game;
};

export function GameContextPicker({
  games,
  onSelectGame,
  selectedGame
}: GameContextPickerProps) {
  const [isOpen, setIsOpen] = useState(false);
  const pickerRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!isOpen) {
      return;
    }

    function closeOnOutsidePointer(event: PointerEvent) {
      if (!pickerRef.current?.contains(event.target as Node)) {
        setIsOpen(false);
      }
    }

    function closeOnEscape(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setIsOpen(false);
      }
    }

    document.addEventListener("pointerdown", closeOnOutsidePointer);
    document.addEventListener("keydown", closeOnEscape);
    return () => {
      document.removeEventListener("pointerdown", closeOnOutsidePointer);
      document.removeEventListener("keydown", closeOnEscape);
    };
  }, [isOpen]);

  return (
    <div className="game-context-picker" ref={pickerRef}>
      <button
        aria-expanded={isOpen}
        aria-haspopup="menu"
        className="ghost-button game-context-picker-trigger"
        onClick={() => setIsOpen((current) => !current)}
        title={`Current game context: ${selectedGame.name}`}
        type="button"
      >
        <strong>{selectedGame.name}</strong>
        <span aria-hidden="true">{isOpen ? "▲" : "▼"}</span>
      </button>

      {isOpen && (
        <div aria-label="Select game context" className="game-context-picker-menu" role="menu">
          {games.map((game) => (
            <button
              aria-checked={game.id === selectedGame.id}
              className={game.id === selectedGame.id ? "active" : ""}
              key={game.id}
              onClick={() => {
                onSelectGame(game.id);
                setIsOpen(false);
              }}
              role="menuitemradio"
              type="button"
            >
              <strong>{game.name}</strong>
              {game.id === selectedGame.id && <span>Current</span>}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
