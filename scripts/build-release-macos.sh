#!/usr/bin/env bash
# The static release of xx for macOS arm64: xx-macos-arm64.tar.gz and its .sha256 in
# $WORK/$VERSION. YubiKey support (libfido2) and what it needs (libcbor, libcrypto) are linked in
# statically, so the program needs only the system's libraries. Run from the root of the
# workspace layout (<root>/mirrorboards-core/mirrorboards-cli/cli-xx and its path dependencies);
# CI runs it in .github/workflows/release.yml, and it works the same on a Mac with Homebrew.
set -euo pipefail

ROOT="${ROOT:-$PWD}"
WORK="${WORK:-${RUNNER_TEMP:-/tmp}/xx-release}"
VERSION="${VERSION:?set VERSION, e.g. v0.2.0}"
mkdir -p "$WORK"

# Homebrew gives libfido2.a, libcrypto.a and cmake; libcbor comes only as a .dylib there.
brew install libfido2 openssl@3 cmake pkg-config
fido2_prefix="$(brew --prefix libfido2)"
openssl_prefix="$(brew --prefix openssl@3)"

# A static libcbor of the version Homebrew's libfido2 is built against (Homebrew ships it only as a
# .dylib): its source and sha256 come from Homebrew's formula, so the two always match.
cbor_formula="$(brew info --json=v2 libcbor)"
cbor_field() {
  printf '%s' "$cbor_formula" | python3 -c "import json, sys; f = json.load(sys.stdin)['formulae'][0]; print($1)"
}
cbor_version="$(cbor_field "f['versions']['stable']")"
cbor_url="$(cbor_field "f['urls']['stable']['url']")"
cbor_sha256="$(cbor_field "f['urls']['stable']['checksum']")"
echo "libcbor $cbor_version from $cbor_url"

mkdir -p "$WORK/deps/dl" "$WORK/deps/src"
curl -sSLf -o "$WORK/deps/dl/libcbor.tar.gz" "$cbor_url"
echo "$cbor_sha256  $WORK/deps/dl/libcbor.tar.gz" | shasum -a 256 -c -
tar xzf "$WORK/deps/dl/libcbor.tar.gz" -C "$WORK/deps/src" --strip-components 1
cmake -S "$WORK/deps/src" -B "$WORK/deps/cbor-build" \
  -DCMAKE_BUILD_TYPE=Release \
  -DBUILD_SHARED_LIBS=OFF \
  -DWITH_EXAMPLES=OFF \
  -DWITH_TESTS=OFF \
  -DCMAKE_OSX_ARCHITECTURES=arm64 \
  -DCMAKE_OSX_DEPLOYMENT_TARGET=15.0 \
  -DCMAKE_INSTALL_PREFIX="$WORK/deps/cbor-install"
cmake --build "$WORK/deps/cbor-build" -j8
cmake --install "$WORK/deps/cbor-build"

# The folder xkeys-ctap's build.rs links libfido2 from (XKEYS_LIBFIDO2_DIR): static archives
# only, since Apple's linker takes a .dylib over a .a in the same folder.
static_dir="$WORK/deps/static"
mkdir -p "$static_dir"
cp "$WORK/deps/cbor-install/lib/libcbor.a" "$static_dir/"
cp "$fido2_prefix/lib/libfido2.a" "$static_dir/"
cp "$openssl_prefix/lib/libcrypto.a" "$static_dir/"

# build.rs adds -lfido2; libfido2's own needs (cbor, crypto, z, IOKit, CoreFoundation) come here.
# Homebrew's archives target this macOS, so the binary says so too.
export XKEYS_LIBFIDO2_DIR="$static_dir"
export MACOSX_DEPLOYMENT_TARGET=15.0
export RUSTFLAGS="-C link-arg=-L$static_dir -C link-arg=-lcbor -C link-arg=-lcrypto -C link-arg=-lz -C link-arg=-framework -C link-arg=IOKit -C link-arg=-framework -C link-arg=CoreFoundation -C strip=debuginfo"
export CARGO_TARGET_DIR="$WORK/target"
(cd "$ROOT/mirrorboards-core/mirrorboards-cli/cli-xx" && cargo build --release --locked)
binary="$CARGO_TARGET_DIR/release/xx"

# Pulumi's Rust language host from the pinned pulumi-rust, which xx mirrorboard deploy puts first
# in Pulumi's PATH (PT-4 of cli-spec xx.typ). Go builds it as the deployment image does.
language_host="$WORK/pulumi-language-rust"
(cd "$ROOT/vendors/pulumi-labs/pulumi-rust/pulumi-language-rust" \
  && CGO_ENABLED=0 GOOS=darwin GOARCH=arm64 go build -trimpath -ldflags "-s -w" -o "$language_host" .)

# Only the system's libraries, for both programs.
for program in "$binary" "$language_host"; do
  otool -L "$program"
  if otool -L "$program" | tail -n +2 | awk '{print $1}' | grep -vE '^(/usr/lib/|/System/Library/)'; then
    echo "$program links a library that is not the system's" >&2
    exit 1
  fi
done

# Stripping breaks the linker's ad-hoc signature; sign it ad hoc again.
out="$WORK/$VERSION"
mkdir -p "$out/pkg"
cp "$binary" "$out/pkg/xx"
strip "$out/pkg/xx"
codesign -s - -f "$out/pkg/xx"
codesign -v "$out/pkg/xx"
"$out/pkg/xx" version
cp "$language_host" "$out/pkg/pulumi-language-rust"
codesign -s - -f "$out/pkg/pulumi-language-rust"
codesign -v "$out/pkg/pulumi-language-rust"
# The Pulumi xx runs (PT-3), as in every archive of a release.
cp "$ROOT/mirrorboards-core/mirrorboards-mirrornet/mirrornet-cli/src/xx/mirrorboard/pulumi.toml" "$out/pkg/"

# xx, the language host and the Pulumi pins at the archive's root, and its sha256 as shasum
# prints it.
cd "$out"
COPYFILE_DISABLE=1 tar -czf xx-macos-arm64.tar.gz -C pkg xx pulumi-language-rust pulumi.toml
shasum -a 256 xx-macos-arm64.tar.gz > xx-macos-arm64.tar.gz.sha256
shasum -a 256 -c xx-macos-arm64.tar.gz.sha256
rm -rf pkg
echo "$out"
