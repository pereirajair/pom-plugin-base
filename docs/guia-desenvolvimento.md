# Development guide

Use this plugin as a small reference when starting a POM plugin:

1. Change the package name, plugin code, and exported C entry point.
2. Update the menu, routes, screens, icon image, and documentation paths in
   `ui/manifest.json`.
3. Keep locale keys aligned in `i18n/en.json` and `i18n/pt-BR.json`.
4. Keep the host boundary in `src/lib.rs` and run the verification commands
   from the root README before packaging.

The menu may use its existing named icon as a fallback. A plugin can also set
`icon_image` to a PNG asset for hosts that support plugin-owned menu images.
