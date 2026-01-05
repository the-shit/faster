# Faster - Architecture Specification

**Voice-driven deterministic intent processor for Claude Code**

## Core Philosophy

Faster translates messy human speech into deterministic, structured commands for Claude. The local AI layer **forces** ambiguous input into a rigid schema, eliminating Claude's need to "figure out what you want" - it receives clear directives only.

## System Roles

### Local AI Layer
**Role**: Intent Translation (NOT execution)
- Listens to chaotic human speech
- Extracts structured data (entities, actions, context)
- Queries knowledge for disambiguation
- Forces into deterministic Command schema
- **Never executes** - only prepares directives

### Claude Layer
**Role**: Execution (NOT intent discovery)
- Receives clear, unambiguous directives
- Four modes only:
  1. **Orchestrate**: Manage workflows, spawn agents
  2. **Research**: Search code, gather context
  3. **Code**: Generate, edit, refactor
  4. **Test**: Run tests, debug, fix
- **Never figures out intent** - executes what it's told

## Architecture Layers

```
┌─────────────────────────────────────────────────┐
│  AUDIO LAYER (Rust)                             │
│  - Continuous listening (VAD)                   │
│  - macOS native STT (free, fast)                │
│  - macOS native TTS (system voices)             │
│  - Interruption support                         │
└──────────────┬──────────────────────────────────┘
               │
               ▼ (transcribed text)
┌─────────────────────────────────────────────────┐
│  INTENT PROCESSING LAYER (Rust + Llama 3.2)    │
│  - Extract entities and actions                 │
│  - Query knowledge for context                  │
│  - Ensemble validation (confidence)             │
│  - Force into structured Command schema         │
└──────────────┬──────────────────────────────────┘
               │
               ▼ (deterministic Command)
┌─────────────────────────────────────────────────┐
│  KNOWLEDGE LAYER (Rust + SQLite)                │
│  - Speech patterns (user's shortcuts)           │
│  - Current context (active work)                │
│  - Goals & milestones (what's being built)      │
│  - Decision rationale (why choices were made)   │
│  - Sync non-sensitive to prefrontal-cortex      │
└──────────────┬──────────────────────────────────┘
               │
               ▼ (enriched Command)
┌─────────────────────────────────────────────────┐
│  EXECUTION LAYER (Rust → Claude Code CLI)      │
│  - Configurable confirmation                    │
│  - Send directive to Claude Code                │
│  - Stream response back                         │
│  - Convert to speech                            │
└──────────────┬──────────────────────────────────┘
               │
               ▼ (async)
┌─────────────────────────────────────────────────┐
│  OBSERVABILITY LAYER (PHP Conduit Bridge)       │
│  - VoiceIntakeTool (ToolRole::INTAKE)           │
│  - HTTP events to prefrontal-cortex             │
│  - Contract tests, Observable, KnowledgeAware   │
└─────────────────────────────────────────────────┘
```

## Deterministic Command Schema

```rust
/// The rigid schema that forces messy speech into structure
pub struct Command {
    /// Intent category (4 options only)
    pub intent: Intent,

    /// Clear directive for Claude (unambiguous instruction)
    pub directive: String,

    /// Extracted entities (files, modules, functions)
    pub entities: Vec<String>,

    /// User context from knowledge system
    pub context: HashMap<String, String>,

    /// Confidence score (0.0-1.0)
    pub confidence: f32,

    /// Timestamp
    pub created_at: DateTime<Utc>,
}

pub enum Intent {
    Orchestrate, // Manage workflows, coordinate agents
    Research,    // Search code, gather context
    Code,        // Generate, edit, refactor
    Test,        // Run tests, debug, fix
}
```

## Data Flow Example

**Input**: "hey run tests on that auth thing"

### Step 1: Audio Layer
```
VAD detects speech → STT transcribes
Output: "hey run tests on that auth thing"
```

### Step 2: Intent Processing
```rust
// Llama 3.2 extracts
entities: ["run", "tests", "auth thing"]
action: "run tests"
target: "auth thing"

// Query knowledge
knowledge.query("auth thing")
  → "authentication module" (from recent context)

// Ensemble validation
ensemble_vote(&entities)
  → confidence: 0.95
```

### Step 3: Force to Schema
```rust
Command {
    intent: Intent::Test,
    directive: "Run test suite for authentication module",
    entities: vec!["authentication", "tests"],
    context: hashmap!{
        "current_module" => "auth",
        "current_goal" => "auth refactor",
    },
    confidence: 0.95,
    created_at: Utc::now(),
}
```

### Step 4: Confirmation (Optional)
```
TTS: "Running tests for authentication module"
[1s pause for interruption]
```

### Step 5: Execute
```bash
claude "Run the test suite for the authentication module"
```

### Step 6: Knowledge Capture
```json
{
  "type": "ambiguity_resolution",
  "from": "auth thing",
  "to": "authentication module",
  "context": "user_working_on_auth_refactor",
  "confidence": 0.95
}
```

### Step 7: Sync (Non-Sensitive)
```
POST /api/knowledge/insights
{
  "category": "speech_pattern",
  "pattern": "auth thing → authentication module",
  "goal": "auth refactor"
}
```

## Knowledge System

### What Gets Captured

| Category | Examples | Sensitivity | Sync |
|----------|----------|-------------|------|
| **Speech Patterns** | "auth thing" → "authentication module" | Medium | No (private) |
| **Current Context** | Working on: auth refactor | Low | Yes |
| **Goals** | "Complete OAuth integration" | Low | Yes |
| **Milestones** | "Auth tests passing" | Low | Yes |
| **Decisions** | "Chose Llama over Phi for speed" | Low | Yes |
| **Raw Transcripts** | Full voice recordings | High | No (private) |

### Triggers for Capture

1. **Ambiguity Resolution**: AI disambiguated vague input
2. **Milestone Completion**: Tests pass, feature done
3. **Explicit Marking**: User says "remember this" or "mark milestone"
4. **Decision Points**: Choosing between options

### Knowledge Injection

- **Not automatic** - User controls with explicit commands
- **Context command**: "with my context" injects relevant knowledge
- **Session-level**: Optionally load at session start
- **Adapts style**: Learns communication preferences (brevity, detail level)

## Technology Stack

### Core Implementation: Rust
- **Why**: Compiled binary, no runtime, fast, memory-safe
- **Crates**:
  - `llama-cpp-rs`: Local AI inference
  - `cpal`: Audio capture/playback (macOS native)
  - `tokio`: Async runtime
  - `sqlx`: SQLite for knowledge
  - `reqwest`: HTTP client for API
  - `serde`: Serialization

### Local AI: Llama 3.2 (3B)
- **Why**: Best speed/quality for classification
- **Fallback**: Claude Haiku API ($0.0001/prompt)
- **Model**: `llama-3.2-3b-instruct.gguf` (~2GB)
- **Inference**: <1s on M-series Mac

### STT/TTS: macOS Native
- **STT**: `NSSpeechRecognizer` (via Swift bridge)
- **TTS**: `say` command with enhanced voices
- **Cost**: $0 (local, private)
- **Latency**: STT ~500ms, TTS ~300ms

### PHP Conduit Bridge
- **Framework**: Laravel (matches ecosystem)
- **Purpose**: Conduit contract compliance
- **Components**:
  - `VoiceIntakeTool` (implements `IntakeManager`)
  - `VoiceIntakeContext` (readonly value object)
  - `VoiceIntakeResult` (readonly value object)
  - Contract tests extending shipped abstracts

### Knowledge Storage
- **Local**: SQLite (privacy-first)
- **Schema**:
  ```sql
  CREATE TABLE patterns (
    id INTEGER PRIMARY KEY,
    from_phrase TEXT,
    to_entity TEXT,
    context TEXT,
    confidence REAL,
    usage_count INTEGER,
    created_at TEXT
  );

  CREATE TABLE goals (
    id INTEGER PRIMARY KEY,
    description TEXT,
    status TEXT, -- active, completed, paused
    created_at TEXT,
    completed_at TEXT
  );

  CREATE TABLE milestones (
    id INTEGER PRIMARY KEY,
    goal_id INTEGER,
    description TEXT,
    completed_at TEXT
  );

  CREATE TABLE decisions (
    id INTEGER PRIMARY KEY,
    question TEXT,
    choice TEXT,
    rationale TEXT,
    created_at TEXT
  );
  ```

## Configuration

### User Config (`~/.faster/config.toml`)

```toml
[audio]
input_device = "default"
sample_rate = 16000
vad_threshold = 0.5

[stt]
provider = "macos-native"
language = "en-US"

[tts]
provider = "macos-native"
voice = "Samantha"
rate = 200  # words per minute

[intent]
model = "llama-3.2-3b-instruct"
confidence_threshold = 0.80
ensemble_size = 3

[confirmation]
mode = "smart"  # always, never, smart, destructive-only
timeout_ms = 1000

[knowledge]
local_db = "~/.faster/knowledge.db"
sync_endpoint = "http://localhost/api/knowledge/sync"
sync_mode = "non-sensitive"  # all, non-sensitive, never

[claude]
cli_path = "claude"
model = "sonnet"

[observability]
conduit_endpoint = "http://localhost/api/voice/events"
```

## Installation & Setup

### Prerequisites
```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Llama.cpp
brew install llama.cpp

# PHP/Composer (for Conduit bridge)
brew install php composer
```

### Build from Source
```bash
cd /Users/jordanpartridge/domain/faster

# Build Rust binary
cargo build --release

# Install to PATH
cp target/release/faster ~/.local/bin/

# Download Llama model
mkdir -p models
curl -L https://huggingface.co/TheBloke/Llama-3.2-3B-Instruct-GGUF/resolve/main/llama-3.2-3b-instruct.Q4_K_M.gguf \
  -o models/llama-3.2-3b-instruct.gguf

# Setup Conduit bridge
cd bridge
composer install
php artisan test  # Verify contract compliance
```

### First Run
```bash
faster --setup

# Prompts for:
# - API endpoint (prefrontal-cortex URL)
# - Confirmation mode
# - Knowledge sync preferences
# - Voice selection

faster --test  # Verify all components
```

## Usage

### Basic Commands
```bash
# Start voice mode
faster

# With specific Claude model
faster --model opus

# Debug mode (show all processing)
faster --debug

# Test installation
faster --test

# Configure
faster --config
```

### Voice Commands

**Examples of deterministic translation:**

| You Say | Local AI Extracts | Claude Receives |
|---------|-------------------|-----------------|
| "run tests on auth" | intent: TEST, target: auth | "Run test suite for authentication module" |
| "fix that bug from yesterday" | intent: CODE, context: bug#1234 | "Debug and fix bug #1234" |
| "show me the recent PRs" | intent: RESEARCH, target: PRs | "List recent pull requests with status" |
| "build the payment feature" | intent: CODE, target: payment | "Implement payment feature module" |
| "check if tests pass" | intent: TEST, action: check | "Run test suite and report status" |

### Knowledge Commands

```bash
# Explicit marking
"remember this decision: chose Rust for speed"

# Mark milestone
"milestone: OAuth integration complete"

# Update goal
"current goal: refactor authentication system"

# Query knowledge (for debugging)
faster knowledge --patterns
faster knowledge --goals
faster knowledge --context
```

## Observability

### Events Emitted to Conduit Bridge

```php
// VoiceIntake events (to prefrontal-cortex)
event(new VoiceIntakeStarted($context));
event(new IntentDetected($command));
event(new AmbiguityResolved($from, $to));
event(new ClaudeExecutionStarted($directive));
event(new ClaudeExecutionCompleted($result));
event(new MilestoneReached($milestone));
event(new VoiceIntakeFailed($error));
```

### Metrics Tracked

- Intent detection accuracy (confidence scores)
- Speech-to-execution latency
- Knowledge query hit rate
- Ambiguity resolution frequency
- Confirmation rate (how often user confirms)
- Interruption rate (how often user interrupts Claude)

## Testing Strategy

### Rust Tests
```bash
# Unit tests
cargo test

# Integration tests (requires local AI model)
cargo test --features integration

# Benchmark intent processing latency
cargo bench
```

### PHP Conduit Bridge Tests
```php
// Extends shipped abstract tests
class VoiceIntakeToolTest extends IntakeManagerContractTest
{
    protected function createTool(): Tool {
        return app(VoiceIntakeTool::class);
    }

    protected function createValidContext(): Context {
        return new VoiceIntakeContext(
            transcript: "run tests on auth module",
            command: new Command(...),
            confidence: 0.95,
        );
    }

    // All contract tests run automatically ✓
}
```

### End-to-End Tests
```bash
# Simulate voice input
faster test --input "run tests on auth"

# Expected output:
# → Intent: TEST
# → Directive: "Run test suite for authentication module"
# → Confidence: 0.95
# → Executing via Claude Code...
# → [Claude response]
```

## Performance Targets

| Metric | Target | Actual |
|--------|--------|--------|
| STT latency | <500ms | TBD |
| Intent processing | <1s | TBD |
| Total turn latency | <2s | TBD |
| Knowledge query | <50ms | TBD |
| Confidence threshold | >80% | TBD |
| Intent accuracy | >90% | TBD |

## Security & Privacy

### Local-First
- All speech processing on-device
- Raw transcripts NEVER leave machine
- API only receives non-sensitive insights

### What Gets Synced
✅ Goals, milestones, decisions
✅ Speech patterns (anonymized)
✅ Intent accuracy metrics
❌ Raw audio
❌ Full transcripts
❌ Personal information

### API Security
- HTTPS only for sync
- JWT authentication
- Rate limiting: 100 req/hour
- No PII in payloads

## Roadmap

### Phase 1: MVP (Current)
- [x] Architecture designed
- [ ] Rust audio layer
- [ ] Local AI integration
- [ ] Deterministic schema
- [ ] Claude Code executor
- [ ] Basic knowledge store

### Phase 2: Knowledge
- [ ] SQLite knowledge DB
- [ ] Pattern learning
- [ ] Context tracking
- [ ] Non-sensitive sync

### Phase 3: Conduit Bridge
- [ ] PHP VoiceIntakeTool
- [ ] Contract tests
- [ ] Observable events
- [ ] HTTP API integration

### Phase 4: Polish
- [ ] Interruption handling
- [ ] Confirmation modes
- [ ] Voice preferences
- [ ] Performance optimization

### Phase 5: Advanced
- [ ] Multi-language support
- [ ] Custom wake word
- [ ] Streaming intent detection
- [ ] Cross-device sync

## References

- [BUILDING_CONDUIT_TOOLS.md](/Users/jordanpartridge/Sites/prefrontal-cortex/docs/BUILDING_CONDUIT_TOOLS.md)
- [Llama 3.2 Model Card](https://huggingface.co/meta-llama/Llama-3.2-3B-Instruct)
- [Claude Code CLI Docs](https://claude.ai/code)
- [Rust Audio Programming](https://docs.rs/cpal)
