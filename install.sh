#!/bin/sh
# Installs xx, the command line of Mirrorboards, from its newest release on GitHub:
#
#   curl -fsSL https://raw.githubusercontent.com/mirrorboards-cli/cli-xx/main/install.sh | sh
#
# It goes to /usr/local/bin when this user may write there, else to ~/.local/bin; XX_INSTALL_DIR
# chooses another folder. The archive is checked against the release's sha256 before anything is
# installed. Afterwards `xx update` keeps it up to date.
set -eu

repository="mirrorboards-cli/cli-xx"

case "$(uname -s)" in
  Darwin) os=macos ;;
  Linux) os=linux ;;
  *) echo "xx: no release for $(uname -s)" >&2; exit 1 ;;
esac
case "$(uname -m)" in
  arm64 | aarch64) arch=arm64 ;;
  x86_64) arch=x64 ;;
  *) echo "xx: no release for the processor $(uname -m)" >&2; exit 1 ;;
esac

asset="xx-$os-$arch.tar.gz"
releases="https://github.com/$repository/releases/latest/download"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

if ! curl -fsSL "$releases/$asset" -o "$work/$asset"; then
  echo "xx: the newest release has no $asset" >&2
  exit 1
fi
curl -fsSL "$releases/$asset.sha256" -o "$work/$asset.sha256"

expected="$(cut -d ' ' -f 1 < "$work/$asset.sha256")"
if command -v shasum > /dev/null 2>&1; then
  actual="$(shasum -a 256 "$work/$asset" | cut -d ' ' -f 1)"
else
  actual="$(sha256sum "$work/$asset" | cut -d ' ' -f 1)"
fi
if [ "$expected" != "$actual" ]; then
  echo "xx: the download's sha256 is $actual, but the release says $expected; nothing was installed" >&2
  exit 1
fi

tar -xzf "$work/$asset" -C "$work"

folder="${XX_INSTALL_DIR:-}"
if [ -z "$folder" ]; then
  if [ -d /usr/local/bin ] && [ -w /usr/local/bin ]; then
    folder=/usr/local/bin
  else
    folder="$HOME/.local/bin"
  fi
fi
mkdir -p "$folder"
install -m 0755 "$work/xx" "$folder/xx"

echo "xx installed in $folder/xx: $("$folder/xx" version)"

# Plain git with rooms (SG-7 of cli-spec xx.typ): git starts git-remote-xgit for xgit:// remotes,
# a link to xx beside it, and the large-file filter goes into the global git config.
if command -v git > /dev/null; then
  "$folder/xx" git install
fi

# Tab completion for the shell the person logs in with. The shell asks xx at every Tab, so the
# completion knows this computer's vaults and identities; reinstalling keeps it as it is.
login_shell="$(basename "${SHELL:-}")"
case "$login_shell" in
  fish)
    completions="${XDG_CONFIG_HOME:-$HOME/.config}/fish/completions"
    mkdir -p "$completions"
    "$folder/xx" completion fish > "$completions/xx.fish"
    echo "Tab completion for fish: $completions/xx.fish"
    ;;
  zsh)
    line='command -v xx > /dev/null && source <(xx completion zsh)'
    if ! grep -qsF "$line" "$HOME/.zshrc"; then
      printf '\n# Tab completion of xx\n%s\n' "$line" >> "$HOME/.zshrc"
    fi
    echo "Tab completion for zsh: ~/.zshrc (new shells)"
    ;;
  *)
    echo "Tab completion: xx completion fish|zsh|bash prints the script for your shell"
    ;;
esac
case ":$PATH:" in
  *":$folder:"*) ;;
  *) echo "add $folder to your PATH, e.g. in ~/.zshrc: export PATH=\"$folder:\$PATH\"" ;;
esac
