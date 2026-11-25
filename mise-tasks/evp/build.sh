#!/usr/bin/env bash
#MISE description="Build evp for one Rust target and stage the release archive files"
#USAGE arg "<target>" help="Rust target triple, such as aarch64-apple-darwin"
set -euo pipefail

# The Ghostty commit upstream's release workflow builds libghostty-vt from.
ghostty_commit=6590196661f769dd8f2b3e85d6c98262c4ec5b3b

target="${usage_target:?}"
case "$target" in
  aarch64-apple-darwin) zig_target=aarch64-macos ;;
  *)
    printf 'evp:build: no release build for %s\n' "$target" >&2
    exit 1
    ;;
esac

root="$PWD"
work="$root/target/ghostty"
lib="$root/assets/libghostty"
stage="$root/target/dist/$target"
rm -rf "$work" "$lib" "$stage"

git clone --filter=blob:none --no-checkout https://github.com/ghostty-org/ghostty.git "$work/src"
git -C "$work/src" checkout --quiet "$ghostty_commit"
cd "$work/src"
zig build \
  -Demit-lib-vt \
  -Doptimize=ReleaseFast \
  -Demit-xcframework=false \
  -Dapp-runtime=none \
  -Dtarget="$zig_target" \
  -Dcpu=baseline \
  --prefix "$work/install"
cd "$root"

# build.rs reads the prebuilt library from assets/libghostty.
mkdir -p "$lib/lib" "$lib/include" "$lib/share/pkgconfig"
cp "$work/install/lib/libghostty-vt.a" "$lib/lib/"
cp -R "$work/install/include/ghostty" "$lib/include/"
pc="$lib/share/pkgconfig/libghostty-vt-static.pc"
cp "$work/install/share/pkgconfig/libghostty-vt-static.pc" "$pc"
# The prefix follows the .pc file, as in upstream's release workflow.
sed -i.orig 's|^prefix=.*|prefix=${pcfiledir}/../..|' "$pc"
rm "$pc.orig"

cargo build --locked --release --target "$target" --bin evp

mkdir -p "$stage/examples"
cp "target/$target/release/evp" "$stage/"
cp README.md AGENTS.md PATCHES.md ARCHITECTURE.md LICENSE "$stage/"
cp -R licenses skills "$stage/"
cp examples/quickstart.tape "$stage/examples/"
"$stage/evp" --version
