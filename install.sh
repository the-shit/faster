#!/usr/bin/env bash
set -euo pipefail

# Faster installer
# Installs voice interface for Claude Code

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

INSTALL_DIR="${HOME}/.faster"
BIN_DIR="${HOME}/.local/bin"

echo "Installing Faster - Voice interface for Claude Code"
echo ""

# Detect platform
case "$(uname -s)" in
    Darwin*) PLATFORM="macos" ;;
    Linux*)  PLATFORM="linux" ;;
    *)
        echo -e "${RED}Unsupported platform${NC}"
        exit 1
        ;;
esac

# Create directories
mkdir -p "${INSTALL_DIR}"
mkdir -p "${BIN_DIR}"

# Check for Claude Code
if ! command -v claude &> /dev/null; then
    echo -e "${YELLOW}⚠ Claude Code not found${NC}"
    echo "Install from: https://claude.ai/code"
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Platform-specific dependencies
install_macos_deps() {
    echo "Installing macOS dependencies..."

    # Check for Homebrew
    if ! command -v brew &> /dev/null; then
        echo -e "${YELLOW}Homebrew not found. Installing...${NC}"
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi

    # Install sox for audio recording
    if ! command -v sox &> /dev/null; then
        echo "Installing sox..."
        brew install sox
    fi

    # Optional: whisper-cpp for better STT
    if ! command -v whisper-cpp &> /dev/null; then
        echo -e "${YELLOW}whisper-cpp not found (optional for better STT)${NC}"
        read -p "Install whisper-cpp? (~500MB download) (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            brew install whisper-cpp
            # Download base model
            mkdir -p "${INSTALL_DIR}/models"
            if [[ ! -f "${INSTALL_DIR}/models/ggml-base.en.bin" ]]; then
                echo "Downloading Whisper base model..."
                curl -L "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin" \
                    -o "${INSTALL_DIR}/models/ggml-base.en.bin"
            fi
        fi
    fi
}

install_linux_deps() {
    echo "Installing Linux dependencies..."

    # Detect package manager
    if command -v apt-get &> /dev/null; then
        PKG_MANAGER="apt-get"
        INSTALL_CMD="sudo apt-get install -y"
    elif command -v dnf &> /dev/null; then
        PKG_MANAGER="dnf"
        INSTALL_CMD="sudo dnf install -y"
    else
        echo -e "${RED}No supported package manager found${NC}"
        exit 1
    fi

    # Install sox
    if ! command -v sox &> /dev/null; then
        echo "Installing sox..."
        ${INSTALL_CMD} sox
    fi

    # Install TTS
    if ! command -v espeak-ng &> /dev/null; then
        echo "Installing espeak-ng..."
        ${INSTALL_CMD} espeak-ng
    fi

    # Install whisper-cpp
    if ! command -v whisper-cpp &> /dev/null; then
        echo "Installing whisper-cpp..."
        echo -e "${YELLOW}whisper-cpp requires manual build on Linux${NC}"
        echo "See: https://github.com/ggerganov/whisper.cpp"
    fi
}

# Install dependencies
case "${PLATFORM}" in
    macos)
        install_macos_deps
        ;;
    linux)
        install_linux_deps
        ;;
esac

# Copy main script
echo "Installing faster command..."
cp faster "${BIN_DIR}/faster"
chmod +x "${BIN_DIR}/faster"

# Add to PATH if needed
if [[ ":$PATH:" != *":${BIN_DIR}:"* ]]; then
    echo ""
    echo -e "${YELLOW}Add to your shell profile:${NC}"
    echo "export PATH=\"\${HOME}/.local/bin:\${PATH}\""
    echo ""

    # Try to add automatically
    for rc in "${HOME}/.zshrc" "${HOME}/.bashrc"; do
        if [[ -f "${rc}" ]]; then
            if ! grep -q "/.local/bin" "${rc}"; then
                echo "# Faster CLI" >> "${rc}"
                echo "export PATH=\"\${HOME}/.local/bin:\${PATH}\"" >> "${rc}"
                echo -e "${GREEN}✓${NC} Added to ${rc}"
            fi
        fi
    done
fi

# Create default config
if [[ ! -f "${INSTALL_DIR}/config.sh" ]]; then
    cat > "${INSTALL_DIR}/config.sh" << 'EOF'
# Faster configuration

# TTS voice (macOS voice name)
FASTER_VOICE="Samantha"

# STT engine (native, whisper)
FASTER_STT="native"

# Silence threshold in seconds
FASTER_SILENCE_THRESHOLD=1.5

# Show transcripts
FASTER_SHOW_TRANSCRIPTS=true

# Default Claude model
FASTER_MODEL="sonnet"
EOF
    echo -e "${GREEN}✓${NC} Created config: ${INSTALL_DIR}/config.sh"
fi

echo ""
echo -e "${GREEN}Installation complete!${NC}"
echo ""
echo "Try it:"
echo "  faster --test    # Test installation"
echo "  faster           # Start voice mode"
echo ""
echo "Note: You may need to reload your shell or run:"
echo "  source ~/.zshrc"
