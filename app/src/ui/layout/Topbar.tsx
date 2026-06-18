import { useAppActions } from "../../application/useAppActions";
import { useAppStore } from "../../store/appStore";
import { Button } from "../components/Button";

export function Topbar(): React.JSX.Element {
  const { dispatch, state } = useAppStore();
  const { searchRiotAccount } = useAppActions();
  const isProfileView = state.view === "profile";
  const title = state.view === "profile" ? "Profil joueur" : "Champions";

  return (
    <header className="topbar">
      <div className="view-title">
        <span className="eyebrow">Workspace</span>
        <h1>{title}</h1>
      </div>
      {isProfileView ? (
        <form
          className="searchbar"
          onSubmit={(event) => {
            event.preventDefault();
            void searchRiotAccount();
          }}
        >
          <div className="search-field">
            <span className="search-glyph" aria-hidden="true" />
            <input
              aria-label="Riot ID"
              placeholder="GameName#TAG"
              value={state.search.input}
              onChange={(event) =>
                dispatch({ search: { input: event.currentTarget.value }, type: "searchChanged" })
              }
            />
          </div>
          <select
            aria-label="Region"
            value={state.search.region}
            onChange={(event) =>
              dispatch({ search: { region: event.currentTarget.value }, type: "searchChanged" })
            }
          >
            <option>EUW1</option>
            <option>NA1</option>
            <option>KR</option>
          </select>
          <Button disabled={state.search.isPending} type="submit">
            {state.search.isPending ? "Search..." : "Search"}
          </Button>
        </form>
      ) : null}
    </header>
  );
}
