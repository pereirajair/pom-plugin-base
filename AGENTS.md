# Project agent memory

- The host ABI entry point and embedded asset interface live in `src/lib.rs`.
- The host menu (with `children` submenus), route, description and screenshot contract is `ui/manifest.json`; screen exports are in `ui/src/screens/index.tsx`. Screenshots under `docs/screenshots/` are not assets: the POM loads them from GitHub.
- `ui/src/host/runtime.ts` is the only place that touches `__POM_HOST__`; it implements the `pom-plugin-events/v1` helpers with a DOM-channel fallback. `docs/guia-eventos.md` documents the protocol; the native side answers `host.event` in `src/lib.rs`.
- The contract test forbids some unrelated product words anywhere in the repository (`repository_text_avoids_unrelated_product_terms`); keep new copy clear of them.
- `scripts/build-ui.sh` generates ignored files under `ui/dist/` before native packaging.
- The manual release workflow and its optional license-server publication are defined in `.github/workflows/publish-release.yml` and documented in `README.md`. `release/manifest.json` is the release/capability contract and example local preference schema; `ui/manifest.json` is only the `pom-plugin-ui/v1` screen contract. `scripts/package.sh` emits per-platform GitHub manifests from the release contract.
- Use `cargo test` and `cargo fmt --check` for the Rust and static contract checks.

## Maintaining this file

Keep this file for knowledge useful to almost every future agent session in this project. Do not repeat what the codebase already shows; point to the authoritative file or command instead. Prefer rewriting or pruning existing entries over appending new ones. When updating this file, preserve this bar for all agents and keep entries concise.
