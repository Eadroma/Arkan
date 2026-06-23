import { describe, expect, test } from "bun:test";

import { hasRiotTag, normalizeRiotId, sameRiotId } from "./riotId";

describe("Riot ID helpers", () => {
  test("normalizes whitespace and case", () => {
    expect(normalizeRiotId("  Eadroma#EUW  ")).toBe("eadroma#euw");
  });

  test("compares Riot IDs without being case-sensitive", () => {
    expect(sameRiotId("Eadroma#999", " eadroma#999 ")).toBe(true);
    expect(sameRiotId("Eadroma#999", "Other#999")).toBe(false);
  });

  test("detects tagged Riot IDs", () => {
    expect(hasRiotTag("Aedroma#001")).toBe(true);
    expect(hasRiotTag("Aedroma")).toBe(false);
  });
});
