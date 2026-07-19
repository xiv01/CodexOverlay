import type { OverlayPreferences } from "../lib/tauri";
import { useEffect, useState } from "react";

export function OverlaySettings({
  preferences,
  onChange,
}: {
  preferences: OverlayPreferences;
  onChange: (next: OverlayPreferences) => void;
}) {
  const [draft, setDraft] = useState(preferences);
  useEffect(() => setDraft(preferences), [preferences]);
  const commit = () => onChange(draft);
  return (
    <section className="overlay-settings">
      <label>
        <span>Opacity</span>
        <input
          type="range"
          min="55"
          max="100"
          value={Math.round(draft.opacity * 100)}
          onChange={(event) =>
            setDraft({ ...draft, opacity: Number(event.target.value) / 100 })
          }
          onPointerUp={commit}
          onKeyUp={commit}
          onBlur={commit}
        />
        <output>{Math.round(draft.opacity * 100)}%</output>
      </label>
      <label>
        <span>Width</span>
        <input
          type="range"
          min="300"
          max="520"
          step="10"
          value={draft.width}
          onChange={(event) =>
            setDraft({ ...draft, width: Number(event.target.value) })
          }
          onPointerUp={commit}
          onKeyUp={commit}
          onBlur={commit}
        />
        <output>{draft.width}px</output>
      </label>
      <label className="toggle">
        <span>Border</span>
        <input
          type="checkbox"
          checked={draft.showBorder}
          onChange={(event) => {
            const next = { ...draft, showBorder: event.target.checked };
            setDraft(next);
            onChange(next);
          }}
        />
        <i />
      </label>
      <label className="toggle">
        <span>Title</span>
        <input
          type="checkbox"
          checked={draft.showTitle}
          onChange={(event) => {
            const next = { ...draft, showTitle: event.target.checked };
            setDraft(next);
            onChange(next);
          }}
        />
        <i />
      </label>
    </section>
  );
}
