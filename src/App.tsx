import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { OverlayBar } from "./components/OverlayBar";
import { DetailsView } from "./components/DetailsView";
import { backend } from "./lib/tauri";
import { useUsageState } from "./state/usageStore";
import { OverlaySettings } from "./components/OverlaySettings";
import type { OverlayPreferences } from "./lib/tauri";

export function App() {
  const state = useUsageState();
  const [placement, setPlacement] = useState(true);
  const [details, setDetails] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [dragging, setDragging] = useState(false);
  const [preferences, setPreferences] = useState<OverlayPreferences>({
    opacity: 0.88,
    showBorder: true,
    showTitle: false,
    width: 370,
  });
  useEffect(() => {
    void backend
      .isPinned()
      .then((pinned) => setPlacement(!pinned))
      .catch(() => undefined);
    void backend
      .preferences()
      .then(setPreferences)
      .catch(() => undefined);
    const cleanups = Promise.all([
      listen<boolean>("placement-mode", ({ payload }) => setPlacement(payload)),
      listen("show-details", () => {
        setSettingsOpen(false);
        setDetails(true);
        void backend.setSettingsOpen(true);
      }),
      listen("show-settings", () => {
        setDetails(false);
        setPlacement(true);
        setSettingsOpen(true);
        void backend.setSettingsOpen(true);
      }),
    ]);
    const keyboard = (event: KeyboardEvent) => {
      if (event.key === "Enter" && placement) void pin();
      if (event.key === "Escape" && placement) {
        setPlacement(false);
        void backend.pin(true);
      }
    };
    addEventListener("keydown", keyboard);
    return () => {
      void cleanups.then((items) => items.forEach((unlisten) => unlisten()));
      removeEventListener("keydown", keyboard);
    };
  }, [placement]);
  async function pin() {
    await backend.setSettingsOpen(false);
    await backend.pin(true);
    setSettingsOpen(false);
    setPlacement(false);
  }
  async function toggleSettings() {
    const open = !settingsOpen;
    if (open) setDetails(false);
    setSettingsOpen(open);
    await backend.setSettingsOpen(open);
  }
  function updatePreferences(next: OverlayPreferences) {
    setPreferences(next);
    void backend
      .updatePreferences(next)
      .then(() => settingsOpen && backend.setSettingsOpen(true));
  }
  const style: React.CSSProperties = {
    width: preferences.width,
    backgroundColor: `rgba(26, 27, 30, ${preferences.opacity})`,
    border: preferences.showBorder ? undefined : "none",
  };
  return (
    <div className="shell">
      <OverlayBar
        state={state}
        placement={placement}
        onPin={() => void pin()}
        onSettings={() => void toggleSettings()}
        onDraggingChange={setDragging}
        settingsOpen={settingsOpen}
        showTitle={preferences.showTitle}
        style={style}
      />
      {placement && settingsOpen && !dragging && (
        <OverlaySettings
          preferences={preferences}
          onChange={updatePreferences}
        />
      )}
      {placement && !settingsOpen && (
        <div className="hint">drag to position · pin when ready</div>
      )}
      <DetailsView state={state} visible={details && !settingsOpen} />
    </div>
  );
}
