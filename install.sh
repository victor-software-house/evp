#!/bin/sh
# Install a checksummed fork binary plus its documentation and skill.
set -eu
REPO=victor-software-house/evp
TAG=${EVP_VERSION:-evp-0.19.0-vsh.1}
BIN=${EVP_INSTALL_DIR:-"$HOME/.local/bin"}
case "$TAG" in *[!a-zA-Z0-9._-]*|'') printf '%s\n' 'Invalid EVP_VERSION' >&2; exit 1;; esac
if [ "$(uname -s)-$(uname -m)" != Darwin-arm64 ]; then
    printf '%s\n' 'This fork release supports Apple Silicon macOS only.' >&2
    exit 1
fi
for tool in curl tar shasum; do
    command -v "$tool" >/dev/null 2>&1 || { printf 'Required tool missing: %s\n' "$tool" >&2; exit 1; }
done
ARCHIVE="$TAG-aarch64-apple-darwin.tar.gz"
URL="https://github.com/$REPO/releases/download/$TAG"
TEMP=$(mktemp -d)
trap 'rm -rf "$TEMP"' EXIT HUP INT TERM
curl -fLsS "$URL/$ARCHIVE" -o "$TEMP/$ARCHIVE"
curl -fLsS "$URL/$ARCHIVE.sha256" -o "$TEMP/$ARCHIVE.sha256"
(cd "$TEMP" && shasum -a 256 -c "$ARCHIVE.sha256")
ROOT="$TEMP/root"
mkdir "$ROOT"
tar -xzf "$TEMP/$ARCHIVE" -C "$ROOT"
test -f "$ROOT/evp"
test -f "$ROOT/skills/evp/SKILL.md"
mkdir -p "$BIN"
BIN=$(cd "$BIN" && pwd)
SHARE="$BIN/../share/evp"
mkdir -p "$SHARE"
install -m 755 "$ROOT/evp" "$BIN/evp"
cp "$ROOT/README.md" "$ROOT/AGENTS.md" "$ROOT/PATCHES.md" "$ROOT/ARCHITECTURE.md" "$ROOT/LICENSE" "$SHARE/"
cp -R "$ROOT/skills" "$ROOT/examples" "$ROOT/licenses" "$SHARE/"
"$BIN/evp" --version
printf '\nBinary: %s/evp\nDocs and skill: %s\nAdd the binary directory to PATH if needed.\n' "$BIN" "$SHARE"
