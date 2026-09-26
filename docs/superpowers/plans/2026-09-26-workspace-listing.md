# Shared Workspace Listing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Demonstrate the POM shared workspace contract by listing immediate, non-hidden project directories in the Example screen.

**Architecture:** The native plugin accepts the POM's top-level `workspace_root` field on `host.configure`, retains it only in process memory, and serves a loopback `ui.upstream` endpoint through the POM plugin proxy. The endpoint reads only the configured root and returns explicit `ready`, `empty`, or `unavailable` states. The Example screen consumes that authenticated proxy endpoint through `__POM_HOST__.api.get` and renders loading, unavailable, empty, or project-list states.

**Tech Stack:** Rust 2021 cdylib, `serde_json`, standard-library TCP server and filesystem APIs, React 18 hooks, esbuild, JSON locale catalogs.

**Spec:** Firstmate launch brief and `README.md` shared workspace contract section.

## Global Constraints

- Consume only the optional top-level `workspace_root` from POM `host.configure`.
- Never present a workspace picker or derive a workspace root locally.
- Read only immediate child directories under the configured root.
- Exclude hidden child directories and never fail the plugin for absent, inaccessible, or empty roots.
- Do not modify the POM or any other plugin.
- Keep English and Portuguese locale catalogs key-aligned.

## Review Focus

- Malformed or missing `workspace_root` must not escape the configured boundary or panic; test invalid type, null, and missing values.
- Hidden directories must not appear in the project response; test a hidden directory alongside visible directories and files.
- Missing, non-directory, and empty roots must return stable unavailable/empty states; test each without relying on process permissions.
- The UI must not treat malformed proxy data as a project list; test the normalizer's fallback state.
- Plugin lifecycle must remain valid when the upstream socket cannot be used; keep `ui.upstream` errors as query errors rather than create failures.

### Task 1: Native workspace contract and listing endpoint

**Files:**
- Modify: `src/lib.rs`
- Test: `src/lib.rs` unit tests

**Interfaces:**
- Consumes: top-level query objects such as `{"operation":"host.configure","workspace_root":"/path"}`.
- Produces: `{"status":"ok"}` for valid `host.configure`; `{"status":"ready|empty|unavailable","projects":["name"]}` for `GET /projects`; `{"status":"ready","port":number,"token":"..."}` for `ui.upstream`.

- [ ] **Step 1: Write failing Rust tests** for top-level `workspace_root` parsing, visible-directory listing, hidden-directory filtering, and unavailable/empty responses.
- [ ] **Step 2: Run `cargo test`** and verify the new tests fail because the workspace state and query operations do not exist.
- [ ] **Step 3: Implement the minimal native state and loopback server** with `host.configure`, `workspace.projects`, `ui.upstream`, and lifecycle cleanup. Use `file_type().is_dir()` and names beginning with `.` as the filesystem boundary/filter.
- [ ] **Step 4: Run the focused and complete Rust tests** and verify they pass.
- [ ] **Step 5: Commit** with `feat: expose shared workspace projects`.

### Task 2: Example screen and host API client

**Files:**
- Modify: `ui/src/host/runtime.ts`
- Modify: `ui/src/screens/Example.tsx`
- Modify: `ui/src/plugin.css`
- Modify: `i18n/en.json`
- Modify: `i18n/pt-BR.json`
- Test: `tests/contract.rs`

**Interfaces:**
- Consumes: the native proxy response from Task 1 and `__POM_HOST__.api.get`.
- Produces: a typed `getPluginApi` helper, a normalized project view model, and an Example screen with loading, unavailable, empty, and ready states.

- [ ] **Step 1: Add failing contract assertions** for the workspace API path, Example state labels, and locale key alignment.
- [ ] **Step 2: Run `cargo test --test contract`** and verify the new assertions fail against the Lorem ipsum screen and current runtime.
- [ ] **Step 3: Implement the typed runtime API helper, response normalizer, Example UI, styles, and both catalogs.** Map failed calls and malformed responses to `unavailable`.
- [ ] **Step 4: Run the contract tests and `scripts/build-ui.sh`** and verify the embedded bundle builds successfully.
- [ ] **Step 5: Commit** with `feat: list shared workspace projects in example screen`.

### Task 3: Document the common contract

**Files:**
- Modify: `README.md`

**Interfaces:**
- Consumes: the final native query and UI proxy behavior from Tasks 1 and 2.
- Produces: a concise maintainer-facing explanation of the host-owned workspace root and failure states.

- [ ] **Step 1: Add the workspace contract section** with the exact top-level `host.configure` shape, proxy behavior, hidden-directory rule, and no-picker rule.
- [ ] **Step 2: Run `cargo test`, `cargo fmt --check`, and `scripts/build-ui.sh`** to verify docs did not disturb the package.
- [ ] **Step 3: Commit** with `docs: describe shared workspace contract`.
