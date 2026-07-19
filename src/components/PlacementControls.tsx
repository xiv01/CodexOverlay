import { Pin, Settings2 } from "lucide-react";

export function PlacementControls({
  onPin,
  onSettings,
  settingsOpen,
}: {
  onPin: () => void;
  onSettings: () => void;
  settingsOpen: boolean;
}) {
  return (
    <span className="placement-controls">
      <button
        className={`pin ${settingsOpen ? "active" : ""}`}
        onClick={onSettings}
        title="Overlay settings"
        aria-label="Overlay settings"
      >
        <Settings2 size={14} strokeWidth={1.8} />
      </button>
      <button
        className="pin"
        onClick={onPin}
        title="Pin overlay (Enter)"
        aria-label="Pin overlay"
      >
        <Pin size={14} strokeWidth={1.8} />
      </button>
    </span>
  );
}
