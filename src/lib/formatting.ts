export function formatWindowDuration(minutes: number | null): string {
  if (!minutes || minutes < 1) return "-";
  if (minutes % 1440 === 0) return `${minutes / 1440}d`;
  if (minutes % 60 === 0) return `${minutes / 60}h`;
  if (minutes > 60) return `${Math.floor(minutes / 60)}h${minutes % 60}m`;
  return `${minutes}m`;
}

export function resetDescription(resetsAt: number | null): string {
  if (!resetsAt) return "Reset time unavailable";
  const remaining = resetsAt * 1000 - Date.now();
  if (remaining <= 0) return "Resetting now";
  const minutes = Math.ceil(remaining / 60000);
  if (minutes < 60) return `Resets in ${minutes}m`;
  if (minutes < 24 * 60)
    return `Resets in ${Math.floor(minutes / 60)}h ${minutes % 60}m`;
  return `Resets ${new Intl.DateTimeFormat(undefined, { weekday: "long", hour: "2-digit", minute: "2-digit" }).format(resetsAt * 1000)}`;
}
