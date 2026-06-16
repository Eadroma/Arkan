# Arkan

Arkan is a Rust-first desktop companion for League of Legends.

The current goal is to build a careful MVP before expanding into live-client or overlay features:

- search a player by Riot ID;
- resolve PUUID and summoner data through Riot APIs;
- cache data locally;
- display ranked state, recent matches, and champion trends;
- keep Riot policy constraints visible in the product and codebase.

## Current Status

The repository starts with a testable Rust core crate and a Tauri scaffold.

```txt
crates/arkan-core   Shared domain logic, parsing, routing, tests
src-tauri           Tauri shell scaffold
app                 Static frontend scaffold
docs                Product and technical docs
```

## Local Checks

```powershell
cargo test -p arkan-core
```

If the MSVC linker is not available yet, use:

```powershell
cargo check -p arkan-core
```

## Local Configuration

Runtime configuration is read from environment variables during early development:

```txt
RIOT_API_KEY=RGAPI-your-development-key
ARKAN_DEFAULT_PLATFORM=EUW1
ARKAN_DEFAULT_LANGUAGE=fr_FR
```

The Tauri command only reports whether a key is configured and masks the value.

## Notion

Project documentation and ticket tracking live in Notion:

- Hub: https://app.notion.com/p/3813f0d3e8bf8139b858df69b6f3ad44
- Tickets: https://app.notion.com/p/21ec57af53224e9bb2b9ab08c75450ab
