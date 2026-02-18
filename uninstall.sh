#!/bin/sh
# Jankenoboe uninstaller — removes the jankenoboe binary.
# Usage: curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/uninstall.sh | sh
#   or:  sh uninstall.sh

set -e

BINARY_NAME="jankenoboe"

# Locations where jankenoboe may have been installed
INSTALL_DIR="${JANKENOBOE_INSTALL_DIR:-$HOME/.local/bin}"
CARGO_BIN="$HOME/.cargo/bin"

removed=0

remove_binary() {
  local dir="$1"
  local path="${dir}/${BINARY_NAME}"
  if [ -f "$path" ]; then
    rm "$path"
    echo "✓ Removed ${path}"
    removed=$((removed + 1))
  fi
}

# Remove from default install location (~/.local/bin)
remove_binary "$INSTALL_DIR"

# Remove from cargo bin (~/.cargo/bin) if installed via cargo install
remove_binary "$CARGO_BIN"

if [ "$removed" -eq 0 ]; then
  echo "No jankenoboe binary found in ${INSTALL_DIR} or ${CARGO_BIN}."
  echo ""
  echo "If you installed it elsewhere, remove it manually:"
  echo "  which jankenoboe && rm \$(which jankenoboe)"
  exit 0
fi

echo ""
echo "Uninstall complete."
echo ""
echo "Optional cleanup (your data is preserved by default):"
echo "  • Remove your database:      rm ~/db/datasource.db"
echo "  • Remove JANKENOBOE_DB from your shell profile (~/.zshrc, ~/.bashrc, etc.)"