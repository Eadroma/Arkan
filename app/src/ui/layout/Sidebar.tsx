import { useAppActions } from "../../application/useAppActions";
import { useAppStore, type ViewName } from "../../store/appStore";

const navItems: Array<{ icon: string; label: string; view?: ViewName }> = [
  { icon: "P", label: "Profil", view: "profile" },
  { icon: "M", label: "Matchs" },
  { icon: "C", label: "Champions", view: "champions" },
  { icon: "I", label: "Insights" },
  { icon: "S", label: "Settings" },
];

export function Sidebar(): React.JSX.Element {
  const { dispatch, state } = useAppStore();
  const { resetToConnectedPlayer, setView } = useAppActions();

  return (
    <aside className="sidebar">
      <div className="sidebar-header">
        <button className="brand" type="button" aria-label="Retour au joueur connecte" onClick={resetToConnectedPlayer}>
          <span className="brand-mark">A</span>
          <span className="brand-name">Arkan</span>
        </button>
        <button
          className="icon-button sidebar-toggle"
          type="button"
          aria-label={state.sidebarCollapsed ? "Expand sidebar" : "Reduce sidebar"}
          onClick={() => dispatch({ collapsed: !state.sidebarCollapsed, type: "sidebarChanged" })}
        >
          <span className="collapse-icon" aria-hidden="true" />
        </button>
      </div>
      <nav className="nav" aria-label="Primary navigation">
        {navItems.map((item) => (
          <button
            className={`nav-item ${item.view === state.view ? "active" : ""}`.trim()}
            type="button"
            key={item.label}
            aria-current={item.view === state.view ? "page" : undefined}
            disabled={!item.view}
            onClick={() => {
              if (item.view) {
                void setView(item.view);
              }
            }}
          >
            <span className="nav-icon" aria-hidden="true">{item.icon}</span>
            <span className="nav-label">{item.label}</span>
          </button>
        ))}
      </nav>
    </aside>
  );
}
