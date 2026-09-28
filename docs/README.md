# POM - Plugins documentation

This directory contains the guides shipped with POM - Plugins, the tutorial
plugin. The POM shows these files in its documentation view after the plugin
is installed.

- [Development guide](guia-desenvolvimento.md)
- [Shared workspace guide](guia-workspace.md)
- [Events and notifications guide](guia-eventos.md)

The file list is registered by `documentation` in `ui/manifest.json` and the
Markdown files are embedded as plugin assets during the native build. The
images in `screenshots/` are not embedded: the POM loads them straight from
GitHub for the Plugins screen.
