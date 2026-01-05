# Claude Provider Integration Summary

## Quick Reference

### Claude CLI Command Structure

```bash
# Basic print mode (non-interactive)
claude -p "prompt" --output-format json

# Streaming mode (real-time events)
claude -p "prompt" --output-format stream-json --verbose

# With session management
claude -p "prompt" --resume "session-id"
claude -p "prompt" --continue  # Resume last session
claude -p "prompt" --resume "session-id" --fork-session

# With tool restrictions
claude -p "prompt" --allowedTools "Read,Grep,Glob"
claude -p "prompt" --disallowedTools "Bash,Write"

# With budget limit
claude -p "prompt" --max-budget-usd 0.50

# With custom model
claude -p "prompt" --model "opus"
```

### Key JSON Output Fields

```json
{
  "type": "result",
  "subtype": "success",
  "is_error": false,
  "result": "The actual response text",
  "session_id": "abc-123-def-456",
  "duration_ms": 3653,
  "duration_api_ms": 12735,
  "num_turns": 1,
  "total_cost_usd": 0.081484,
  "usage": {
    "input_tokens": 2,
    "output_tokens": 13,
    "cache_read_input_tokens": 17680,
    "cache_creation_input_tokens": 14097
  },
  "modelUsage": {
    "claude-sonnet-4-5-20250929": {
      "inputTokens": 4,
      "outputTokens": 162,
      "costUSD": 0.076401
    }
  }
}
```

### Stream JSON Event Types

```json
// 1. System initialization
{"type": "system", "subtype": "init", "session_id": "...", "tools": [...], "model": "..."}

// 2. Hook responses (from SessionStart, etc)
{"type": "system", "subtype": "hook_response", "hook_name": "SessionStart:startup", ...}

// 3. Assistant messages (incremental)
{"type": "assistant", "message": {"content": [{"type": "text", "text": "..."}]}, ...}

// 4. Final result
{"type": "result", "subtype": "success", "result": "...", "total_cost_usd": 0.063, ...}
```

## PHP Implementation Patterns

### Basic Execution

```php
use Illuminate\Support\Facades\Process;

$result = Process::run([
    'claude',
    '-p',
    'What is 2+2?',
    '--output-format',
    'json'
]);

$data = json_decode($result->output(), true);
echo $data['result'];
// Output: 2 + 2 = 4
```

### Streaming with Callbacks

```php
$buffer = '';
$events = [];

$result = Process::run(
    ['claude', '-p', 'List 5 colors', '--output-format', 'stream-json', '--verbose'],
    function (string $type, string $output) use (&$buffer, &$events) {
        if ($type !== 'out') {
            return;
        }

        $buffer .= $output;
        $lines = explode("\n", $buffer);
        $buffer = array_pop($lines); // Keep incomplete line

        foreach ($lines as $line) {
            if (empty(trim($line))) {
                continue;
            }

            $event = json_decode($line, true);
            $events[] = $event;

            // Handle different event types
            if ($event['type'] === 'assistant') {
                $text = $event['message']['content'][0]['text'] ?? '';
                echo "Assistant: $text\n";
            }

            if ($event['type'] === 'result') {
                echo "Final: {$event['result']}\n";
                echo "Cost: \${$event['total_cost_usd']}\n";
            }
        }
    }
);
```

### With Working Directory Context

```php
use Illuminate\Support\Facades\Process;

$result = Process::path('/path/to/project')
    ->run([
        'claude',
        '-p',
        'Analyze this codebase',
        '--output-format',
        'json',
        '--allowedTools',
        'Read,Grep,Glob'
    ]);
```

### Session Management

```php
// First request - capture session ID
$result1 = Process::run([
    'claude',
    '-p',
    'Review this code',
    '--output-format',
    'json'
]);

$data = json_decode($result1->output(), true);
$sessionId = $data['session_id'];

// Follow-up in same session
$result2 = Process::run([
    'claude',
    '-p',
    'Now add tests',
    '--resume',
    $sessionId,
    '--output-format',
    'json'
]);

// Fork for alternative approach
$result3 = Process::run([
    'claude',
    '-p',
    'Try a different approach',
    '--resume',
    $sessionId,
    '--fork-session',
    '--output-format',
    'json'
]);
```

## Error Detection

### Check Installation

```php
$installed = Process::run(['which', 'claude'])->successful();

if (!$installed) {
    throw new Exception('Claude Code not installed');
}
```

### Check Authentication

```php
$authenticated = Process::run(['claude', '--version'])->successful();

if (!$authenticated) {
    throw new Exception('Not authenticated - run: claude auth login');
}
```

### Parse Execution Errors

```php
$result = Process::run(['claude', '-p', 'test', '--output-format', 'json']);

if (!$result->successful()) {
    $exitCode = $result->exitCode();
    $error = $result->errorOutput();

    throw new Exception("Claude failed (exit $exitCode): $error");
}

$data = json_decode($result->output(), true);

if ($data['is_error'] ?? false) {
    throw new Exception("Claude execution error: {$data['result']}");
}
```

## Common Patterns

### Read-Only Analysis

```php
$context = [
    'prompt' => 'Analyze security vulnerabilities',
    'allowedTools' => ['Read', 'Grep', 'Glob'],
    'disallowedTools' => ['Bash', 'Write', 'Edit'],
];

$command = [
    'claude',
    '-p',
    $context['prompt'],
    '--allowedTools',
    implode(',', $context['allowedTools']),
    '--disallowedTools',
    implode(',', $context['disallowedTools']),
    '--output-format',
    'json',
];

$result = Process::path($projectPath)->run($command);
```

### Budget-Limited Execution

```php
$command = [
    'claude',
    '-p',
    'Generate API documentation',
    '--max-budget-usd',
    '0.25',
    '--output-format',
    'json',
];

$result = Process::run($command);
$data = json_decode($result->output(), true);

if ($data['total_cost_usd'] >= 0.25) {
    Log::warning('Claude execution hit budget limit', [
        'cost' => $data['total_cost_usd'],
    ]);
}
```

### Real-time Broadcasting

```php
use Illuminate\Support\Facades\Broadcast;

$result = Process::run(
    ['claude', '-p', $prompt, '--output-format', 'stream-json', '--verbose'],
    function (string $type, string $output) use ($channelName) {
        if ($type !== 'out') {
            return;
        }

        $lines = explode("\n", trim($output));

        foreach ($lines as $line) {
            if (empty($line)) {
                continue;
            }

            $event = json_decode($line, true);

            if ($event['type'] === 'assistant') {
                $text = $event['message']['content'][0]['text'] ?? '';

                Broadcast::channel($channelName)->send([
                    'type' => 'chunk',
                    'text' => $text,
                ]);
            }
        }
    }
);
```

## Cost Tracking

### Per-Request Tracking

```php
$result = Process::run(['claude', '-p', $prompt, '--output-format', 'json']);
$data = json_decode($result->output(), true);

DB::table('claude_usage')->insert([
    'session_id' => $data['session_id'],
    'prompt' => $prompt,
    'result' => $data['result'],
    'cost_usd' => $data['total_cost_usd'],
    'input_tokens' => $data['usage']['input_tokens'],
    'output_tokens' => $data['usage']['output_tokens'],
    'duration_ms' => $data['duration_ms'],
    'created_at' => now(),
]);
```

### Model-Specific Costs

```php
foreach ($data['modelUsage'] as $model => $usage) {
    Log::info("Model usage", [
        'model' => $model,
        'input_tokens' => $usage['inputTokens'],
        'output_tokens' => $usage['outputTokens'],
        'cost_usd' => $usage['costUSD'],
    ]);
}
```

## Testing

### Unit Test Example

```php
use Illuminate\Process\PendingProcess;
use Illuminate\Support\Facades\Process;

test('executes claude command successfully', function () {
    Process::fake([
        'claude*' => Process::result(
            output: json_encode([
                'type' => 'result',
                'subtype' => 'success',
                'result' => 'Test response',
                'session_id' => 'test-session',
                'is_error' => false,
                'total_cost_usd' => 0.01,
                'usage' => [
                    'input_tokens' => 10,
                    'output_tokens' => 5,
                ],
            ])
        ),
    ]);

    $provider = app(ClaudeProvider::class);
    $result = $provider->execute(PromptContext::make('test'));

    expect($result->result)->toBe('Test response');
    expect($result->sessionId)->toBe('test-session');
    expect($result->successful())->toBeTrue();
});
```

### Integration Test Example

```php
test('claude integration works', function () {
    $claude = app(ClaudeProvider::class);

    if (!$claude->isInstalled() || !$claude->isAuthenticated()) {
        $this->markTestSkipped('Claude not available');
    }

    $context = PromptContext::make('What is 2+2?')
        ->withBudget(0.10);

    $result = $claude->execute($context);

    expect($result->successful())->toBeTrue();
    expect($result->result)->toContain('4');
    expect($result->totalCostUsd)->toBeLessThan(0.10);
});
```

## Performance Considerations

### Caching Results

```php
use Illuminate\Support\Facades\Cache;

$cacheKey = 'claude:' . md5($prompt);

$result = Cache::remember($cacheKey, now()->addHour(), function () use ($prompt) {
    $result = Process::run([
        'claude',
        '-p',
        $prompt,
        '--output-format',
        'json',
    ]);

    return json_decode($result->output(), true);
});
```

### Async Execution with Queues

```php
use Illuminate\Bus\Queueable;
use Illuminate\Contracts\Queue\ShouldQueue;

class ProcessClaudePrompt implements ShouldQueue
{
    use Queueable;

    public function __construct(
        public string $prompt,
        public ?string $sessionId = null,
    ) {}

    public function handle(ClaudeProvider $claude): void
    {
        $context = PromptContext::make($this->prompt);

        if ($this->sessionId) {
            $context = $context->withSession($this->sessionId);
        }

        $result = $claude->execute($context);

        // Store result, broadcast event, etc.
        event(new ClaudeExecutionCompleted($result));
    }
}

// Dispatch
ProcessClaudePrompt::dispatch('Analyze this codebase');
```

### Timeouts

```php
$result = Process::timeout(300) // 5 minutes
    ->run(['claude', '-p', $longRunningPrompt, '--output-format', 'json']);
```

## Security Considerations

1. **Never execute untrusted prompts** - Always validate/sanitize user input
2. **Use tool restrictions** - Limit to read-only tools when possible
3. **Set budget limits** - Use `--max-budget-usd` to prevent runaway costs
4. **Validate working directory** - Ensure Claude only accesses intended directories
5. **Log all executions** - Track what prompts are being executed and by whom
6. **Rate limiting** - Implement rate limits on Claude executions per user/tenant

## Troubleshooting

### Issue: "command not found: claude"
- **Cause**: Claude Code not installed
- **Fix**: Install from https://claude.com/product/claude-code

### Issue: Authentication errors
- **Cause**: Not logged in
- **Fix**: Run `claude auth login`

### Issue: "Error: When using --print, --output-format=stream-json requires --verbose"
- **Cause**: Missing --verbose flag
- **Fix**: Add `--verbose` when using `--output-format stream-json`

### Issue: JSON parsing errors
- **Cause**: Claude output contains non-JSON text (hooks, errors)
- **Fix**: Filter stderr, handle hooks, validate JSON before parsing

### Issue: High costs
- **Cause**: Large context, long conversations
- **Fix**: Use `--max-budget-usd`, limit tools, use smaller models

### Issue: Timeout errors
- **Cause**: Long-running operations
- **Fix**: Increase Process timeout, use async execution
