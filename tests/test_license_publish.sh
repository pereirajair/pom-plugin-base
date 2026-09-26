#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
tmp_dir="$root/target/license-publish-test-$$"
trap 'rm -rf "$tmp_dir"' EXIT

fail() { printf 'FAIL: %s\n' "$1" >&2; exit 1; }
assert_contains() {
  local haystack="$1" needle="$2"
  [[ "$haystack" == *"$needle"* ]] || fail "missing expected field: $needle"
}

mkdir -p "$tmp_dir/bin" "$tmp_dir/dist"
artifact="$tmp_dir/dist/pom-plugin-base-linux-x86_64.so"
metadata="$tmp_dir/dist/pom-plugin-base-linux-x86_64.metadata.json"
printf '%s' 'sample plugin bytes' > "$artifact"
sha256="$(shasum -a 256 "$artifact" | awk '{print $1}')"
size="$(wc -c < "$artifact" | tr -d '[:space:]')"
cat > "$metadata" <<JSON
{"plugin_code":"base","version":"0.1.0","platform":"linux-x86_64","plugin_abi":1,"sha256":"$sha256","size":$size,"preferences":{"features":[{"key":"example_toggle","label":"Example toggle","default":false}],"storage_directory":{"label":"Example storage","default":null}}}
JSON

cat > "$tmp_dir/bin/curl" <<'FAKE_CURL'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$@" >> "$POM_TEST_CURL_LOG"
response_file=""
fail_on_http=false
while (($# > 0)); do
  case "$1" in
    --output) response_file="$2"; shift 2 ;;
    --fail) fail_on_http=true; shift ;;
    *) shift ;;
  esac
done
status="${POM_TEST_HTTP_STATUS:-200}"
if [[ -n "$response_file" && "$status" != 200 ]]; then
  printf '%s' 'plugin code is not registered' > "$response_file"
fi
if [[ "$fail_on_http" == true && "$status" =~ ^[45][0-9][0-9]$ ]]; then
  printf 'curl: (22) The requested URL returned error: %s\n' "$status" >&2
  exit 22
fi
printf '%s' "$status"
FAKE_CURL
chmod +x "$tmp_dir/bin/curl"

curl_log="$tmp_dir/curl.log"
dry_output="$(
  PATH="$tmp_dir/bin:$PATH" \
  "$root/scripts/publish-to-license-server.sh" \
    --version v0.1.0 --platform linux-x86_64 --artifact "$artifact" --metadata "$metadata" --dry-run
)"
assert_contains "$dry_output" 'status=dry-run'
[[ ! -s "$curl_log" ]] || fail 'dry run attempted an upload'

output="$(
  PATH="$tmp_dir/bin:$PATH" \
  POM_TEST_CURL_LOG="$curl_log" \
  POM_RELEASE_API='https://license.test' \
  POM_RELEASE_TOKEN='test-token' \
  "$root/scripts/publish-to-license-server.sh" \
    --version v0.1.0 --platform linux-x86_64 --artifact "$artifact" --metadata "$metadata"
)"
assert_contains "$output" 'release_id=base-0.1.0-linux-x86_64'
assert_contains "$output" 'status=published'
curl_args="$(cat "$curl_log")"
assert_contains "$curl_args" 'plugin=base'
assert_contains "$curl_args" 'version=0.1.0'
assert_contains "$curl_args" 'os=linux'
assert_contains "$curl_args" 'arch=x86_64'
assert_contains "$curl_args" 'plugin_abi=1'
assert_contains "$curl_args" 'feature_set=base.core'
assert_contains "$curl_args" 'internal/releases/upload'
# Regression: /internal/releases/upload rejects every field it does not know and
# answers an opaque 400, so the typed preferences schema must stay out of the
# upload even though the publisher validates it locally.
[[ "$curl_args" != *preferences=* ]] || fail 'upload carried a preferences field the license server rejects'

: > "$curl_log"
if PATH="$tmp_dir/bin:$PATH" POM_TEST_CURL_LOG="$curl_log" POM_TEST_HTTP_STATUS=400 \
  POM_RELEASE_API='https://license.test' POM_RELEASE_TOKEN='test-token' \
  "$root/scripts/publish-to-license-server.sh" \
    --version v0.1.0 --platform linux-x86_64 --artifact "$artifact" --metadata "$metadata" \
    >"$tmp_dir/http-output.txt" 2>"$tmp_dir/http-error.txt"; then
  fail 'HTTP 400 should fail the publication'
fi
assert_contains "$(cat "$tmp_dir/http-error.txt")" 'HTTP 400'
assert_contains "$(cat "$tmp_dir/http-error.txt")" 'plugin code is not registered'

cat > "$metadata" <<JSON
{"plugin_code":"base","version":"0.1.0","platform":"linux-x86_64","plugin_abi":1,"sha256":"0000000000000000000000000000000000000000000000000000000000000000","size":$size,"preferences":{"features":[{"key":"example_toggle","label":"Example toggle","default":false}],"storage_directory":{"label":"Example storage","default":null}}}
JSON
: > "$curl_log"
if PATH="$tmp_dir/bin:$PATH" POM_TEST_CURL_LOG="$curl_log" POM_RELEASE_TOKEN='test-token' \
  "$root/scripts/publish-to-license-server.sh" \
    --version v0.1.0 --platform linux-x86_64 --artifact "$artifact" --metadata "$metadata" 2>"$tmp_dir/error.txt"; then
  fail 'checksum mismatch should fail'
fi
assert_contains "$(cat "$tmp_dir/error.txt")" 'sha256 mismatch'
[[ ! -s "$curl_log" ]] || fail 'invalid artifact was uploaded'

renamed_root="$tmp_dir/renamed"
mkdir -p "$renamed_root/scripts" "$renamed_root/ui" "$renamed_root/release"
cp "$root/scripts/package.sh" "$renamed_root/scripts/package.sh"
cp "$root/scripts/publish-to-license-server.sh" "$renamed_root/scripts/publish-to-license-server.sh"
sed 's/"plugin_code": "base"/"plugin_code": "my_plugin"/' "$root/ui/manifest.json" > "$renamed_root/ui/manifest.json"
# package.sh reads the release contract too, and its plugin code and namespaced
# feature set must follow the renamed plugin.
jq '.plugin_code = "my_plugin" | .feature_set = ["my_plugin.core"]' "$root/release/manifest.json" > "$renamed_root/release/manifest.json"
cat > "$renamed_root/scripts/build.sh" <<'FAKE_BUILD'
#!/usr/bin/env bash
set -euo pipefail
output="$PWD/dist-release"
while (($# > 0)); do
  case "$1" in
    --platform) platform="$2"; shift 2 ;;
    --output) output="$2"; shift 2 ;;
    *) shift ;;
  esac
done
mkdir -p "$output"
artifact="$output/pom-plugin-base-${platform}.so"
printf '%s' 'renamed plugin bytes' > "$artifact"
sha256="$(shasum -a 256 "$artifact" | awk '{print $1}')"
size="$(wc -c < "$artifact" | tr -d '[:space:]')"
printf 'artifact=%s\nsha256=%s\nsize=%s\nplatform=%s\ntarget=x86_64-unknown-linux-gnu\n' \
  "$artifact" "$sha256" "$size" "$platform"
FAKE_BUILD
chmod +x "$renamed_root/scripts/build.sh"

renamed_output="$tmp_dir/renamed-dist"
renamed_package_output="$("$renamed_root/scripts/package.sh" --platform linux-x86_64 --version 0.0.3 --output "$renamed_output")"
renamed_artifact="$(printf '%s\n' "$renamed_package_output" | sed -n 's/^artifact=//p')"
renamed_metadata="$(printf '%s\n' "$renamed_package_output" | sed -n 's/^metadata=//p')"
[[ "$(jq -r '.plugin_code' "$renamed_metadata")" == my_plugin ]] || fail 'package did not derive plugin code from manifest'

: > "$curl_log"
renamed_publish_output="$(
  PATH="$tmp_dir/bin:$PATH" \
  POM_TEST_CURL_LOG="$curl_log" \
  POM_RELEASE_API='https://license.test' \
  POM_RELEASE_TOKEN='test-token' \
  "$renamed_root/scripts/publish-to-license-server.sh" \
    --version v0.0.3 --platform linux-x86_64 --artifact "$renamed_artifact" --metadata "$renamed_metadata"
)"
assert_contains "$renamed_publish_output" 'release_id=my_plugin-0.0.3-linux-x86_64'
renamed_curl_args="$(cat "$curl_log")"
assert_contains "$renamed_curl_args" 'plugin=my_plugin'
assert_contains "$renamed_curl_args" 'feature_set=my_plugin.core'
[[ "$renamed_curl_args" != *preferences=* ]] || fail 'renamed upload carried a preferences field the license server rejects'

printf 'test_license_publish: ok\n'
