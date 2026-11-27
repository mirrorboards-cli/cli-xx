#!/usr/bin/env bash
# The release of xx for Linux x64: xx-linux-x64.tar.gz and its .sha256 in $WORK/$VERSION. It is
# built on Debian bookworm, so it runs where glibc is 2.36 or newer. YubiKey support links libfido2
# from the system, which needs it installed (Debian, Ubuntu: apt install libfido2-1). Run from the
# root of the workspace layout (<root>/mirrorboards-core/mirrorboards-cli/cli-xx and its path
# dependencies); CI runs it as root in the container rust:<version>-bookworm
# (.github/workflows/release.yml), with Go on PATH.
set -euo pipefail

ROOT="${ROOT:-$PWD}"
WORK="${WORK:-${RUNNER_TEMP:-/tmp}/xx-release}"
VERSION="${VERSION:?set VERSION, e.g. v0.2.0}"
mkdir -p "$WORK"

# libfido2 for xkeys-ctap's build.rs (pkg-config), protoc for the provider's Pulumi protocol.
apt-get update
apt-get install -y --no-install-recommends libfido2-dev pkg-config protobuf-compiler

export RUSTFLAGS="-C strip=debuginfo"
export CARGO_TARGET_DIR="$WORK/target"
(cd "$ROOT/mirrorboards-core/mirrorboards-cli/cli-xx" && cargo build --release --locked)
binary="$CARGO_TARGET_DIR/release/xx"

# Pulumi's Rust language host from the pinned pulumi-rust, which xx mirrorboard deploy puts first
# in Pulumi's PATH (PT-4 of cli-spec xx.typ). Without cgo it needs no library at all. No VCS stamp:
# in the container git refuses the checkouts, which belong to another user than root.
language_host="$WORK/pulumi-language-rust"
(cd "$ROOT/vendors/pulumi-labs/pulumi-rust/pulumi-language-rust" \
  && CGO_ENABLED=0 GOOS=linux GOARCH=amd64 \
     go build -trimpath -buildvcs=false -ldflags "-s -w" -o "$language_host" .)

# xx links only glibc's libraries and libfido2 with what libfido2 needs, all of them in the system.
ldd "$binary"
if ldd "$binary" | grep -q 'not found'; then
  echo "$binary needs a library this system does not have" >&2
  exit 1
fi
allowed='^(linux-vdso\.so|/lib64/ld-linux-x86-64\.so|lib(c|m|gcc_s|pthread|dl|rt)\.so|lib(fido2|cbor|crypto|ssl|udev|z)\.so)'
if ldd "$binary" | awk '{print $1}' | grep -vE "$allowed"; then
  echo "$binary links a library other than glibc's and libfido2's" >&2
  exit 1
fi
# ldd fails on a static program.
if ldd "$language_host" > /dev/null 2>&1; then
  echo "$language_host is not static" >&2
  exit 1
fi

out="$WORK/$VERSION"
mkdir -p "$out/pkg"
cp "$binary" "$out/pkg/xx"
strip "$out/pkg/xx"
"$out/pkg/xx" version
cp "$language_host" "$out/pkg/pulumi-language-rust"
# The Pulumi xx runs (PT-3), for the deployment image, which installs it ahead (PT-5).
cp "$ROOT/mirrorboards-core/mirrorboards-mirrornet/mirrornet-cli/src/xx/mirrorboard/pulumi.toml" "$out/pkg/"

# xx, the language host and the Pulumi pins at the archive's root, and its sha256 as shasum
# prints it.
cd "$out"
tar -czf xx-linux-x64.tar.gz -C pkg xx pulumi-language-rust pulumi.toml
sha256sum xx-linux-x64.tar.gz > xx-linux-x64.tar.gz.sha256
sha256sum -c xx-linux-x64.tar.gz.sha256
rm -rf pkg
echo "$out"
