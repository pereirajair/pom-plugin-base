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

The manifest sets `plugin_code` to `base`. Its menu entries point to routes declared in the same file, so the host can add both screens without a host-side menu change. The exported C entry point is `pom_base_plugin_v1`.

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

## Start a plugin

1. Update the package name, plugin code, and exported entry point for the new plugin.
2. Add or adjust its menu item, route, screen export, and UI assets in `ui/manifest.json`.
3. Implement plugin behavior behind the versioned ABI in `src/lib.rs`.
4. Add screen components under `ui/src/screens/` and keep the `en` and `pt-BR` catalogs aligned.
5. Run the verification and build commands above before packaging.

The UI bundle is generated under `ui/dist/` and embedded into the native library during the Rust build. Do not commit that generated directory.
