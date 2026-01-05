# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Faster** is a voice interface for Claude Code that enables natural conversational interaction via speech-to-text (STT) and text-to-speech (TTS). Built as a portable, zero-config extension that uses Claude Code's existing authentication.

## Core Architecture

### Design Principles
- **Zero Config**: Uses Claude Code's auth - if you're signed in, voice works
- **Ultra Portable**: Single install script, works on macOS/Linux/Windows
- **Local First**: No additional API keys required - uses free/local STT/TTS
- **Speed First**: Every component optimized for minimum latency
- **Stateful Sessions**: Leverages Claude Code's session management
- **Enforced Brevity**: Voice naturally encourages concise, imperative commands - "do X" not paragraphs

### Technology Stack Decisions

**Distribution Method:**
- Shell script (bash/zsh) that integrates with Claude Code CLI
- Installable as a Claude Code skill or standalone command
- Single `curl | bash` install for end users

**Authentication:**
- **Zero new auth required**: Uses Claude Code's existing session token
- If not logged into Claude Code → prompt user to run `claude login`
- No OPENAI_API_KEY or other credentials needed

**STT (Speech-to-Text):**
- **macOS**: Use native Dictation (free, no API needed)
- **Linux/Windows**: whisper-cpp with base.en model (~140MB download)
- **Fallback**: Online services only if local fails

**TTS (Text-to-Speech):**
- **macOS**: Native `say` command (improved with better voices)
- **Linux**: `espeak-ng` or `festival`
- **Windows**: PowerShell SAPI voices
- **Optional upgrade**: edge-tts (free, natural, no API key)

**Claude Integration:**
- Direct shell calls to `claude` CLI command
- Uses existing session and context
- No separate API integration needed

### Component Architecture

```
┌─────────────┐
│  Audio In   │ (microphone capture via sox/pyaudio)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│     VAD     │ (detect speech start/stop)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│     STT     │ (transcribe to text)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Claude    │ (process request, stream response)
│    Code     │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│     TTS     │ (convert to speech, stream audio)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Audio Out  │ (speaker output)
└─────────────┘
```

### Key Implementation Details

**Async Pipeline:**
- Each component runs in separate thread/async task
- Use queues for inter-component communication
- STT can process while Claude is generating response
- TTS can start speaking while Claude is still generating

**Interruption Handling:**
- Monitor audio input during TTS playback
- Cancel TTS playback if user starts speaking
- Clear response buffer and restart pipeline

**State Management:**
- Maintain conversation history in memory
- Support named sessions for context switching
- Persist important context to disk on interruption

**Latency Targets:**
- STT: <500ms from speech end to text
- Claude: streaming start <1s
- TTS: first audio chunk <300ms
- Total turn latency: <2s for simple queries

## Installation

### One-Line Install
```bash
curl -fsSL https://raw.githubusercontent.com/USER/faster/main/install.sh | bash
```

### What It Does
1. Checks if Claude Code is installed and authenticated
2. Installs platform-specific dependencies (whisper-cpp on Linux, etc.)
3. Adds `faster` command to PATH
4. Optionally installs as a Claude Code skill: `/voice`

### Manual Install
```bash
git clone https://github.com/USER/faster.git
cd faster
./install.sh
```

## Usage

### Basic Voice Mode
```bash
# Start voice conversation
faster

# Or as Claude Code skill
claude /voice
```

### First Run
```bash
$ faster
Checking Claude Code authentication...
❌ Not authenticated with Claude Code

Please run:
  claude login

Then try again: faster
```

### Authenticated Flow
```bash
$ faster
✓ Authenticated as user@email.com
🎤 Listening... (press Ctrl+C to exit)

[User speaks: "what files are in this directory?"]
📝 You said: what files are in this directory?
🤖 Claude: Let me check...

[Claude responds with file listing via voice]

🎤 Listening...
```

### Commands
```bash
# Voice mode with specific Claude model
faster --model opus

# Continuous listening (no "wake word", always on)
faster --continuous

# Debug mode (show all transcripts and timing)
faster --debug

# Test installation
faster --test

# Update to latest version
faster --update
```

## Configuration

**Auto-detected from Claude Code:**
- Authentication session (no separate login)
- User preferences (model, temperature, etc.)
- Project context (current directory)

**Optional Config (~/.faster/config.sh):**
```bash
# TTS voice selection (platform-specific)
FASTER_VOICE="Samantha"  # macOS voice name

# STT engine preference
FASTER_STT="whisper"  # whisper, native, or auto

# Silence detection threshold (seconds)
FASTER_SILENCE_THRESHOLD=1.5

# Show transcripts inline
FASTER_SHOW_TRANSCRIPTS=true
```

## Implementation Strategy

**Primary: Bash/Shell Script**
- Maximum portability (works everywhere)
- Zero runtime dependencies beyond standard tools
- Easy to audit and modify
- Can shell out to platform-specific tools
- Simple install: `curl -fsSL install.sh | bash`

**Platform-Specific Modules:**
- Detect OS and use native capabilities
- macOS: Shortcuts/AppleScript for dictation, `say` for TTS
- Linux: whisper-cpp + espeak-ng
- Windows: PowerShell for dictation + SAPI

**Why Not Python/Rust/Go:**
- Python: Requires runtime, complex dependencies
- Rust/Go: Overkill for gluing existing tools together
- Shell: Already on every system, perfect for this use case

## Critical Success Factors

1. **Latency Measurement**: Build in telemetry from day one
2. **Error Recovery**: Network failures, bad audio, STT errors must not crash
3. **UX Feedback**: Visual indicators in CLI (listening, thinking, speaking states)
4. **Testing Strategy**: Record test audio samples for regression testing
5. **Streaming**: Use streaming for Claude responses and TTS to minimize perceived latency
