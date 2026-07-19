import type { CodexUsageState, QuotaWindow } from "../types/usage";
import { resetDescription } from "../lib/formatting";

function Row({ quota }: { quota: QuotaWindow | null }) {
  if (!quota) return null;
  return (
    <>
      <div>{quota.label} remaining</div>
      <b>{Math.round(quota.remainingPercent ?? 0)}%</b>
      <div>{resetDescription(quota.resetsAt)}</div>
      <span />
    </>
  );
}
export function DetailsView({
  state,
  visible,
}: {
  state: CodexUsageState;
  visible: boolean;
}) {
  if (!visible) return null;
  return (
    <section className="details">
      <Row quota={state.primary} />
      <Row quota={state.secondary} />
      {state.planType && (
        <>
          <div>Plan</div>
          <b>{state.planType}</b>
        </>
      )}
      {state.resetCreditCount !== null && (
        <>
          <div>Reset credits</div>
          <b>{state.resetCreditCount}</b>
        </>
      )}
      <div>Status</div>
      <b>{state.status.replaceAll("-", " ")}</b>
    </section>
  );
}
