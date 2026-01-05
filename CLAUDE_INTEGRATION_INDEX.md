# Claude Code CLI Integration - Documentation Index

## Overview

This documentation provides comprehensive research and design for integrating Claude Code CLI with Laravel/PHP applications through a `ClaudeProvider` contract.

## Documents

### 1. CLAUDE_PROVIDER_DESIGN.md (34K)
**Complete technical specification and architecture**

Contains:
- Research findings on Claude CLI authentication, session management, and command structure
- Detailed PHP interface design for `ClaudeProvider` contract
- Complete value object definitions (PromptContext, ClaudeResult, etc.)
- Full implementation using Symfony Process / Laravel Process facade
- Exception hierarchy
- Observable events
- Service provider registration
- Architecture decisions and rationale
- Implementation checklist
- Comprehensive references and sources

**Use this for**: Complete understanding of the design, implementation details, and architecture decisions.

### 2. CLAUDE_PROVIDER_SUMMARY.md (11K)
**Quick reference and practical patterns**

Contains:
- Claude CLI command structure examples
- JSON output format reference
- Stream JSON event types
- PHP implementation patterns (basic, streaming, sessions)
- Error detection strategies
- Common usage patterns (read-only, budget-limited, broadcasting)
- Cost tracking examples
- Testing examples (unit and integration)
- Performance considerations (caching, async, timeouts)
- Security considerations
- Troubleshooting guide

**Use this for**: Quick reference during development, copy-paste examples, troubleshooting.

### 3. CLAUDE_PROVIDER_EXAMPLE.php (15K)
**Real-world implementation examples**

Contains 10 practical examples:
1. Basic code analysis
2. Generate tests with streaming progress
3. Multi-turn conversation for code review
4. Cached analysis to avoid redundant executions
5. Real-time broadcasting for collaborative sessions
6. Budget-aware batch processing
7. Usage tracking and analytics
8. Alternative approaches using session forking
9. Health check and diagnostics
10. Graceful degradation and fallbacks

**Use this for**: Copy-paste implementation patterns, understanding real-world usage.

## Quick Start

### 1. Check if Claude is Available

```php
use App\Conduit\Contracts\ClaudeProvider;

$claude = app(ClaudeProvider::class);

if (!$claude->isInstalled()) {
    // Install from https://claude.com/product/claude-code
}

if (!$claude->isAuthenticated()) {
    // Run: claude auth login
}
```

### 2. Simple Execution

```php
use App\Conduit\DataTransferObjects\PromptContext;

$context = PromptContext::make('What is 2+2?');
$result = $claude->execute($context);

echo $result->result; // "2 + 2 = 4"
echo "Cost: $" . $result->totalCostUsd;
```

### 3. Streaming with Progress

```php
$context = PromptContext::make('Generate tests for this file');

$result = $claude->stream($context, function (ClaudeStreamEvent $event) {
    if ($event->isAssistant()) {
        echo $event->getMessage();
    }
});
```

## Key Features

### Authentication
- Uses Claude Code's built-in OAuth authentication
- Credentials stored in macOS Keychain
- No additional API keys required
- Check status: `claude auth status`

### Session Management
- Auto-created session IDs for each execution
- Resume sessions: `--resume "session-id"`
- Continue last session: `--continue`
- Fork sessions for alternatives: `--fork-session`

### Output Formats
- `text` - Plain text (default)
- `json` - Structured JSON with metadata, costs, usage
- `stream-json` - Real-time newline-delimited JSON events

### Cost Control
- `--max-budget-usd` - Hard limit on execution cost
- Token usage tracking (input, output, cache)
- Per-model cost breakdown
- Real-time cost reporting

### Tool Control
- `--allowedTools` - Whitelist specific tools
- `--disallowedTools` - Blacklist specific tools
- Example: Read-only mode: `['Read', 'Grep', 'Glob']`

### Context Management
- Working directory sets project context
- Reads CLAUDE.md for project-specific instructions
- `--add-dir` for additional directory access
- `--append-system-prompt` for custom instructions

## Architecture Highlights

### Immutable Value Objects
All DTOs are immutable with fluent builder methods:

```php
$context = PromptContext::make('Analyze code')
    ->withWorkingDirectory('/path/to/project')
    ->withTools(allowed: ['Read', 'Grep'])
    ->withBudget(0.50)
    ->withModel('opus');
```

### Exception Handling
Specific exceptions for different failure modes:
- `ClaudeNotInstalledException` - CLI not found
- `ClaudeAuthenticationException` - Not logged in
- `ClaudeExecutionException` - Execution failed

### Observable Events
- `ClaudeExecutionStarted`
- `ClaudeExecutionCompleted`
- `ClaudeExecutionFailed`
- `ClaudeStreamEventReceived`

### Laravel Integration
- Uses Laravel Process facade
- Compatible with Laravel 10, 11, 12
- Service provider for dependency injection
- Cache integration for result caching
- Queue integration for async execution
- Broadcasting for real-time updates

## Common Patterns

### Read-Only Analysis
```php
$context = PromptContext::make('Find security vulnerabilities')
    ->withWorkingDirectory($projectPath)
    ->withTools(
        allowed: ['Read', 'Grep', 'Glob'],
        disallowed: ['Bash', 'Write', 'Edit']
    );
```

### Budget-Limited
```php
$context = PromptContext::make('Generate docs')
    ->withBudget(0.25); // Max $0.25
```

### Multi-Turn Conversation
```php
$result1 = $claude->execute(PromptContext::make('Review code'));
$sessionId = $result1->sessionId;

$result2 = $claude->execute(
    PromptContext::make('Add tests')->withSession($sessionId)
);
```

### Real-Time Broadcasting
```php
$result = $claude->stream($context, function (ClaudeStreamEvent $event) {
    if ($event->isAssistant()) {
        broadcast(new MessageChunk($event->getMessage()));
    }
});
```

## Performance Optimization

### Caching
```php
$cacheKey = 'analysis:' . md5($filePath . filemtime($filePath));

$result = Cache::remember($cacheKey, 3600, function () use ($context, $claude) {
    return $claude->execute($context);
});
```

### Async Execution
```php
ProcessClaudePrompt::dispatch($prompt, $sessionId);
```

### Timeouts
```php
Process::timeout(300)->run($command); // 5 minutes
```

## Security Best Practices

1. **Validate Input**: Always sanitize user-provided prompts
2. **Restrict Tools**: Use `allowedTools` to limit capabilities
3. **Set Budgets**: Use `--max-budget-usd` to prevent runaway costs
4. **Validate Paths**: Ensure working directories are safe
5. **Log Executions**: Track all Claude executions for auditing
6. **Rate Limit**: Implement rate limits per user/tenant

## Testing

### Unit Tests
```php
Process::fake([
    'claude*' => Process::result(
        output: json_encode([...])
    ),
]);

$result = $claude->execute($context);
expect($result->successful())->toBeTrue();
```

### Integration Tests
```php
if (!$claude->isInstalled() || !$claude->isAuthenticated()) {
    $this->markTestSkipped('Claude not available');
}

$result = $claude->execute(PromptContext::make('Test'));
expect($result->successful())->toBeTrue();
```

## Cost Tracking

### Per-Request
```php
DB::table('claude_usage')->insert([
    'session_id' => $result->sessionId,
    'prompt' => $context->prompt,
    'cost_usd' => $result->totalCostUsd,
    'input_tokens' => $result->usage->inputTokens,
    'output_tokens' => $result->usage->outputTokens,
]);
```

### Monthly Aggregation
```php
DB::table('user_monthly_usage')
    ->where('user_id', $userId)
    ->where('month', now()->format('Y-m'))
    ->increment('total_cost_usd', $result->totalCostUsd);
```

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| "command not found: claude" | Not installed | Install from claude.com |
| Authentication errors | Not logged in | Run `claude auth login` |
| "requires --verbose" | Missing flag for streaming | Add `--verbose` with `stream-json` |
| JSON parsing errors | Non-JSON output | Filter stderr, validate JSON |
| High costs | Large context | Use `--max-budget-usd`, limit tools |
| Timeouts | Long operations | Increase timeout, use async |

## References

### Official Documentation
- [Claude Code CLI Reference](https://code.claude.com/docs/en/cli-reference)
- [Run Claude Code Programmatically](https://code.claude.com/docs/en/headless)
- [Identity and Access Management](https://code.claude.com/docs/en/iam)
- [Laravel Process Documentation](https://laravel.com/docs/12.x/processes)
- [Symfony Process Component](https://symfony.com/doc/current/components/process.html)

### Community Resources
- [Claude Code GitHub](https://github.com/anthropics/claude-code)
- [Print Mode Use Cases](https://github.com/anthropics/claude-code/issues/762)
- [Awesome Claude Code](https://github.com/hesreallyhim/awesome-claude-code)

### Blog Posts
- [Laravel's New Process Facade](https://beyondco.de/blog/laravel-10-new-process-facade)
- [Claude Code Cheat Sheet](https://shipyard.build/blog/claude-code-cheat-sheet/)
- [Running Commands with Laravel Process](https://fly.io/laravel-bytes/run-commands-with-laravel-process/)

## Next Steps

1. Review CLAUDE_PROVIDER_DESIGN.md for complete specifications
2. Use CLAUDE_PROVIDER_SUMMARY.md as quick reference during development
3. Copy patterns from CLAUDE_PROVIDER_EXAMPLE.php for implementation
4. Implement the contracts and DTOs
5. Write tests (unit and integration)
6. Add logging and monitoring
7. Deploy and iterate

## Questions?

This documentation is comprehensive but if you need clarification on any aspect:
- Check the detailed design document for architectural decisions
- Review the examples for practical implementation patterns
- Consult the summary for quick reference and troubleshooting
