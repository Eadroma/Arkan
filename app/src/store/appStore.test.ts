import { describe, expect, test } from "bun:test";

import { appReducer, initialState, type AppState } from "./appStore";
import type { PlayerProfile } from "../domain/league";

describe("appReducer connected player updates", () => {
  test("refreshes the displayed champion pool when the detected account matches the displayed profile", () => {
    const displayedProfile = playerProfile("Eadroma#999");
    const detectedProfile = playerProfile("eadroma#999");
    const state: AppState = {
      ...initialState,
      championPool: [],
      connectedChampionPool: [],
      connectedPlayerProfile: playerProfile("Recherche automatique..."),
      playerProfile: displayedProfile,
    };

    const nextState = appReducer(state, {
      pool: [{ championId: 29, championLevel: 5, championPoints: 28_723 }],
      profile: detectedProfile,
      type: "connectedPlayerChanged",
    });

    expect(nextState.playerProfile.displayName).toBe("eadroma#999");
    expect(nextState.championPool).toEqual([{ championId: 29, championLevel: 5, championPoints: 28_723 }]);
    expect(nextState.connectedChampionPool).toEqual([{ championId: 29, championLevel: 5, championPoints: 28_723 }]);
  });

  test("keeps a searched profile visible when polling detects another connected account", () => {
    const state: AppState = {
      ...initialState,
      championPool: [{ championId: 1, championLevel: 4, championPoints: 12_000 }],
      connectedChampionPool: [{ championId: 29, championLevel: 5, championPoints: 28_723 }],
      connectedPlayerProfile: playerProfile("Eadroma#999"),
      playerProfile: playerProfile("AnnieMain#EUW"),
    };

    const nextState = appReducer(state, {
      pool: [{ championId: 67, championLevel: 4, championPoints: 17_456 }],
      profile: playerProfile("OtherAccount#EUW"),
      type: "connectedPlayerChanged",
    });

    expect(nextState.playerProfile.displayName).toBe("AnnieMain#EUW");
    expect(nextState.championPool).toEqual([{ championId: 1, championLevel: 4, championPoints: 12_000 }]);
    expect(nextState.connectedPlayerProfile.displayName).toBe("OtherAccount#EUW");
    expect(nextState.connectedChampionPool).toEqual([{ championId: 67, championLevel: 4, championPoints: 17_456 }]);
  });
});

function playerProfile(displayName: string): PlayerProfile {
  return {
    championMasteries: [],
    displayName,
    kicker: "Test",
    level: "--",
    region: "EUW1",
    status: "Detected",
  };
}
