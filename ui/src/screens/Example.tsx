import { usePluginI18n } from "../host/runtime";

export function Example() {
  const { t } = usePluginI18n();
  return (
    <main className="pb-example">
      <p>{t("example.body")}</p>
    </main>
  );
}
