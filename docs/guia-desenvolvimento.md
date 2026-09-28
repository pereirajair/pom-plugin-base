# Development guide

Use this plugin as the reference when starting a POM plugin:

1. Change the package name, plugin code, and exported C entry point.
2. Update the menu, submenus, routes, screens, icon image, description,
   screenshots and documentation paths in `ui/manifest.json`.
3. Keep locale keys aligned in `i18n/en.json` and `i18n/pt-BR.json`.
4. Keep the host boundary in `src/lib.rs` and run the verification commands
   from the root README before packaging.

## Menus and submenus

A `menu[]` item with `children` becomes a section of the POM sidebar; each
child has `id`, `label` (with `en`), optional `icon` and `roles`, and a `to`
covered by a declared route. Keep `to` on the parent too: hosts without
submenu support show that single destination. This plugin declares the
section `POM - Plugins` with the children `Plugin` and `Projects`.

The menu may use its named icon as a fallback. `icon_image` points to a PNG
asset used by hosts that support plugin-owned menu images.

## Description and screenshots

`description` holds one short text per locale (`en` required). `screenshots`
lists up to eight images as paths relative to the repository root or
`https://` URLs. Relative paths are resolved to
`https://raw.githubusercontent.com/<owner>/<repo>/HEAD/<path>`, so the browser
loads them directly from GitHub; they are not plugin assets and never pass
through the license server. Both fields are optional.

## Events and notifications

See [the events guide](guia-eventos.md) for the `pom-plugin-events/v1`
protocol: events from the POM (language, theme, preferences, notices) and
commands from the plugin (notices to this user or to the network, and Accept
or Cancel questions answered in the plugin screen).
