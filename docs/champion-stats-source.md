# Champion Stats Source

## Decision

Arkan should build champion statistics from official Riot data instead of scraping U.GG, OP.GG, Porofessor, or similar sites.

The first-party source stack is:

- Riot ACCOUNT-V1 and SUMMONER-V4 for player identity and profile metadata.
- Riot MATCH-V5 for match ids, match details, and timelines.
- Riot Data Dragon for static champion, item, spell, rune, and profile icon metadata.
- Local SQLite for cached raw matches and derived champion aggregates.

## Why not scrape third-party sites

U.GG, OP.GG, Porofessor, and Blitz-like products appear to run their own aggregation pipelines over Riot match data. Their public web pages are presentation surfaces, not stable data contracts for another app. Depending on those pages would make Arkan fragile, harder to test, and harder to keep aligned with Riot policy.

## Riot-backed aggregation model

The aggregation pipeline should collect match ids by PUUID through MATCH-V5, fetch match details and timelines, then normalize participant data by patch, platform, queue, tier, champion, and role.

Champion pages need these derived records:

- role-level winrate and pickrate;
- sample size and collection timestamp;
- recommended skill order from timeline skill-level events;
- best item build from participant items and timeline purchases;
- source metadata: patch, platform, queue, tier, and source name.

## Initial SQLite shape

The local schema now includes:

- `champion_role_stats`: one aggregate row per champion, role, patch, platform, queue, and tier.
- `champion_skill_orders`: ranked skill-order candidates for a stats row.
- `champion_item_builds`: ranked item-build candidates for a stats row.

This lets the UI stay honest: winrate and pickrate filters should remain disabled until rows exist from a real MATCH-V5 aggregation job.

## Implementation Plan

1. Fetch recent match ids for a seed player or a curated sample of players.
2. Fetch match details and timelines.
3. Persist raw match JSON in `matches`.
4. Normalize `player_matches` for profile history.
5. Aggregate champion role stats into `champion_role_stats`.
6. Aggregate skill orders and item builds into their dedicated tables.
7. Enable champion list winrate/pickrate filters once the aggregate table is populated.
8. Make champion tiles clickable and load the champion detail page from static Data Dragon data plus local aggregates.

## Source References

- Riot Developer Portal: League of Legends docs and routing values.
- Riot Data Dragon: static game data and assets.
- Riot Developer API Policy: API key security and approved aggregate/stat use cases.
