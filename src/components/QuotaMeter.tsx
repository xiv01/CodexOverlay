import type { QuotaWindow } from "../types/usage";

export function QuotaMeter({ quota }: { quota: QuotaWindow | null }) {
  if (!quota)
    return (
      <div className="quota quota-empty">
        <span>-</span>
        <span className="meter" />
        <strong>-</strong>
      </div>
    );
  const value = quota.remainingPercent ?? 0;
  const tone = value < 15 ? "critical" : value <= 35 ? "warning" : "normal";
  return (
    <div
      className={`quota ${tone}`}
      title={
        quota.resetsAt
          ? `Resets ${new Date(quota.resetsAt * 1000).toLocaleString()}`
          : undefined
      }
    >
      <span className="duration">{quota.label}</span>
      <span className="meter">
        <i style={{ width: `${value}%` }} />
      </span>
      <strong>
        {quota.remainingPercent === null ? "-" : `${Math.round(value)}%`}
      </strong>
    </div>
  );
}
