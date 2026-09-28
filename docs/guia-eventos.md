# Events and notifications guide

The POM and its plugins talk through one protocol, `pom-plugin-events/v1`.
Every event uses the same JSON envelope, whatever channel carries it:

```json
{
  "protocol": "pom-plugin-events/v1",
  "id": "5f0c...",
  "type": "theme.changed",
  "target": "*",
  "source": "pom",
  "at": "2026-09-28T13:00:00.000Z",
  "payload": { "theme": "light" }
}
```

`target` is `*` for every plugin or the `plugin_code` of one plugin.

## Events the POM sends

| Type | Payload | When |
|---|---|---|
| `locale.changed` | `{locale}` | the interface language changes, and once at start |
| `theme.changed` | `{theme}` (`light` or `dark`) | the interface theme changes, and once at start |
| `preferences.changed` | `{preferences}` | the preferences of this plugin are saved |
| `notification.received` | `{message_id, plugin_code, title, body, level, sender_node_id, sent_at}` | a plugin notice arrives from the network |
| `notification.response` | `{request_id, kind, action, error?}` | answer to a command of this plugin |

## Channels

- **Host SDK** (`__POM_HOST__`, version 2 or later):
  `__POM_HOST__.plugin(code).events.subscribe(type, handler, {replay})`.
  `replay: true` delivers the last `locale.changed` and `theme.changed`
  right away. `__POM_HOST__.context()` returns the current locale and theme.
- **DOM**: the POM fires `CustomEvent("pom:plugin-event", {detail: envelope})`
  on `window`. Use it from code that does not load the SDK, such as a
  third-party web application embedded in a plugin screen.
- **iframes**: an `<iframe data-pom-plugin="<plugin_code>">` of the same origin
  receives the envelope through `postMessage`.
- **Native backend**: the POM calls `query` with
  `{"operation": "host.event", "event": <envelope>}`. A plugin that does not
  know the operation answers an error, which the POM ignores, exactly as with
  `host.configure`. This plugin keeps the last events and serves them on
  `GET /events` through the plugin proxy.

## Commands a plugin sends

| Command | Payload | Result |
|---|---|---|
| `notification.notify` | `{title, body?, level?, scope?}` | `delivered` or `failed` |
| `notification.confirm` | `{title, body?, acceptLabel?, cancelLabel?, tone?}` | `accepted` or `cancelled` |

`level` is `info`, `success`, `warning` or `error`. `scope: "local"` (default)
shows a notice only in this interface; `scope: "network"` sends it to every
node through the authenticated network chat transport, where it is kept in
memory for 24 hours and shown as a notice. The network chat must be enabled.
`notification.confirm` opens an Accept or Cancel dialog for the current user.

With the SDK, `__POM_HOST__.plugin(code).notifications.notify(...)` and
`.confirm(...)` return a promise with the response. Without it, fire
`CustomEvent("pom:plugin-request", {detail: {protocol, id, type, plugin_code, payload}})`
on `window`, or `postMessage` the same object from a same-origin iframe, and
wait for the `notification.response` event whose `request_id` is your `id`.

`ui/src/host/runtime.ts` wraps both paths in `onPomEvent`, `usePomEvent`,
`usePomContext`, `notify` and `confirm`, falling back to the DOM channel when
the host SDK is older than version 2.
