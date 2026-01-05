# Faster: Two-Tier AI Development Assistant
**A Privacy-First, Always-On Voice Interface for Claude Code**

*Proposal v1.0 - January 4, 2026*

---

## Executive Summary

**Faster** is a voice-powered development assistant that combines local AI (always listening, privacy-first) with Claude Code (complex reasoning, code generation) to create a natural, efficient development workflow.

**Key Innovation:** Two-tier intelligence routing - local AI handles 90% of queries instantly and privately, invoking Claude only for complex tasks like code generation and analysis.

**Core Value:** Transform 27 hours/month of repetitive scaffolding into 3 hours through voice-activated skill automation.

---

## The Problem

### Current Development Workflow

1. **Repetitive Scaffolding**: Developers spend 30-45 minutes manually creating each API endpoint, model, test, etc.
   - Copy/paste from previous examples
   - Manual modification of boilerplate
   - Route registration, test creation
   - **27 hours/month** on repetitive tasks (based on audit of 54 features/month)

2. **Context Switching**: Constant switching between keyboard, documentation, terminal
   - Breaking flow state
   - Losing train of thought
   - Slower than natural speech for expressing ideas

3. **Privacy Concerns**: Sending every query to cloud APIs
   - Personal file contents
   - Private repository data
   - Local development context

4. **Latency**: Traditional voice assistants require cloud round-trips
   - 2-5 seconds for simple queries
   - Breaks conversational flow

---

## The Solution: Two-Tier Architecture

### Tier 1: Local AI (Always Listening)

**Purpose:** Handle simple queries, route complex ones, protect privacy

**Capabilities:**
- Voice activity detection (VAD)
- Speech-to-text (Whisper locally)
- Intent classification
- Simple queries (filesystem, git, time/date)
- Privacy filtering (personal data stays local)
- **Decision routing:** Local or Claude?

**Technology:**
- **STT:** Whisper.cpp (base.en model, ~140MB)
- **Intelligence:** Llama 3.2 3B / Phi-3 Mini / Gemma 2B
- **TTS:** Native OS voices (macOS: `say`, Linux: `espeak`, Windows: SAPI)
- **Cost:** $0 (all local)
- **Latency:** <500ms for local queries

**Example Flows (Local Only):**
```
You: "What time is it?"
Local: [Instant] "2:47 PM"

You: "List files in this directory"
Local: [Reads filesystem] "main.py, config.json, README.md, tests/"

You: "What's my last git commit?"
Local: [Runs git log] "feat: add session recovery endpoint"

You: "Show me the TODO in main.py"
Local: [Greps file] "Line 42: TODO: Add error handling"
```

### Tier 2: Claude Code (On-Demand)

**Purpose:** Complex reasoning, code generation, skill execution

**Invoked For:**
- Code generation (models, controllers, tests)
- Refactoring and analysis
- Complex queries requiring deep understanding
- Skill execution (scaffolding automation)
- Planning and architecture decisions

**Technology:**
- Claude Code CLI (existing authentication)
- Streaming responses
- Skill system for code generation

**Example Flows (Local → Claude):**
```
You: "Create an API endpoint for user preferences"
Local: [Detects complexity] → Invokes Claude
Claude: [Analyzes intent] → "This is an API endpoint pattern"
Claude: [Executes] /scaffold:api-endpoint user-preferences
Claude: [Generates] Controller + Routes + Tests
Local: [Speaks] "Created endpoint at app/Http/Controllers/Api/UserPreferencesController.php with routes and tests"

You: "Why is this function slow?"
Local: [Detects analysis needed] → Invokes Claude
Claude: [Analyzes code] → Identifies N+1 query problem
Claude: [Suggests] "The issue is an N+1 query on line 23. Use eager loading..."
Local: [Speaks] Claude's analysis
```

---

## Intelligence Routing Logic

### How Local AI Decides

```python
def route_query(transcript: str) -> Response:
    """Local AI decision tree"""

    intent = classify_intent(transcript)

    # Simple system/filesystem queries → Local
    if intent in ["filesystem", "git", "time", "clipboard"]:
        return handle_locally(transcript)

    # Privacy-sensitive data → Local only
    if contains_personal_data(transcript):
        return handle_locally(transcript)

    # Code generation/analysis → Claude
    if intent in ["code_gen", "refactor", "analyze", "debug"]:
        return invoke_claude(transcript)

    # Skill execution → Claude
    if matches_skill_pattern(transcript):
        return invoke_claude(transcript)

    # Uncertain → Use Claude (better safe than wrong)
    return invoke_claude(transcript)
```

### Intent Classification Examples

| User Input | Intent | Handler | Latency |
|------------|--------|---------|---------|
| "What time is it?" | `time` | Local | <100ms |
| "List files here" | `filesystem` | Local | <500ms |
| "Last commit message" | `git` | Local | <300ms |
| "Create a User model" | `code_gen` | Claude | 2-5s |
| "Why is this slow?" | `analyze` | Claude | 3-8s |
| "Refactor this function" | `refactor` | Claude | 5-10s |

---

## Skill Automation System

### The Pattern Problem

**Audit Finding:** 54 feature implementations in last month following repetitive patterns:

1. **API Endpoints** (16+ instances)
   - Manual effort: 30-45 min each
   - Pattern: Controller + Route + Test + Middleware

2. **Models** (11 instances)
   - Manual effort: 45-60 min each
   - Pattern: Model + Migration + Factory + Test

3. **Filament Widgets** (5 instances)
   - Manual effort: 20-30 min each
   - Pattern: Widget + Test + Registration

4. **Tests** (86 total, gaps exist)
   - Manual effort: 15-30 min each
   - Pattern: Pest describe/it + Factories + Assertions

**Total Time Spent:** ~27 hours/month on repetitive scaffolding

### The Skill Solution

**Skills = Standardized Code Generation Templates**

Each skill encodes project-specific patterns and conventions:

```bash
/scaffold:api-endpoint {name}
→ Generates: Controller + Route + Test following exact project standards

/scaffold:model {name} --has-many=posts
→ Generates: Model + Migration + Factory + Test with relationships

/scaffold:filament-widget {name}
→ Generates: Widget + Test + Auto-registration

/generate:tests {file}
→ Analyzes existing code, generates comprehensive test coverage
```

### Voice → Skill Integration

**Natural Language Input:**
```
You: "I need an API endpoint for dashboard statistics"

Local AI: [Transcribes] → [Analyzes intent] → [Routes to Claude]

Claude: [Understands] "User wants API endpoint"
        [Extracts] Resource name: "dashboard-statistics"
        [Executes] /scaffold:api-endpoint dashboard-stats --read-only

Skill: [Generates]
  ✅ app/Http/Controllers/Api/DashboardStatsController.php
  ✅ Route in routes/api.php
  ✅ tests/Feature/Api/DashboardStatsTest.php

Claude: [Reports] "Created API endpoint with controller, routes, and tests"

Local: [Speaks result to user]
```

**Time Saved:** 30-45 minutes → 5 seconds

### Priority Skills to Build

**Phase 1 (Highest ROI):**
1. `/scaffold:api-endpoint` - 16+ uses/month
2. `/scaffold:model` - Foundational pattern
3. `/generate:tests` - Fill coverage gaps

**Phase 2:**
4. `/scaffold:filament-widget` - Dashboard components
5. `/scaffold:filament-resource` - Full CRUD

**Phase 3:**
6. `/scaffold:service` - Business logic layer
7. `/scaffold:event-listener` - Event-driven patterns

---

## System Architecture

```
┌─────────────────────────────────────────────────────────┐
│              User (Voice Input)                         │
└────────────────────┬────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────────────┐
│  Tier 1: Local AI (Always Listening)                    │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│  • Microphone → VAD → Whisper STT                       │
│  • Intent Classification (Llama/Phi/Gemma)              │
│  • Privacy Filter                                        │
│  • Route Decision                                        │
└────────┬──────────────────────────────┬─────────────────┘
         │                              │
    [Simple Query]               [Complex Query]
         │                              │
         ↓                              ↓
┌────────────────────┐    ┌─────────────────────────────┐
│  Local Handler     │    │  Tier 2: Claude Code        │
│  ━━━━━━━━━━━━━━━  │    │  ━━━━━━━━━━━━━━━━━━━━━━━━  │
│  • Filesystem ops  │    │  • Code generation          │
│  • Git commands    │    │  • Complex analysis         │
│  • Time/date       │    │  • Skill execution          │
│  • Clipboard       │    │  • Refactoring              │
│  • System info     │    │  • Planning                 │
└────────┬───────────┘    └──────────┬──────────────────┘
         │                           │
         └──────────┬────────────────┘
                    ↓
         ┌──────────────────────┐
         │  Local TTS Output    │
         │  (OS native voices)  │
         └──────────────────────┘
```

---

## Privacy Architecture

### Data Flow Classification

**Level 1: Never Leaves Machine (Local AI Only)**
- File listings and directory structure
- Git status and commit messages
- Personal notes and TODO items
- Time/date queries
- System information
- Clipboard contents

**Level 2: Sent to Claude (User Consent)**
- Code being generated
- Files being analyzed
- Refactoring requests
- Architecture questions
- Public repository context

**Privacy Controls:**
```bash
# User configuration (~/.faster/config.sh)

# Privacy mode: Only send code explicitly marked for analysis
FASTER_PRIVACY_MODE=strict

# Whitelist: Only these directories can be sent to Claude
FASTER_ALLOWED_PATHS="~/projects/public/*"

# Blacklist: Never send these to Claude
FASTER_BLOCKED_PATHS="~/personal/*,~/.env*,*/secrets/*"
```

---

## Implementation Strategy

### Technology Choices

**Distribution:**
- Shell script (bash/zsh) for maximum portability
- Single `curl | bash` install
- Zero runtime dependencies beyond Claude Code

**Platform Support:**
- **macOS:** Native Dictation API, `say` command for TTS
- **Linux:** whisper.cpp + espeak-ng
- **Windows:** PowerShell dictation + SAPI voices

**Local AI Models:**
- **Primary:** Llama 3.2 3B (best balance)
- **Fallback:** Phi-3 Mini (faster, less capable)
- **Ultra-light:** Gemma 2B (low-end hardware)

**Installation:**
```bash
# One-line install
curl -fsSL https://raw.githubusercontent.com/USER/faster/main/install.sh | bash

# What it does:
# 1. Checks Claude Code authentication
# 2. Installs platform-specific dependencies
# 3. Downloads local AI model (~2GB)
# 4. Adds 'faster' command to PATH
# 5. Optional: Installs as Claude Code skill (/voice)
```

---

## Usage Examples

### Typical Development Session

```
$ faster
✓ Authenticated as user@email.com
🎤 Listening... (Ctrl+C to exit)

[You]: "What's in this directory?"
[Local AI]: "You have app/, config/, database/, routes/, tests/,
             and composer.json"

[You]: "Show me recent commits"
[Local AI]: "Last 3 commits:
             - feat: add session recovery
             - fix: blade syntax error
             - feat: PR insights widget"

[You]: "I need a model for tracking API usage"
[Local AI]: [Routes to Claude]
[Claude]: "Creating a model for API usage tracking..."
[Claude]: [Executes /scaffold:model api-usage]
[Claude]: "Generated ApiUsage model with migration, factory, and tests"

[You]: "Add a relationship to User"
[Claude]: "Adding belongsTo relationship to User model..."
[Claude]: [Updates model, migration, test]
[Claude]: "Done. ApiUsage now belongs to User"

[You]: "Run the tests"
[Local AI]: [Executes] php artisan test --filter=ApiUsageTest
[Local AI]: "All tests passing. 8 tests, 24 assertions"

[You]: "Commit this"
[Claude]: [Analyzes changes]
[Claude]: [Creates commit]
[Local AI]: "Committed: 'feat: add ApiUsage model with User relationship'"
```

---

## Performance Targets

### Latency Goals

| Operation | Target | Current (Manual) |
|-----------|--------|------------------|
| Simple query (local) | <500ms | N/A (typing) |
| Complex query (Claude) | <2s to first word | 30-45 min |
| Code generation | <5s total | 30-45 min |
| Skill execution | <3s | 30-45 min |

### Accuracy Goals

| Metric | Target |
|--------|--------|
| STT accuracy | >95% (technical terms) |
| Intent classification | >90% |
| Routing decision | >95% correct tier |
| Generated code quality | 100% test pass rate |

### Cost Targets

| Usage | Cost |
|-------|------|
| Local queries | $0 (100% local) |
| Claude invocations | ~10-20/day avg |
| Monthly Claude API | <$50 for heavy use |

---

## Expected Impact

### Time Savings

**Before (Manual):**
- API endpoint: 30-45 min
- Model scaffold: 45-60 min
- Widget: 20-30 min
- Tests: 15-30 min
- **Monthly total:** 27 hours (54 features × 30min avg)

**After (With Faster):**
- API endpoint: 5 seconds
- Model scaffold: 5 seconds
- Widget: 3 seconds
- Tests: 5 seconds
- **Monthly total:** 2.7 hours (54 features × 3min avg)

**Savings: 24 hours/month → 96 hours/quarter → 384 hours/year**

### Workflow Improvements

1. **No Context Switching**: Stay in flow state
2. **Natural Expression**: Speak ideas naturally vs typing
3. **Instant Feedback**: Immediate response for simple queries
4. **Hands-Free**: Code while walking, cooking, away from desk
5. **Privacy First**: Personal data never leaves machine

---

## Development Roadmap

### Phase 1: Foundation (Weeks 1-2)
- [ ] Build local AI routing logic
- [ ] Integrate Whisper.cpp for STT
- [ ] Set up native TTS per platform
- [ ] Create `/scaffold:api-endpoint` skill
- [ ] Create `/scaffold:model` skill
- [ ] Test end-to-end flow

### Phase 2: Core Skills (Weeks 3-4)
- [ ] Build `/generate:tests` skill
- [ ] Build `/scaffold:filament-widget` skill
- [ ] Build `/scaffold:filament-resource` skill
- [ ] Refine intent classification
- [ ] Add privacy filtering

### Phase 3: Polish (Weeks 5-6)
- [ ] Optimize latency (VAD tuning)
- [ ] Add interruption support
- [ ] Create installation script
- [ ] Documentation and examples
- [ ] Beta testing with real workflows

### Phase 4: Release (Week 7)
- [ ] Public release
- [ ] Usage analytics
- [ ] Iterate based on feedback

---

## Success Metrics

### Quantitative (3 Months Post-Launch)
- **Adoption:** 100+ active users
- **Time saved:** >20 hours/month per user
- **Query distribution:** 90% local, 10% Claude (as designed)
- **Accuracy:** >90% intent classification
- **User satisfaction:** >4.5/5 rating

### Qualitative
- Users report staying in flow state longer
- Reduced context switching
- More natural development experience
- Privacy concerns addressed
- "Feels like pair programming with expert"

---

## Risk Mitigation

### Technical Risks

| Risk | Mitigation |
|------|------------|
| Local AI misroutes query | Default to Claude on uncertainty; learn from corrections |
| STT accuracy on technical terms | Build custom vocabulary; allow manual correction |
| Privacy leak | Strict allowlist/blocklist; audit all Claude invocations |
| Model compatibility | Support multiple models (Llama, Phi, Gemma) |
| Platform-specific issues | Extensive testing; fallback modes |

### User Experience Risks

| Risk | Mitigation |
|------|------------|
| "Creepy" always-listening | Clear visual indicators; easy pause/disable |
| Too slow for simple tasks | Local AI must be <500ms; show "thinking" states |
| Generated code not matching style | Encode all standards in skills; iterate based on feedback |
| Interruption handling failures | Robust state management; clear "listening" vs "speaking" modes |

---

## Competitive Analysis

| Solution | Faster | GitHub Copilot | Cursor | Tabnine |
|----------|--------|----------------|--------|---------|
| Voice input | ✅ Core feature | ❌ | ❌ | ❌ |
| Always listening | ✅ Local AI | ❌ | ❌ | ❌ |
| Privacy-first | ✅ Two-tier | ❌ All cloud | ❌ All cloud | ⚠️ Hybrid |
| Skill system | ✅ Custom skills | ❌ | ⚠️ Limited | ❌ |
| Scaffolding | ✅ Full stack | ❌ | ⚠️ Basic | ❌ |
| Local fallback | ✅ 90% local | ❌ | ❌ | ⚠️ Some |
| Cost | Low (mostly local) | $$$ | $$$ | $$ |

**Unique Value Props:**
1. Only voice-first solution for Claude Code
2. Only two-tier architecture (privacy + power)
3. Only solution with custom skill scaffolding
4. Lowest cost (90% operations are free/local)

---

## Conclusion

**Faster** transforms development from manual scaffolding to natural conversation:

- **Talk through ideas** with always-listening local AI
- **Trigger automation** with voice-activated skills
- **Protect privacy** with local-first architecture
- **Save time** by eliminating 24 hours/month of boilerplate

The two-tier system gives you the best of both worlds: instant, private responses for simple queries, and powerful Claude Code reasoning for complex tasks.

**Next Steps:**
1. Review and approve this proposal
2. Build Phase 1 (foundation + first two skills)
3. Test with real workflow
4. Iterate and expand

---

*Proposal by: Jordan Partridge*
*Date: January 4, 2026*
*Version: 1.0*
