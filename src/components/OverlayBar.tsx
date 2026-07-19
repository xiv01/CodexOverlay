import { QuotaMeter } from "./QuotaMeter";
import { ConnectionIndicator } from "./ConnectionIndicator";
import { PlacementControls } from "./PlacementControls";
import type { CodexUsageState } from "../types/usage";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function OverlayBar({
  state,
  placement,
  onPin,
  onSettings,
  onDraggingChange,
  settingsOpen,
  showTitle,
  style,
}: {
  state: CodexUsageState;
  placement: boolean;
  onPin: () => void;
  onSettings: () => void;
  onDraggingChange: (dragging: boolean) => void;
  settingsOpen: boolean;
  showTitle: boolean;
  style: React.CSSProperties;
}) {
  const unavailable =
    state.status === "codex-not-found"
      ? "Codex not found"
      : state.status === "logged-out"
        ? "Not signed in · run codex login"
        : state.status === "unsupported-auth"
          ? "Usage unavailable"
          : null;
  const primary =
    state.primary?.windowDurationMins === 300 ? state.primary : null;
  // With the 5h limit disabled, app-server may expose the 7d window as primary.
  const secondary = state.secondary ?? (primary ? null : state.primary);
  const showPrimary = primary !== null;
  const showSecondary = secondary !== null;
  const startDragging = async (event: React.PointerEvent<HTMLElement>) => {
    if (!placement || (event.target as HTMLElement).closest("button")) return;
    onDraggingChange(true);
    // Let React remove WebView form controls before entering Windows' native move loop.
    await new Promise<void>((resolve) =>
      requestAnimationFrame(() => resolve()),
    );
    try {
      await getCurrentWindow().startDragging();
    } finally {
      onDraggingChange(false);
    }
  };
  return (
    <main
      className={`pill ${placement ? "placing" : ""} ${showTitle ? "has-title" : ""} ${style.border === "none" ? "borderless" : ""}`}
      style={style}
      onPointerDown={startDragging}
    >
      {showTitle && <span className="overlay-title">Codex usage</span>}
      {unavailable ? (
        <div className="message">{unavailable}</div>
      ) : (
        <>
          {showPrimary && <QuotaMeter quota={primary} />}
          {showPrimary && showSecondary && <span className="divider" />}
          {showSecondary && <QuotaMeter quota={secondary} />}
          {!showPrimary && !showSecondary && <div className="message">-</div>}
        </>
      )}
      <ConnectionIndicator status={state.status} />
      {placement && (
        <PlacementControls
          onPin={onPin}
          onSettings={onSettings}
          settingsOpen={settingsOpen}
        />
      )}
    </main>
  );
}
