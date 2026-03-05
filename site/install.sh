#!/bin/sh
set -eu

REPO="aetherwing-io/fcp-regex"
INSTALL_DIR="${FCP_REGEX_INSTALL_DIR:-$HOME/.local/bin}"

main() {
  os=$(uname -s)
  arch=$(uname -m)

  case "$os" in
    Darwin) os_target="apple-darwin" ;;
    Linux)  os_target="unknown-linux-gnu" ;;
    *)      err "Unsupported OS: $os (fcp-regex supports macOS and Linux)" ;;
  esac

  case "$arch" in
    arm64|aarch64) arch_target="aarch64" ;;
    x86_64)        arch_target="x86_64" ;;
    *)             err "Unsupported architecture: $arch" ;;
  esac

  target="${arch_target}-${os_target}"

  printf "  detecting platform... %s\n" "$target"

  # Fetch latest release tag
  tag=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | head -1 | cut -d'"' -f4)

  if [ -z "$tag" ]; then
    err "Could not determine latest release"
  fi

  printf "  latest release... %s\n" "$tag"

  url="https://github.com/${REPO}/releases/download/${tag}/fcp-regex-${tag}-${target}.tar.gz"

  # Download and extract
  tmpdir=$(mktemp -d)
  trap 'rm -rf "$tmpdir"' EXIT

  printf "  downloading... "
  curl -fsSL "$url" -o "$tmpdir/fcp-regex.tar.gz"
  printf "ok\n"

  tar xzf "$tmpdir/fcp-regex.tar.gz" -C "$tmpdir"

  # Install
  mkdir -p "$INSTALL_DIR"
  mv "$tmpdir/fcp-regex" "$INSTALL_DIR/fcp-regex"
  chmod +x "$INSTALL_DIR/fcp-regex"

  printf "  installed to %s/fcp-regex\n" "$INSTALL_DIR"

  # Verify
  if "$INSTALL_DIR/fcp-regex" --version >/dev/null 2>&1; then
    version=$("$INSTALL_DIR/fcp-regex" --version 2>&1)
    printf "\n  + %s\n" "$version"
  else
    printf "\n  + binary installed (could not verify version)\n"
  fi

  # PATH check
  case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
      printf "\n  note: add %s to your PATH:\n" "$INSTALL_DIR"
      printf "    export PATH=\"%s:\$PATH\"\n" "$INSTALL_DIR"
      ;;
  esac

  printf "\n  MCP config (add to .claude/settings.json or claude_desktop_config.json):\n"
  printf "    \"fcp-regex\": { \"command\": \"fcp-regex\" }\n"
  printf "\n"
}

err() {
  printf "  ! %s\n" "$1" >&2
  exit 1
}

main
