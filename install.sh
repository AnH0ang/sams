#!/bin/bash

# Default variables
REPO="AnH0ang/sams"
VERSION="latest"
INSTALL_DIR="/usr/local/bin"

# Function to display help message
usage() {
  echo "Usage: $0 [-v version] [-h] [--uninstall]"
  echo "  -v, --version   Specify the version to install (default: latest)"
  echo "  -h, --help      Show this help message"
  echo "  --uninstall     Uninstall sams"
  exit 0
}

# Function to uninstall sams
uninstall() {
  echo "Uninstalling sams..."
  sudo rm -f $INSTALL_DIR/sams || {
    echo "Failed to remove sams from $INSTALL_DIR"
    exit 1
  }
  echo "sams uninstalled successfully."
  exit 0
}

# Parse command-line arguments
while [[ "$#" -gt 0 ]]; do
  case "$1" in
    -v|--version)
      VERSION="$2"
      shift 2
      ;;
    -h|--help)
      usage
      ;;
    --uninstall)
      uninstall
      ;;
    *)
      echo "Unknown option: $1"
      usage
      ;;
  esac
done

# Detect the operating system
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
if [[ "$OS" == "linux" ]]; then
  ASSET_NAME="sams-x86_64-unknown-linux-gnu.tar.gz"
elif [[ "$OS" == "darwin" ]]; then
  ASSET_NAME="sams-x86_64-apple-darwin.tar.gz"
else
  echo "Unsupported operating system: $OS"
  exit 1
fi

# Get the release URL
if [[ "$VERSION" == "latest" ]]; then
  RELEASE_URL="https://api.github.com/repos/$REPO/releases/latest"
else
  RELEASE_URL="https://api.github.com/repos/$REPO/releases/tags/$VERSION"
fi

# Fetch the download URL for the asset
ASSET_URL=$(curl -s $RELEASE_URL | grep -o "https://.*/$ASSET_NAME" | head -n 1)
if [[ -z "$ASSET_URL" ]]; then
  echo "Failed to find asset $ASSET_NAME for version $VERSION"
  exit 1
fi

# Download the asset
echo "Downloading $ASSET_NAME..."
curl -L -o $ASSET_NAME $ASSET_URL || {
  echo "Failed to download $ASSET_NAME"
  exit 1
}

# Extract the binary
echo "Installing sams..."
tar -xzf $ASSET_NAME || {
  echo "Failed to extract $ASSET_NAME"
  exit 1
}

# Make the binary executable
chmod +x sams || {
  echo "Failed to make sams executable"
  exit 1
}

# Remove macOS quarantine attribute (if on macOS)
if [[ "$OS" == "darwin" ]]; then
  xattr -d com.apple.quarantine sams || {
    echo "The com.apple.quarantine attribute could not be removed"
  }
fi

# Move the binary to the installation directory
echo "Moving sams to $INSTALL_DIR..."
sudo mv sams $INSTALL_DIR/sams || {
  echo "Failed to move sams to $INSTALL_DIR"
  exit 1
}

# Clean up
rm $ASSET_NAME

echo "sams installed successfully to $INSTALL_DIR/sams"
