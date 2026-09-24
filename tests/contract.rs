use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn text(path: &str) -> String {
    fs::read_to_string(root().join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
}

fn json(path: &str) -> Value {
    serde_json::from_str(&text(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
}

fn contains_term(haystack: &[u8], pattern: &[u8]) -> bool {
    haystack
        .windows(pattern.len())
        .enumerate()
        .any(|(start, window)| {
            if !window.eq_ignore_ascii_case(pattern) {
                return false;
            }
            let is_word = |byte: u8| byte.is_ascii_alphanumeric() || byte >= 128;
            let before_is_word = start > 0 && is_word(haystack[start - 1]);
            let end = start + pattern.len();
            let after_is_word = end < haystack.len() && is_word(haystack[end]);
            !before_is_word && !after_is_word
        })
}

#[test]
fn manifest_registers_both_full_bleed_screens() {
    let manifest = json("ui/manifest.json");
    assert_eq!(manifest["schema"], "pom-plugin-ui/v1");
    assert_eq!(manifest["plugin_code"], "base");

    let menu = manifest["menu"].as_array().expect("menu array");
    assert_eq!(menu.len(), 2);
    let base_menu = menu
        .iter()
        .find(|entry| entry["id"] == "base")
        .expect("base menu entry");
    assert_eq!(base_menu["to"], "/plugin-de-base");
    assert_eq!(base_menu["label"]["en"], "Base Plugin");
    assert_eq!(base_menu["label"]["pt-BR"], "Plugin de Base");
    let example_menu = menu
        .iter()
        .find(|entry| entry["id"] == "example")
        .expect("example menu entry");
    assert_eq!(example_menu["to"], "/example");
    assert_eq!(example_menu["label"]["en"], "Example");
    assert_eq!(example_menu["label"]["pt-BR"], "Exemplo");

    let routes = manifest["routes"].as_array().expect("routes array");
    assert_eq!(routes.len(), 2);
    for (path, screen) in [("/plugin-de-base", "base"), ("/example", "example")] {
        let route = routes
            .iter()
            .find(|route| route["path"] == path)
            .expect("route");
        assert_eq!(route["screen"], screen);
        assert_eq!(route["full_bleed"], true);
        if path == "/plugin-de-base" {
            assert_eq!(route["roles"], serde_json::json!(["admin"]));
        }
        let descriptor = &manifest["screens"][screen];
        assert_eq!(descriptor["module"], "ui/screens.js");
        assert_eq!(descriptor["export"], screen);
        assert!(descriptor["styles"]
            .as_array()
            .unwrap()
            .iter()
            .any(|style| style == "ui/plugin.css"));
    }
}

#[test]
fn catalogs_assets_and_screen_text_keys_are_complete() {
    let manifest = json("ui/manifest.json");
    assert_eq!(manifest["i18n"]["en"], "i18n/en.json");
    assert_eq!(manifest["i18n"]["pt-BR"], "i18n/pt-BR.json");

    let assets: BTreeSet<_> = manifest["assets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|asset| asset.as_str().unwrap())
        .collect();
    assert_eq!(
        assets,
        BTreeSet::from([
            "ui/screens.js",
            "ui/plugin.css",
            "i18n/en.json",
            "i18n/pt-BR.json",
        ])
    );
    assert!(root().join("ui/src/screens/index.tsx").is_file());
    assert!(root().join("ui/src/plugin.css").is_file());
    let screen_index = text("ui/src/screens/index.tsx");
    assert!(screen_index.contains("BasePlugin as base"));
    assert!(screen_index.contains("Example as example"));

    let en = json("i18n/en.json");
    let pt = json("i18n/pt-BR.json");
    assert_eq!(
        en.as_object().unwrap().keys().collect::<Vec<_>>(),
        pt.as_object().unwrap().keys().collect::<Vec<_>>()
    );
    let mut used = BTreeSet::new();
    for path in [
        "ui/src/screens/BasePlugin.tsx",
        "ui/src/screens/Example.tsx",
    ] {
        let source = text(path);
        for rest in source.split("t(\"").skip(1) {
            let key = rest.split_once('\"').expect("translation key").0;
            used.insert(key.to_owned());
        }
    }
    let catalog_keys: BTreeSet<_> = en.as_object().unwrap().keys().cloned().collect();
    assert_eq!(used, catalog_keys);
    assert_eq!(en["example.body"], "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.");
    assert_eq!(pt["example.body"], en["example.body"]);

    let example = text("ui/src/screens/Example.tsx");
    assert_eq!(example.matches("t(\"example.body\")").count(), 1);
    assert_eq!(example.matches("<p").count(), 1);
    assert!(!example.contains("<h1") && !example.contains("<h2"));
}

#[test]
fn i18n_namespace_comes_from_manifest_plugin_code() {
    let manifest = json("ui/manifest.json");
    let plugin_code = manifest["plugin_code"].as_str().expect("plugin code");
    let en = json("i18n/en.json");
    let pt = json("i18n/pt-BR.json");
    let english_keys = en.as_object().expect("English catalog");
    let portuguese_keys = pt.as_object().expect("Portuguese catalog");

    assert!(english_keys.contains_key("title"));
    assert!(english_keys.contains_key("example.body"));
    assert!(!english_keys.keys().any(|key| key.starts_with("base.")));
    assert_eq!(
        english_keys.keys().collect::<Vec<_>>(),
        portuguese_keys.keys().collect::<Vec<_>>()
    );

    let runtime = text("ui/src/host/runtime.ts");
    assert!(runtime.contains("${pluginCode}.${key}"));
    let build = text("ui/build.mjs");
    assert!(build.contains("manifest.plugin_code"));
    assert!(build.contains("namespaceLocaleCatalog"));

    let renamed_code = "my_plugin";
    let derived_keys: BTreeSet<_> = english_keys
        .keys()
        .map(|key| format!("{renamed_code}.{key}"))
        .collect();
    assert!(derived_keys.contains("my_plugin.title"));
    assert!(derived_keys.contains("my_plugin.example.body"));

    let generated = root().join("ui/dist/i18n/en.json");
    if generated.exists() {
        let generated: Value = serde_json::from_slice(&fs::read(generated).unwrap()).unwrap();
        assert!(generated.get(format!("{plugin_code}.title")).is_some());
        assert!(generated
            .get(format!("{plugin_code}.example.body"))
            .is_some());
    }
}

#[test]
fn plugin_styles_do_not_write_global_rules_and_get_plugin_scoped_names() {
    let css = text("ui/src/plugin.css");
    assert!(!css.contains(":root"));
    let build = text("ui/build.mjs");
    assert!(build.contains("namespacePluginCode"));
    assert!(build.contains("pluginCode"));
}

#[test]
fn release_manifest_provides_typed_default_preference_examples() {
    let manifest = json("release/manifest.json");
    assert_eq!(manifest["schema"], 1);
    assert_eq!(manifest["plugin_code"], "base");
    assert_eq!(manifest["feature_set"], serde_json::json!(["base.core"]));

    let preferences = &manifest["preferences"];
    let features = preferences["features"]
        .as_array()
        .expect("boolean preferences");
    assert_eq!(features.len(), 1);
    assert_eq!(features[0]["key"], "example_feature_enabled");
    assert_eq!(features[0]["default"], false);
    assert_eq!(
        preferences["storage_directory"]["label"],
        "Plugin data directory"
    );
    assert!(preferences["storage_directory"]["default"].is_null());

    let package = text("scripts/package.sh");
    assert!(package.contains("pom-plugin-${platform}.json"));
    assert!(package.contains(".preferences"));
    let publisher = text("scripts/publish-to-license-server.sh");
    assert!(publisher.contains("-F \"preferences=${preferences}\""));
}

#[test]
fn generic_build_and_project_files_are_present() {
    let cargo = text("Cargo.toml");
    assert!(cargo.contains("name = \"pom-plugin-base\""));
    assert!(cargo.contains("crate-type = [\"cdylib\"]"));
    let ui_package = json("ui/package.json");
    assert_eq!(ui_package["scripts"]["build"], "node build.mjs");
    let ui_build = text("ui/build.mjs");
    assert!(ui_build.contains("src/screens/index.tsx"));
    assert!(ui_build.contains("dist/plugin.css"));
    assert!(root().join("README.md").is_file());
    assert!(root().join("AGENTS.md").is_file());
    for path in [
        "build.rs",
        "scripts/build.sh",
        "scripts/build-ui.sh",
        "scripts/ci-plan.sh",
        "scripts/package.sh",
        "scripts/publish-to-license-server.sh",
        "release/manifest.json",
        ".github/workflows/publish-release.yml",
    ] {
        assert!(root().join(path).is_file(), "missing {path}");
    }
}

#[test]
fn repository_text_avoids_unrelated_product_terms() {
    let forbidden: &[&[u8]] = &[
        &[101, 110, 116, 101, 114, 112, 114, 105, 115, 101],
        &[108, 111, 103, 115],
        &[115, 116, 97, 116, 105, 115, 116, 105, 99, 115],
        &[114, 101, 113, 117, 101, 115, 116, 115],
        &[97, 117, 100, 105, 116],
        &[100, 97, 116, 97, 98, 97, 115, 101],
        &[109, 101, 116, 114, 105, 99, 115],
        &[115, 111, 108, 100],
        &[101, 115, 116, 97, 116, 195, 173, 115, 116, 105, 99, 97, 115],
        &[
            114, 101, 113, 117, 105, 115, 105, 195, 167, 195, 181, 101, 115,
        ],
        &[97, 117, 100, 105, 116, 111, 114, 105, 97],
        &[
            98, 97, 110, 99, 111, 32, 100, 101, 32, 100, 97, 100, 111, 115,
        ],
        &[109, 195, 169, 116, 114, 105, 99, 97, 115],
        &[118, 101, 110, 100, 105, 100, 111, 115],
    ];
    let mut pending = vec![root()];
    while let Some(path) = pending.pop() {
        let name = path
            .file_name()
            .and_then(|part| part.to_str())
            .unwrap_or_default();
        if [".git", "target", "node_modules", "dist", "dist-release"].contains(&name) {
            continue;
        }
        if path.is_dir() {
            pending.extend(
                fs::read_dir(&path)
                    .unwrap()
                    .flatten()
                    .map(|entry| entry.path()),
            );
            continue;
        }
        let Ok(contents) = fs::read(&path) else {
            continue;
        };
        let mut haystack = path.to_string_lossy().as_bytes().to_vec();
        haystack.push(b'\n');
        haystack.extend(contents);
        for pattern in forbidden {
            assert!(
                !contains_term(&haystack, pattern),
                "unrelated product term found in {}",
                path.display()
            );
        }
    }
}

#[test]
fn example_screen_is_a_minimal_lorem_template() {
    let source = text("ui/src/screens/Example.tsx");
    assert!(source.contains("export function Example"));
    assert!(source.contains("usePluginI18n"));
    assert!(source.contains("t(\"example.body\")"));
    assert_eq!(source.matches("<main").count(), 1);
    assert_eq!(source.matches("<p").count(), 1);
    assert!(!source.contains("<button") && !source.contains("<input"));
}
