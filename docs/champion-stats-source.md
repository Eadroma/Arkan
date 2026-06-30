# Champion Stats Source ADR

Status: accepted
Last reviewed: 2026-06-30
Ticket: Explorer la source de donnees champions builds et statistiques

## Decision

Arkan should build champion statistics from Riot-provided game data and sampled MATCH-V5 aggregation. We should not scrape U.GG, OP.GG, Porofessor, Blitz, or other presentation sites as a data source.

The source stack is:

- Riot ACCOUNT-V1 and SUMMONER-V4 for player identity and profile metadata.
- Riot LEAGUE-V4 for rank/tier context when aggregating a player's own matches.
- Riot MATCH-V5 for match ids, match details, and timelines.
- Riot Data Dragon for static champion, item, summoner spell, rune, profile icon, and ability assets.
- CommunityDragon only as a secondary static metadata option when Data Dragon does not expose a needed asset cleanly.
- Local SQLite for cached raw match payloads and derived champion aggregates.

## Product Target

The champion detail page should feel similar in information density to U.GG: hero header, patch, role, rank filter context, win rate, pick rate, ban rate when available, match count, recommended runes, summoner spells, matchup cards, skill priority, skill path, and item blocks.

U.GG publicly presents these data groups on champion build pages, including patch, role, tier, win rate, rank, pick rate, ban rate, matches, runes, summoner spells, matchups, skill priority, skill path, and items. Their app page also advertises build import, skill leveling, live stats, post-game analytics, and LP tracking. These are useful UX references, not API contracts.

## Why Third-Party Scraping Is Rejected

Third-party pages are not stable machine contracts. They can change markup, lazy-loaded payloads, naming, filters, anti-abuse controls, or legal boundaries without notice.

Scraping would also make Arkan harder to test and harder to explain to Riot review. Riot's developer terms define game information as data provided through the Riot API and require applications to follow Riot specifications and policy changes. Riot's developer portal also documents rate limits and routing rules; staying on official APIs gives us predictable constraints.

Rejected options:

- Scrape U.GG/OP.GG/Porofessor pages: fast visual parity, but fragile and poor compliance posture.
- Reverse engineer private app APIs: likely brittle, legally risky, and not acceptable for a public repo.
- Ship static copied builds: stale immediately and impossible to personalize by patch/rank/role.
- Depend only on a user's own history: compliant and useful for personal stats, but too sparse for global champion pages.

## Recommended Architecture

Use a two-stage model:

1. Fast personal history from the connected or searched player's recent matches.
2. Champion aggregate stats from a controlled seed set of accounts or imported match samples.

MATCH-V5 does not expose a public endpoint for "last matches by champion". The practical model is seed-based: fetch up to 500 recent matches for selected Riot accounts, cache the raw match payloads, and aggregate every participant in those matches. This gives all champions represented in the sample a shared denominator while staying on official Riot APIs.

The first high-quality seed source should be Riot LEAGUE-V4 top ladders: Challenger, Grandmaster, and Master entries for `RANKED_SOLO_5x5`. Those entries provide encrypted summoner ids, which must be resolved through SUMMONER-V4 before MATCH-V5 can fetch match ids by PUUID. Top-player syncs must stay capped by default because a small development API key cannot safely crawl every top ladder account at full depth.

### Runtime Lookup Flow

1. Resolve Riot ID to PUUID with ACCOUNT-V1.
2. Fetch summoner metadata with SUMMONER-V4.
3. Fetch rank entries with LEAGUE-V4.
4. Fetch match ids with MATCH-V5.
5. Fetch match detail and timeline for each match.
6. Persist raw match JSON in `matches`.
7. Normalize participants into `player_matches`.
8. Aggregate every participant into champion stats tables keyed by patch, platform, queue, tier, champion, and role.
9. Render champion pages from Data Dragon static metadata plus sampled aggregate rows.

### Aggregation Dimensions

Required dimensions:

- `patch`: normalize from `info.gameVersion`, preferably major.minor.
- `platform_id`: `EUW1`, `NA1`, etc.
- `regional_route`: `europe`, `americas`, `asia`, `sea`.
- `queue_id`: start with ranked solo/duo `420`.
- `tier`: start with player's ranked tier, later bucket as `EMERALD_PLUS`, `DIAMOND_PLUS`, etc.
- `champion_id` and `champion_key`.
- `role`: TOP, JUNGLE, MIDDLE, BOTTOM, UTILITY.

Required measures:

- games and wins.
- win rate.
- pick rate within the aggregation slice.
- ban rate only when we have a reliable ban denominator. MATCH-V5 includes team bans, but ban rate needs a full slice denominator across matches, not only picked champion rows.
- skill order candidates from timeline skill-level events.
- summoner spell pair candidates from participant spell ids.
- item build candidates from final participant items first, then timeline purchase paths later.
- matchup stats from lane opponent detection by role/team.

## Current SQLite Fit

The schema already has the foundation:

- `matches`: raw MATCH-V5 payload and game metadata.
- `player_matches`: normalized participant row for a tracked player.
- `champion_role_stats`: aggregate row by champion, role, patch, platform, queue, tier, and sample source.
- `champion_skill_orders`: ranked skill-order candidates.
- `champion_item_builds`: ranked item-build candidates.

Needed additions:

- `champion_spell_pairs` for summoner spell recommendations.
- `champion_rune_pages` once rune ids are extracted from MATCH-V5 participant perks.
- `champion_matchups` for lane matchup cards.
- optional `aggregation_runs` to track source, patch, sample size, freshness, and errors.

## MVP Cut

First implementation should avoid pretending we have U.GG-scale global data.

MVP scope:

- Aggregate from cached MATCH-V5 matches seeded by connected/searched accounts.
- Label stats as "MATCH-V5 sample" in the UI.
- Show sample size prominently.
- Disable or soften global filters when no aggregate rows exist.
- Keep champion page static sections from Data Dragon visible: abilities, role tags, icons, spell descriptions.
- Fill skill priority, spell pairs, and items only when sample size is non-zero.

This gives immediate value without violating policy or inventing placeholder numbers.

## Production Path

After MVP:

1. Add a background aggregation command that processes cached matches incrementally.
2. Add rank/tier enrichment from LEAGUE-V4.
3. Expand the seeded sampling strategy beyond the current displayed profile.
4. Add freshness rules per patch.
5. Add a production Riot API application before any broad crawl.
6. Add dashboards/tests for rate-limit handling and data quality.

## Compliance Rules

- Use only the latest stable Riot API endpoints.
- Respect platform and regional routing.
- Treat Riot API keys as secrets; never commit keys.
- Cache raw data locally to avoid repeated calls.
- Surface Riot disclaimer in README/app before distribution.
- Avoid Brawl or any mode Riot marks as unavailable or disallowed for third-party aggregation.
- Do not imply Riot endorsement.

## Follow-Up Tickets

1. Aggregate champion role stats from cached MATCH-V5 details.
2. Add aggregate read command for champion detail pages.
3. Add `champion_spell_pairs` and `champion_rune_pages`.
4. Replace champion page placeholders with MATCH-V5 sample cards.
5. Add `aggregation_runs` freshness metadata.
6. Add ban-rate support only after validating denominator quality.

## References

- Riot League of Legends docs: https://developer.riotgames.com/docs/lol
- Riot API list: https://developer.riotgames.com/apis
- Riot general policy: https://developer.riotgames.com/policies/general
- Riot API terms: https://developer.riotgames.com/terms
- U.GG champion page UX reference: https://u.gg/lol/champions/singed/build
- U.GG app feature reference: https://u.gg/app
