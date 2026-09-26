type I18nHook = () => { t: (key: string) => string };

declare const __POM_PLUGIN_CODE__: string;

type PluginApi = {
  get: <T>(path: string) => Promise<T>;
};

type PomHost = {
  hooks: {
    useI18n: I18nHook;
  };
  api?: PluginApi;
};

export function useI18n(): ReturnType<I18nHook> {
  const host = (globalThis as typeof globalThis & { __POM_HOST__?: PomHost }).__POM_HOST__;
  if (!host) throw new Error("POM host SDK is not installed");
  return host.hooks.useI18n();
}

export function usePluginI18n(): ReturnType<I18nHook> {
  const { t } = useI18n();
  const pluginCode = __POM_PLUGIN_CODE__;
  return { t: (key: string) => t(`${pluginCode}.${key}`) };
}

export function getPluginApi<T>(path: string): Promise<T> {
  const host = (globalThis as typeof globalThis & { __POM_HOST__?: PomHost }).__POM_HOST__;
  if (!host?.api) {
    return Promise.reject(new Error("POM host API is not installed"));
  }
  const normalizedPath = path.replace(/^\/+/, "");
  const endpoint = `/api/ui/plugins/${encodeURIComponent(__POM_PLUGIN_CODE__)}/proxy/${normalizedPath}`;
  return host.api.get<T>(endpoint);
}
