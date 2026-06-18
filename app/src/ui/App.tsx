import { useEffect } from "react";

import { useAppActions } from "../application/useAppActions";
import { useAppStore } from "../store/appStore";
import { ChampionDetailPage } from "./champions/ChampionDetailPage";
import { ChampionCatalogView } from "./champions/ChampionCatalogView";
import { Shell } from "./layout/Shell";
import { ProfileView } from "./profile/ProfileView";

export function App(): React.JSX.Element {
  const { state } = useAppStore();
  const { detectLeagueClient } = useAppActions();

  useEffect(() => {
    void detectLeagueClient();
  }, [detectLeagueClient]);

  return (
    <Shell>
      {state.view === "profile" ? <ProfileView /> : null}
      {state.view === "champions" ? <ChampionCatalogView /> : null}
      {state.view === "champion-detail" && state.selectedChampion ? (
        <ChampionDetailPage champion={state.selectedChampion} />
      ) : null}
    </Shell>
  );
}
