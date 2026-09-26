# Shared workspace guide

The POM owns the user's shared project folder. It sends the optional
`workspace_root` field in the top-level `host.configure` query:

```json
{
  "operation": "host.configure",
  "workspace_root": "/path/to/projects"
}
```

The base plugin does not select a folder. Its Example screen reads only the
immediate child directories under that root through the plugin proxy. Hidden
entries, including plugin state directories, are excluded. If the root is
missing, inaccessible, or empty, the screen shows a safe status instead of
failing the plugin.
