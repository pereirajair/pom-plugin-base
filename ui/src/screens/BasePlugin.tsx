import { useI18n } from "../host/runtime";

export function BasePlugin() {
  const { t } = useI18n();

  return (
    <main className="pb-page">
      <header className="pb-header">
        <div className="pb-mark" aria-hidden="true"><span>P</span></div>
        <div>
          <p className="pb-eyebrow">{t("base.eyebrow")}</p>
          <h1>{t("base.title")}</h1>
        </div>
        <span className="pb-version">{t("base.version")}</span>
      </header>

      <section className="pb-intro">
        <div className="pb-intro-copy">
          <span className="pb-overline">{t("base.introLabel")}</span>
          <h2>{t("base.introTitle")}</h2>
          <p>{t("base.introBody")}</p>
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
              <span className="pb-overline">01 / {t("base.structureLabel")}</span>
              <h2>{t("base.structureTitle")}</h2>
            </div>
            <span className="pb-file-count">05</span>
          </div>
          <p className="pb-body-copy">{t("base.structureBody")}</p>
          <pre className="pb-file-tree"><code>{t("base.fileTree")}</code></pre>
        </section>

        <div className="pb-concepts">
          <section className="pb-card pb-concept-card">
            <span className="pb-concept-icon pb-icon-abi" aria-hidden="true">01</span>
            <div>
              <h2>{t("base.abiTitle")}</h2>
              <p>{t("base.abiBody")}</p>
            </div>
          </section>
          <section className="pb-card pb-concept-card">
            <span className="pb-concept-icon pb-icon-manifest" aria-hidden="true">02</span>
            <div>
              <h2>{t("base.manifestTitle")}</h2>
              <p>{t("base.manifestBody")}</p>
            </div>
          </section>
          <section className="pb-card pb-concept-card">
            <span className="pb-concept-icon pb-icon-ui" aria-hidden="true">03</span>
            <div>
              <h2>{t("base.uiTitle")}</h2>
              <p>{t("base.uiBody")}</p>
            </div>
          </section>
        </div>
      </div>

      <section className="pb-card pb-flow">
        <div className="pb-flow-heading">
          <div>
            <span className="pb-overline">02 / {t("base.flowLabel")}</span>
            <h2>{t("base.flowTitle")}</h2>
          </div>
          <p>{t("base.flowBody")}</p>
        </div>
        <ol className="pb-steps">
          <li><span>01</span><p>{t("base.stepOne")}</p></li>
          <li><span>02</span><p>{t("base.stepTwo")}</p></li>
          <li><span>03</span><p>{t("base.stepThree")}</p></li>
        </ol>
      </section>
    </main>
  );
}
