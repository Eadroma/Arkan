# Notion Pending Updates

Notion updates are pending because the connector hit the current usage limit.

## 2026-06-17 - CI and Tests

Update the ticket board with:

- Ticket: `Ajouter une CI GitHub Actions`
- Status: `Done`
- Branch: `codex/ci-and-tests`
- Commit: `ea75cd6`
- Commit URL: `https://github.com/Eadroma/Arkan/commit/ea75cd6`
- Notes: Added Windows GitHub Actions CI with `cargo fmt --all --check`, `cargo test -p arkan-core`, and `cargo check --manifest-path src-tauri/Cargo.toml`. Added integration tests for Riot ID, LCU lockfile, regional routing, and ACCOUNT-V1 URL contracts. Merged and pushed to `main`.

## 2026-06-17 - Riot ACCOUNT-V1 Client

Update the ticket board with:

- Ticket: `Créer RiotClient avec ACCOUNT-V1`
- Status: `Done`
- Branch: `codex/riot-account-client`
- Commit: `93619cf`
- Commit URL: `https://github.com/Eadroma/Arkan/commit/93619cf`
- Notes: Added testable `RiotApiClient`, ACCOUNT-V1 URL construction, Riot account response model, API key validation, and Tauri command `resolve_riot_account`. Local tests passed and CI on `main` passed. App launched after the task for manual testing.

## 2026-06-17 - SQLite Initial Migrations

Update the ticket board after this branch is merged:

- Ticket: `Créer migrations SQLite initiales`
- Status: `Done`
- Branch: `codex/sqlite-initial-migrations`
- Commit: `e02fdd1`
- Commit URL: `https://github.com/Eadroma/Arkan/commit/e02fdd1`
- Notes: Added SQLite schema migration v1 for players, ranks, matches, and player_matches. Added in-memory migration tests, idempotency tests, and player upsert/read tests. App launched after the task for manual testing.

## 2026-06-17 - Connected Profile UI Cleanup

Update the ticket board with:

- Ticket: `Nettoyer l'UI du profil connecte`
- Status: `Done`
- Branch: `codex/profile-ui-cleanup`
- Commit: `f43e678`
- Commit URL: `https://github.com/Eadroma/Arkan/commit/f43e678`
- Notes: Removed the redundant `Client League connecte / Profil local pret a synchroniser` badge from the hero, removed the technical LCU port from the visible UI, replaced it with region, kept the player icon and detected profile display. Local tests/check/build passed, app was relaunched for manual testing, branch merged and pushed to `main`. CI on `main` passed.
