import { invoke } from "@tauri-apps/api/core";
import type { CodexUsageState } from "../types/usage";

export type OverlayPreferences = {
  opacity: number;
  showBorder: boolean;
  showTitle: boolean;
  width: number;
};

export const backend = {
  pin: (pinned: boolean) => invoke("set_pinned", { pinned }),
  togglePlacement: () => invoke("toggle_placement"),
  refresh: () => invoke("refresh_usage"),
  details: () => invoke("show_details"),
  currentUsage: () => invoke<CodexUsageState>("current_usage_state"),
  isPinned: () => invoke<boolean>("is_pinned"),
  preferences: () => invoke<OverlayPreferences>("overlay_preferences"),
  updatePreferences: (preferences: OverlayPreferences) =>
    invoke("update_overlay_preferences", { ...preferences }),
  setSettingsOpen: (open: boolean) => invoke("set_settings_open", { open }),
};
