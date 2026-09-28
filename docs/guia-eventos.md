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
| `deployment.started` | `{kind, deploy_id, job_id, model, nodes, deployment_name?, reused?, version?}` | a deployment (`kind: "deploy"`) or redeployment (`"redeploy"`) starts |
| `model.serving` | `{model, deploy_ids, node_ids}` | a model starts being served by the network (it appears in `/v1/models`) |
| `model.stopped` | `{model}` | a model stops being served |

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
  `host.configure`. The node sends `preferences.changed`,
  `deployment.started`, `model.serving` and `model.stopped` this way, so the
  backend learns about them even with no screen open. This plugin keeps the
  last events and serves them on `GET /events` through the plugin proxy.
  The POM interface reads the same node events from
  `GET /api/ui/plugins/events?after=<seq>` and repeats them on the browser
  channels.

## Commands a plugin sends

| Command | Payload | Result |
|---|---|---|
| `notification.notify` | `{title, body?, level?, scope?}` | `delivered` or `failed` |
| `notification.confirm` | `{title, body?, acceptLabel?, cancelLabel?, tone?, display?}` | `accepted` or `cancelled` |

`level` is `info`, `success`, `warning` or `error`. `scope: "local"` (default)
shows a notice only in this interface; `scope: "network"` sends it to every
node through the authenticated network chat transport, where it is kept in
memory for 24 hours and shown as a notice. The network chat must be enabled.
`notification.confirm` puts an Accept or Reject entry in the POM notification
bell of the current user, the same way a file transfer offer waits there;
`display: "dialog"` opens a modal dialog instead.

With the SDK, `__POM_HOST__.plugin(code).notifications.notify(...)` and
`.confirm(...)` return a promise with the response. Without it, fire
`CustomEvent("pom:plugin-request", {detail: {protocol, id, type, plugin_code, payload}})`
on `window`, or `postMessage` the same object from a same-origin iframe, and
wait for the `notification.response` event whose `request_id` is your `id`.

`ui/src/host/runtime.ts` wraps both paths in `onPomEvent`, `usePomEvent`,
`usePomContext`, `notify` and `confirm`, falling back to the DOM channel when
the host SDK is older than version 2.
