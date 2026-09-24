# Project agent memory

- The host ABI entry point and embedded asset interface live in `src/lib.rs`.
- The host menu and route contract is `ui/manifest.json`; screen exports are in `ui/src/screens/index.tsx`.
- `scripts/build-ui.sh` generates ignored files under `ui/dist/` before native packaging.
- The manual release workflow and its optional license-server publication are defined in `.github/workflows/publish-release.yml` and documented in `README.md`. `release/manifest.json` is the release/capability contract and example local preference schema; `ui/manifest.json` is only the `pom-plugin-ui/v1` screen contract. `scripts/package.sh` emits per-platform GitHub manifests from the release contract.
- Use `cargo test` and `cargo fmt --check` for the Rust and static contract checks.

## Maintaining this file

Keep this file for knowledge useful to almost every future agent session in this project. Do not repeat what the codebase already shows; point to the authoritative file or command instead. Prefer rewriting or pruning existing entries over appending new ones. When updating this file, preserve this bar for all agents and keep entries concise.
