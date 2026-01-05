# ⚡ Faster

**The way coding was always meant to be.**

Talk to Claude Code like you talk to your pair programming partner. Natural, conversational, instant. No typing. No context-switching. Just pure flow.

```bash
curl -fsSL https://raw.githubusercontent.com/the-shit/faster/main/install.sh | bash
faster
```

🎤 *"What files are in this directory?"*
🤖 *[Claude responds via voice]*
🎤 *"Run the tests"*
🤖 *[Tests execute, results spoken]*
🎤 *"Fix the failing one"*
🤖 *[Code fixed, committed]*

**That's it. That's the entire workflow.**

---

## Why This Changes Everything

### ⚡ **Zero Friction**
If you're logged into Claude Code, voice works. No API keys. No configuration files. No setup wizard. One install command, then just talk.

### 🧠 **Enforced Clarity**
Speech forces you to think in commands, not essays. You can't ramble. You say what you mean. Claude does it. This constraint is the feature.

### 🚀 **Impossible Speed**
Sub-2-second turnaround. STT running while Claude generates. TTS streaming while Claude thinks. Every millisecond optimized. You'll forget you're not talking to a human.

### 🔒 **Local-First Philosophy**
Your voice never leaves your machine (whisper-cpp). No cloud STT APIs. No telemetry. No privacy trade-offs. Just local AI doing local work.

### 🎯 **Context That Actually Works**
Uses Claude Code's session management. Claude remembers your conversation. Your project. Your preferences. It's already in the repo. It already knows.

---

## The Vision

We're building toward a future where:

- **Voice is the primary interface** for development work
- **Hands stay on the keyboard** only when writing complex logic
- **Context switching disappears** - no alt-tabbing to ChatGPT
- **Flow states extend** because you never break focus to type
- **Interruptions are natural** - cancel mid-response, ask follow-ups, iterate live

This isn't just "speech-to-text for your terminal." This is rethinking how developers interact with AI.

---

## Installation

### One-Line Install
```bash
curl -fsSL https://raw.githubusercontent.com/the-shit/faster/main/install.sh | bash
```

**What it does:**
1. Checks for Claude Code auth (prompts you to `claude login` if needed)
2. Installs platform-specific dependencies (whisper-cpp, sox, etc.)
3. Adds `faster` to your PATH
4. Optionally installs as Claude Code skill: `/voice`

**First run:**
```bash
$ faster
✓ Authenticated as you@email.com
🎤 Listening... (press Ctrl+C to exit)
```

---

## How It Works

```
┌─────────────┐
│  You Speak  │ → Microphone capture (sox/cpal)
└─────┬───────┘
      ▼
┌─────────────┐
│     VAD     │ → Detect speech start/stop
└─────┬───────┘
      ▼
┌─────────────┐
│     STT     │ → whisper-cpp (local, fast, accurate)
└─────┬───────┘
      ▼
┌─────────────┐
│ Claude Code │ → Uses your existing session, streams response
└─────┬───────┘
      ▼
┌─────────────┐
│     TTS     │ → Native voices (macOS/Linux/Windows)
└─────┬───────┘
      ▼
┌─────────────┐
│   Speaker   │ → You hear the response while Claude is still generating
└─────────────┘
```

**The magic:** Each stage runs asynchronously. STT can process while Claude generates. TTS starts speaking before Claude finishes. Latency targets: <2s for simple queries.

---

## Usage Patterns

### Natural Commands
```
"What files are in this directory?"
"Show me the implementation of the auth middleware"
"Run the test suite"
"Create a new component called UserProfile"
"Fix the TypeScript error on line 47"
"What's the git status?"
"Commit these changes"
```

### Continuous Conversation
```
You: "What's in package.json?"
Claude: "You have dependencies for React, TypeScript, Vite..."
You: "Add lodash to that"
Claude: "Installing lodash... Done. Added to dependencies."
You: "Now import it in utils.ts"
Claude: "Imported at the top of utils.ts. Anything else?"
```

### Interruption Handling
Claude is speaking, but you realize you asked the wrong thing:
```
Claude: "Let me explain how React hooks work. First, useState allows you to—"
You: "Actually, just show me an example"
Claude: [stops talking, shows example code]
```

### Advanced Modes
```bash
faster --model opus          # Use Claude Opus
faster --continuous          # Always listening (no push-to-talk)
faster --debug               # Show transcripts and timing
faster --test                # Verify installation
```

---

## Configuration

Optional `~/.faster/config.sh`:

```bash
# TTS Voice (platform-specific)
FASTER_VOICE="Samantha"  # macOS: "Samantha", "Alex", etc.
                         # Linux: espeak voice names
                         # Windows: SAPI voice names

# STT Engine
FASTER_STT="whisper"     # whisper, native, or auto

# Silence threshold (seconds of silence = end of speech)
FASTER_SILENCE_THRESHOLD=1.5

# Show transcripts inline
FASTER_SHOW_TRANSCRIPTS=true

# Model preference
FASTER_MODEL="sonnet"    # sonnet, opus, haiku
```

---

## Platform Support

| Platform | STT | TTS | Status |
|----------|-----|-----|--------|
| macOS    | whisper-cpp / Dictation | `say` | ✅ Fully supported |
| Linux    | whisper-cpp | espeak-ng | ✅ Fully supported |
| Windows  | whisper-cpp | SAPI | 🚧 Experimental |

---

## The Philosophy

### Brevity Over Verbosity
Voice forces concise thinking. You can't type a 3-paragraph prompt while talking. This is good. Say what you need. Claude handles the rest.

### Speed Over Features
We optimize latency over everything. Every component is benchmarked. If something adds >100ms, it's cut. Flow state demands speed.

### Local Over Cloud
Your voice, your data, your machine. We use free, local AI wherever possible. Only Claude's responses go to the cloud (because that's the Claude Code API you're already using).

### Simplicity Over Configurability
One install command. Zero required config. Sensible defaults. Advanced users can tweak, but beginners should just work.

---

## Roadmap

**v0.1** (Current)
- [x] Basic voice loop (listen → transcribe → Claude → speak)
- [x] macOS/Linux support
- [x] Local whisper-cpp STT
- [x] Sub-2s latency target

**v0.2** (Next)
- [ ] Interruption handling (cancel TTS on new speech)
- [ ] Session management (name sessions, switch context)
- [ ] Streaming TTS (speak while Claude generates)
- [ ] Wake word support (optional)

**v0.3** (Future)
- [ ] Windows native support
- [ ] Multi-language support
- [ ] Voice activity visualization
- [ ] Conversation history export
- [ ] Integration with Claude Code skills

**v1.0** (Vision)
- [ ] Real-time code generation via voice
- [ ] Multi-turn refactoring conversations
- [ ] Voice-first debugging workflows
- [ ] Team voice sessions (multi-user)

---

## Contributing

This is **early**. We're figuring it out as we go. If you want to help:

1. **Use it.** Break it. Tell us what's broken.
2. **Optimize latency.** Every millisecond matters. Benchmark your changes.
3. **Add platform support.** We want this everywhere.
4. **Improve STT/TTS.** Better models, better voices, better accuracy.

See `CLAUDE.md` for architecture details and `ARCHITECTURE.md` for implementation design.

---

## FAQ

**Q: Do I need API keys?**
A: No. Uses Claude Code's existing auth.

**Q: Does my voice go to OpenAI/Google/etc?**
A: No. whisper-cpp runs locally. Only Claude responses use the cloud API.

**Q: How accurate is the transcription?**
A: Very. whisper-cpp is state-of-the-art. Accuracy depends on mic quality and background noise.

**Q: Can I use it offline?**
A: STT and TTS work offline. Claude Code API requires internet (same as typing to Claude).

**Q: How much does it cost?**
A: Same as using Claude Code normally. No additional API costs.

**Q: Why Rust?**
A: Speed. Latency is everything. Rust gives us control + safety.

**Q: Can I use a different TTS voice?**
A: Yes. Configure `FASTER_VOICE` in `~/.faster/config.sh`. Platform voices or edge-tts.

---

## Credits

Built on top of:
- [whisper-cpp](https://github.com/ggerganov/whisper.cpp) - Fast, local STT
- [cpal](https://github.com/RustAudio/cpal) - Cross-platform audio
- [Claude Code](https://claude.ai/code) - The AI pair programmer

---

## License

MIT

---

**Talk less. Code more. Go faster.** ⚡
