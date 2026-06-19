# Arkan

<p align="center">
  <img src="app/public/arkane-icon.svg" alt="Arkan icon" width="128" height="128" />
</p>

[![CI](https://github.com/Eadroma/Arkan/actions/workflows/ci.yml/badge.svg)](https://github.com/Eadroma/Arkan/actions/workflows/ci.yml)

Arkan is a Rust-first desktop companion for League of Legends: a local-first player profile, match history, and progression analysis app inspired by tools like OP.GG and Blitz, built with a careful Riot API integration.

The project starts with a focused MVP before expanding into live-client or overlay features:

- search a player by Riot ID;
- resolve PUUID and summoner data through Riot APIs;
- cache data locally;
- display ranked state, recent matches, and champion trends;
- keep Riot policy constraints visible in the product and codebase.

## Repository Description

Rust/Tauri League of Legends companion app for Riot ID lookup, local match-history cache, champion statistics, and progression insights.

## Visual Identity

Arkan uses an arcane gold-and-crystal identity: deep navy surfaces, antique gold accents, and blue crystal highlights. The application icon is the source asset for the in-app brand mark and desktop bundle icon.

## Current Status

The repository currently contains a testable Rust core crate and a Tauri scaffold.

```txt
crates/arkan-core   Shared domain logic, parsing, routing, tests
src-tauri           Tauri shell scaffold
app                 React/Vite frontend with application, store, and UI layers
docs                Product and technical docs
```

## Local Checks

Frontend checks:

```powershell
bun install
bun run typecheck
bun test
bun run build
```

Local development can also use pnpm when Bun is unavailable:

```powershell
pnpm install
pnpm exec tsc --noEmit
pnpm exec vite build --configLoader runner
```

```powershell
cargo test -p arkan-core
```

If the MSVC linker is not available yet, use:

```powershell
cargo check -p arkan-core
```

For the desktop shell:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
```

The GitHub Actions workflow runs frontend typechecking/tests/build, Rust formatting, core tests, and the Tauri shell check on Windows.

## Local Configuration

Runtime configuration is read from environment variables during early development:

```txt
RIOT_API_KEY=RGAPI-your-development-key
ARKAN_DEFAULT_PLATFORM=EUW1
ARKAN_DEFAULT_LANGUAGE=fr_FR
```

The Tauri command only reports whether a key is configured and masks the value.

## Early Features

- `arkan-core` Riot ID parser with unit tests.
- Riot platform and regional route mapping.
- Local app configuration through environment variables.
- Tauri command scaffold for parser/config access.
- React desktop UI with a typed application layer, reducer-backed store, generic UI components, and SCSS/CSS variable styling.

## Roadmap Snapshot

1. Riot API client with ACCOUNT-V1.
2. SQLite migrations and local cache.
3. Summoner and ranked profile lookup.
4. MATCH-V5 history and match detail normalization.
5. Data Dragon champion/item integration.
6. Player profile and match detail UI.

## Notion

Project documentation and ticket tracking live in Notion:

- Hub: https://app.notion.com/p/3813f0d3e8bf8139b858df69b6f3ad44
- Tickets: https://app.notion.com/p/21ec57af53224e9bb2b9ab08c75450ab
