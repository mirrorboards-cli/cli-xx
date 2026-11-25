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

# A static libcbor 0.14.0, the version Homebrew's libfido2 is built against.
mkdir -p "$WORK/deps/dl"
curl -sSLf -o "$WORK/deps/dl/libcbor.tar.gz" \
  https://github.com/PJK/libcbor/archive/refs/tags/v0.14.0.tar.gz
echo "a8c1516e741562cf95aa4479c64916c3d4d2623e24fdc35e414e2320e7300aae  $WORK/deps/dl/libcbor.tar.gz" \
  | shasum -a 256 -c -
tar xzf "$WORK/deps/dl/libcbor.tar.gz" -C "$WORK/deps/dl"
cmake -S "$WORK/deps/dl/libcbor-0.14.0" -B "$WORK/deps/cbor-build" \
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

# Only the system's libraries.
otool -L "$binary"
if otool -L "$binary" | tail -n +2 | awk '{print $1}' | grep -vE '^(/usr/lib/|/System/Library/)'; then
  echo "xx links a library that is not the system's" >&2
  exit 1
fi

# Stripping breaks the linker's ad-hoc signature; sign it ad hoc again.
out="$WORK/$VERSION"
mkdir -p "$out/pkg"
cp "$binary" "$out/pkg/xx"
strip "$out/pkg/xx"
codesign -s - -f "$out/pkg/xx"
codesign -v "$out/pkg/xx"
"$out/pkg/xx" version

# One file, xx, at the archive's root, and its sha256 as shasum prints it.
cd "$out"
COPYFILE_DISABLE=1 tar -czf xx-macos-arm64.tar.gz -C pkg xx
shasum -a 256 xx-macos-arm64.tar.gz > xx-macos-arm64.tar.gz.sha256
shasum -a 256 -c xx-macos-arm64.tar.gz.sha256
rm -rf pkg
echo "$out"
