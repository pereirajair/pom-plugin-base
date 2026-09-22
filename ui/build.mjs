import { build } from "esbuild";
import { createRequire } from "node:module";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";

const require = createRequire(import.meta.url);
const hostModules = {
  react: "React",
  "react-dom": "ReactDOM",
  "react/jsx-runtime": "jsxRuntime",
};

function hostModule(specifier, property) {
  const names = Object.keys(require(specifier)).filter(
    (name) => /^[A-Za-z_$][\w$]*$/.test(name) && name !== "default",
  );
  return [
    `const module = globalThis.__POM_HOST__.${property};`,
    "export default module;",
    ...names.map((name) => `export const ${name} = module.${name};`),
  ].join("\n");
}

const hostPlugin = {
  name: "pom-host",
  setup(builder) {
    builder.onResolve(
      { filter: /^(react|react-dom|react\/jsx-runtime)$/ },
      (args) => ({ path: args.path, namespace: "pom-host" }),
    );
    builder.onLoad({ filter: /.*/, namespace: "pom-host" }, (args) => ({
      contents: hostModule(args.path, hostModules[args.path]),
      loader: "js",
    }));
  },
};

rmSync("dist", { recursive: true, force: true });
mkdirSync("dist", { recursive: true });
await build({
  entryPoints: { screens: "src/screens/index.tsx" },
  outdir: "dist",
  bundle: true,
  format: "esm",
  target: "es2022",
  minify: true,
  jsx: "automatic",
  loader: { ".css": "empty" },
  define: { "process.env.NODE_ENV": '"production"' },
  plugins: [hostPlugin],
  logLevel: "info",
});
writeFileSync("dist/plugin.css", readFileSync("src/plugin.css", "utf8"));
console.log("dist/plugin.css", readFileSync("dist/plugin.css", "utf8").length, "bytes");
