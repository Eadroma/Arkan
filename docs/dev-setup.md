# Development Setup

## Prerequisites

- Rust toolchain with Cargo.
- Node/npm or another frontend package manager for the Tauri UI.
- A Riot development API key from https://developer.riotgames.com/.

## Environment

Copy `.env.example` to `.env` and set a local development key.

The API key must not be committed. For a public build, do not embed a Riot API key in the distributed binary.

## Checks

Run the core tests:

```powershell
cargo test -p arkan-core
```

## Git Note

If Git reports dubious ownership in this workspace, run a safe-directory configuration from a trusted shell:

```powershell
git config --global --add safe.directory C:/Users/bouko/Documents/Arkan
```

This may be needed before branches and commits can be created from automation.

