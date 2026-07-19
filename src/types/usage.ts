export type UsageStatus = "starting" | "connected" | "stale" | "reconnecting" | "logged-out" | "codex-not-found" | "unsupported-auth" | "error";

export type QuotaWindow = {
  usedPercent: number | null;
  remainingPercent: number | null;
  windowDurationMins: number | null;
  label: string;
  resetsAt: number | null;
};

export type CodexUsageState = {
  status: UsageStatus;
  primary: QuotaWindow | null;
  secondary: QuotaWindow | null;
  planType: string | null;
  rateLimitReachedType: string | null;
  resetCreditCount: number | null;
  lastSuccessfulRefreshAt: number | null;
  errorMessage: string | null;
};
