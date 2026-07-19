import type { UsageStatus } from "../types/usage";

export function ConnectionIndicator({ status }: { status: UsageStatus }) {
  if (status === "connected") return null;
  return (
    <span
      className={`connection ${status}`}
      title={status.replaceAll("-", " ")}
      aria-label={status}
    />
  );
}
