export function normalizeRiotId(value: string): string {
  return value.trim().toLocaleLowerCase("en-US");
}

export function sameRiotId(first: string, second: string): boolean {
  return normalizeRiotId(first) === normalizeRiotId(second);
}

export function hasRiotTag(value: string): boolean {
  return value.trim().includes("#");
}
