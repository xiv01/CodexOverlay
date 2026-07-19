import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import type { CodexUsageState } from "../types/usage";
import { backend } from "../lib/tauri";

const initial: CodexUsageState = {
  status: "starting",
  primary: null,
  secondary: null,
  planType: null,
  rateLimitReachedType: null,
  resetCreditCount: null,
  lastSuccessfulRefreshAt: null,
  errorMessage: null,
};

export function useUsageState() {
  const [state, setState] = useState(initial);
  useEffect(() => {
    const apply = (payload: CodexUsageState) => {
      setState((previous) => ({
        ...payload,
        primary: payload.primary ?? previous.primary,
        secondary: payload.secondary ?? previous.secondary,
      }));
    };
    void backend
      .currentUsage()
      .then(apply)
      .catch(() => undefined);
    let unlisten: (() => void) | undefined;
    void listen<CodexUsageState>("usage-state", ({ payload }) =>
      apply(payload),
    ).then((stop) => {
      unlisten = stop;
    });
    return () => {
      unlisten?.();
    };
  }, []);
  return state;
}
