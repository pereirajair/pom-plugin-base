import { useEffect, useState } from "react";
import { getPluginApi, usePluginI18n } from "../host/runtime";

type WorkspaceStatus = "loading" | "unavailable" | "empty" | "ready";

type WorkspaceState = {
  status: WorkspaceStatus;
  projects: string[];
};

const loadingState: WorkspaceState = { status: "loading", projects: [] };

export function normalizeWorkspace(value: unknown): WorkspaceState {
  if (typeof value !== "object" || value === null) {
    return { status: "unavailable", projects: [] };
  }

  const payload = value as { status?: unknown; projects?: unknown };
  const status = payload.status;
  const projects = payload.projects;
  if (
    (status !== "ready" && status !== "empty" && status !== "unavailable") ||
    !Array.isArray(projects) ||
    !projects.every((project): project is string => typeof project === "string")
  ) {
    return { status: "unavailable", projects: [] };
  }

  if (status === "unavailable") {
    return { status, projects: [] };
  }
  if (projects.length === 0) {
    return { status: "empty", projects: [] };
  }
  if (status === "empty") {
    return { status: "unavailable", projects: [] };
  }
  return { status: "ready", projects };
}

function useWorkspace(): WorkspaceState {
  const [state, setState] = useState(loadingState);

  useEffect(() => {
    let active = true;
    getPluginApi<unknown>("projects")
      .then((value) => {
        if (active) setState(normalizeWorkspace(value));
      })
      .catch(() => {
        if (active) setState({ status: "unavailable", projects: [] });
      });
    return () => {
      active = false;
    };
  }, []);

  return state;
}

export function Example() {
  const { t } = usePluginI18n();
  const workspace = useWorkspace();
  const description = t("example.body");

  return (
    <main className="pb-example">
      <section className="pb-workspace-card" aria-live="polite">
        <header className="pb-workspace-heading">
          <span className="pb-overline">{t("example.eyebrow")}</span>
          <h1>{t("example.title")}</h1>
          <p>{description}</p>
        </header>

        {workspace.status === "loading" && (
          <p className="pb-workspace-state" role="status">{t("example.loading")}</p>
        )}
        {workspace.status === "unavailable" && (
          <p className="pb-workspace-state pb-workspace-state-muted" role="status">
            {t("example.unavailable")}
          </p>
        )}
        {workspace.status === "empty" && (
          <p className="pb-workspace-state pb-workspace-state-muted" role="status">
            {t("example.empty")}
          </p>
        )}
        {workspace.status === "ready" && (
          <ul className="pb-project-list" aria-label={description}>
            {workspace.projects.map((project) => (
              <li key={project}>
                <span className="pb-project-marker" aria-hidden="true" />
                <span>{project}</span>
              </li>
            ))}
          </ul>
        )}
      </section>
    </main>
  );
}
