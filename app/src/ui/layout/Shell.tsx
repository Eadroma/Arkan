import type { ReactNode } from "react";

import { useAppStore } from "../../store/appStore";
import { Sidebar } from "./Sidebar";
import { Topbar } from "./Topbar";

export function Shell({ children }: { children: ReactNode }): React.JSX.Element {
  const { state } = useAppStore();

  return (
    <main className="shell" data-sidebar={state.sidebarCollapsed ? "collapsed" : "expanded"}>
      <Sidebar />
      <section className="content">
        <Topbar />
        {children}
      </section>
    </main>
  );
}
