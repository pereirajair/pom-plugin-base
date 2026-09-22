import { usePluginI18n } from "../host/runtime";

export function BasePlugin() {
  const { t } = usePluginI18n();

  return (
    <main className="pb-page">
      <header className="pb-header">
        <div className="pb-mark" aria-hidden="true"><span>P</span></div>
        <div>
          <p className="pb-eyebrow">{t("eyebrow")}</p>
          <h1>{t("title")}</h1>
        </div>
        <span className="pb-version">{t("version")}</span>
      </header>

      <section className="pb-intro">
        <div className="pb-intro-copy">
          <span className="pb-overline">{t("introLabel")}</span>
          <h2>{t("introTitle")}</h2>
          <p>{t("introBody")}</p>
        </div>
        <div className="pb-intro-symbol" aria-hidden="true">
          <span className="pb-symbol-ring pb-symbol-ring-back" />
          <span className="pb-symbol-ring pb-symbol-ring-front" />
          <span className="pb-symbol-core">P</span>
        </div>
      </section>

      <div className="pb-content-grid">
        <section className="pb-card pb-structure">
          <div className="pb-card-heading">
            <div>
              <span className="pb-overline">01 / {t("structureLabel")}</span>
              <h2>{t("structureTitle")}</h2>
            </div>
            <span className="pb-file-count">05</span>
          </div>
          <p className="pb-body-copy">{t("structureBody")}</p>
          <pre className="pb-file-tree"><code>{t("fileTree")}</code></pre>
        </section>

        <div className="pb-concepts">
          <section className="pb-card pb-concept-card">
            <span className="pb-concept-icon pb-icon-abi" aria-hidden="true">01</span>
            <div>
              <h2>{t("abiTitle")}</h2>
              <p>{t("abiBody")}</p>
            </div>
          </section>
          <section className="pb-card pb-concept-card">
            <span className="pb-concept-icon pb-icon-manifest" aria-hidden="true">02</span>
            <div>
              <h2>{t("manifestTitle")}</h2>
              <p>{t("manifestBody")}</p>
            </div>
          </section>
          <section className="pb-card pb-concept-card">
            <span className="pb-concept-icon pb-icon-ui" aria-hidden="true">03</span>
            <div>
              <h2>{t("uiTitle")}</h2>
              <p>{t("uiBody")}</p>
            </div>
          </section>
        </div>
      </div>

      <section className="pb-card pb-flow">
        <div className="pb-flow-heading">
          <div>
            <span className="pb-overline">02 / {t("flowLabel")}</span>
            <h2>{t("flowTitle")}</h2>
          </div>
          <p>{t("flowBody")}</p>
        </div>
        <ol className="pb-steps">
          <li><span>01</span><p>{t("stepOne")}</p></li>
          <li><span>02</span><p>{t("stepTwo")}</p></li>
          <li><span>03</span><p>{t("stepThree")}</p></li>
        </ol>
      </section>
    </main>
  );
}
