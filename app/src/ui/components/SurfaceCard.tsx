import type { ReactNode } from "react";

type SurfaceCardProps = {
  aside?: ReactNode;
  children: ReactNode;
  title: string;
  wide?: boolean;
};

export function SurfaceCard({ aside, children, title, wide = false }: SurfaceCardProps): React.JSX.Element {
  return (
    <article className={`surface-card ${wide ? "wide" : ""}`.trim()}>
      <div className="card-header">
        <h3>{title}</h3>
        {aside}
      </div>
      {children}
    </article>
  );
}
