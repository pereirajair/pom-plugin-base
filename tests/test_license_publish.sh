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
{"plugin_code":"base","version":"0.1.0","platform":"linux-x86_64","plugin_abi":1,"sha256":"$sha256","size":$size}
JSON

cat > "$tmp_dir/bin/curl" <<'FAKE_CURL'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$@" >> "$POM_TEST_CURL_LOG"
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
assert_contains "$output" 'release_id=base-v0.1.0-linux-x86_64'
assert_contains "$output" 'status=published'
curl_args="$(cat "$curl_log")"
assert_contains "$curl_args" 'plugin=base'
assert_contains "$curl_args" 'version=0.1.0'
assert_contains "$curl_args" 'os=linux'
assert_contains "$curl_args" 'arch=x86_64'
assert_contains "$curl_args" 'plugin_abi=1'
assert_contains "$curl_args" 'feature_set='
assert_contains "$curl_args" 'internal/releases/upload'

cat > "$metadata" <<JSON
{"plugin_code":"base","version":"0.1.0","platform":"linux-x86_64","plugin_abi":1,"sha256":"0000000000000000000000000000000000000000000000000000000000000000","size":$size}
JSON
: > "$curl_log"
if PATH="$tmp_dir/bin:$PATH" POM_TEST_CURL_LOG="$curl_log" POM_RELEASE_TOKEN='test-token' \
  "$root/scripts/publish-to-license-server.sh" \
    --version v0.1.0 --platform linux-x86_64 --artifact "$artifact" --metadata "$metadata" 2>"$tmp_dir/error.txt"; then
  fail 'checksum mismatch should fail'
fi
assert_contains "$(cat "$tmp_dir/error.txt")" 'sha256 mismatch'
[[ ! -s "$curl_log" ]] || fail 'invalid artifact was uploaded'

printf 'test_license_publish: ok\n'
