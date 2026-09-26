# POM Plugin Base

A small public starter for building plugins for the POM. It shows the host ABI boundary, automatic menu registration through `pom-plugin-ui/v1`, embedded React screens, locale catalogs, and repeatable build and packaging commands.

The included **Plugin de Base** screen describes the scaffold. The **Example** screen contains only Lorem ipsum and is intended as the smallest screen to copy when starting a new view.

## Project layout

```text
Cargo.toml             Rust cdylib and dependencies
src/lib.rs              Versioned plugin ABI and UI asset interface
build.rs                Embeds built UI files and locale catalogs
ui/manifest.json        Menu, route, screen, locale, and asset declarations
ui/src/screens/          React screen components
ui/src/plugin.css        Screen styles
i18n/                    en and pt-BR catalogs
scripts/                 UI, native build, and package commands
```

The UI manifest sets `plugin_code` to `base`. Menu entries point to routes declared in the same file. The exported C entry point is `pom_base_plugin_v1`. Release metadata lives separately in `release/manifest.json`: it declares the namespaced `base.core` capability and example local preferences, one boolean feature toggle and one storage directory. The UI manifest remains the `pom-plugin-ui/v1` contract and does not carry license or release preferences.

Locale source keys stay plugin-neutral; the UI build prefixes them with the current `plugin_code` for the POM translation catalog. The same build scopes CSS class names and matching screen markup to that code. When cloning the scaffold, change `plugin_code` in `ui/manifest.json` and the generated identifiers follow it. The license publisher derives the release plugin name from the same manifest and sends the generic feature identifier `<plugin_code>.core`.

## Build and verify

Requirements: Rust stable, Node.js 20 or newer, and npm.

```sh
cargo test
cargo fmt --check
scripts/build-ui.sh
```

Build the native library and copy a named artifact with its SHA-256 and size:

```sh
scripts/build.sh --platform macos-aarch64
```

Supported platform identifiers are `linux-x86_64`, `macos-aarch64`, and `windows-x86_64`. Package the library with small build metadata:

```sh
scripts/package.sh --platform macos-aarch64 --version 0.1.0
```

Both build commands accept `--output <dir>`. By default, artifacts go to `dist-release/`, which is ignored by Git.

## GitHub release workflow

The manual `Publish plugin release` workflow builds selected platforms, attaches the packages and metadata to GitHub releases, and publishes stable releases to the license server. Prereleases go to GitHub only. Each selected platform can use its own `vX.Y.Z` tag; choose the same tag when publishing assets for one multi-platform release. Each platform package also generates `pom-plugin-<os>-<arch>.json`, the public GitHub update manifest consumed by POM. Its SHA-256 and byte size describe the adjacent native asset; keep both assets attached to the release. The packaging metadata carries the typed `preferences` schema through the license publisher, while the preference values themselves are always stored by POM locally and are not license data.

Before running a stable release, configure the repository secret `POM_RELEASE_TOKEN` with a token accepted by the license server. The workflow uses the server's default endpoint; optionally set the `POM_RELEASE_API` repository secret to override it. The publish script verifies the package metadata, ABI, version, platform, size, and SHA-256 before uploading. The upload protocol requires a non-empty namespaced feature set, so this generic scaffold sends `base.core`. A prerelease is the safe way to exercise the build and GitHub release steps without contacting the license server.

The workflow and its plan/publish helpers are in `.github/workflows/publish-release.yml` and `scripts/`.

## Shared workspace contract

The POM owns the user's shared project folder and passes it to an active plugin
through the optional top-level `workspace_root` field of `host.configure`:

```json
{
  "operation": "host.configure",
  "workspace_root": "/path/to/projects"
}
```

This base plugin does not ask the user to choose a folder and does not derive a
fallback root. It keeps the received value in memory, exposes a loopback
`ui.upstream` endpoint to the POM, and serves `GET /projects` through the
authenticated plugin proxy. The endpoint reads only immediate child
directories, sorts their names, and omits hidden entries such as `.plugin-state`
so plugin state cannot appear as a user project.

The Example screen renders `ready`, `empty`, and `unavailable` responses. A
missing or inaccessible root is unavailable, and a readable root without
projects is empty; neither condition prevents the plugin from loading.

## Start a plugin

1. Update the package name, plugin code, and exported entry point for the new plugin.
2. Add or adjust its menu item, route, screen export, and UI assets in `ui/manifest.json`.
3. Implement plugin behavior behind the versioned ABI in `src/lib.rs`.
4. Add screen components under `ui/src/screens/` and keep the `en` and `pt-BR` catalogs aligned.
5. Run the verification and build commands above before packaging.

The UI bundle is generated under `ui/dist/` and embedded into the native library during the Rust build. Do not commit that generated directory.
