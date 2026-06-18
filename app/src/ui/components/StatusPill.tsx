import type { LeagueClientCard } from "../../domain/league";

export function StatusPill({
  children,
  variant,
}: {
  children: string;
  variant: LeagueClientCard["variant"];
}): React.JSX.Element {
  return <span className="status-pill" data-variant={variant}>{children}</span>;
}
