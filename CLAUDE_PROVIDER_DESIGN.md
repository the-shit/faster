# Claude Provider Design for Laravel/PHP Integration

## Research Summary

### 1. How Claude Code CLI Works

#### Authentication
- **Stored Credentials**: Claude Code securely stores authentication in macOS Keychain (on macOS)
- **OAuth Flow**: Uses OAuth requiring a browser for initial authentication
- **Auth Commands**:
  - `claude auth status` - Check authentication status
  - `claude auth login` - Initiate login flow
  - `claude auth logout` - Clear credentials
- **API Key Helper**: Supports custom credential scripts via `apiKeyHelper` setting
- **Session-based**: Once authenticated via `claude login`, all subsequent CLI calls use stored credentials

#### Session Management
- **Auto-creation**: Each query creates a session with a unique session ID
- **Session Storage**: Saved in `$HOME/.claude` as line-delimited JSON
- **Resume Options**:
  - `claude -c` or `claude --continue` - Resume most recent conversation
  - `claude -r "session-id"` - Resume specific session
  - `--fork-session` - Create new session from existing one
- **Context Persistence**: CLAUDE.md file provides project-specific context for all sessions

#### CLI Modes
- **Interactive Mode** (default): REPL-style conversation
- **Print Mode** (`-p` or `--print`): Non-interactive, single query execution
- **Output Formats**:
  - `text` - Plain text (default)
  - `json` - Single JSON result with metadata
  - `stream-json` - Real-time newline-delimited JSON (requires `--verbose`)

### 2. Calling Claude from PHP/Laravel

The recommended approach is using Laravel's Process facade (Laravel 10+) or Symfony Process component:

```php
use Illuminate\Support\Facades\Process;

// Simple execution
$result = Process::run('claude -p "What is 2+2?" --output-format json');

// With streaming callback
$result = Process::run('claude -p "Analyze code" --output-format stream-json --verbose',
    function(string $type, string $output) {
        if ($type === 'out') {
            // Handle real-time output chunks
            $data = json_decode($output, true);
        }
    }
);
```

### 3. Streaming Responses

Claude Code supports streaming via `--output-format stream-json --verbose`:

**Stream Event Types**:
```json
{"type": "system", "subtype": "init", ...}
{"type": "system", "subtype": "hook_response", ...}
{"type": "assistant", "message": {...}, ...}
{"type": "result", "subtype": "success", "result": "...", ...}
```

**Key Fields**:
- `type`: Event type (system, assistant, result)
- `subtype`: Event subtype (init, hook_response, success, error)
- `session_id`: Session identifier
- `result`: Final result text
- `usage`: Token usage and cost information
- `total_cost_usd`: Cost in USD

### 4. Passing Context

**Directory Context**:
```bash
cd /path/to/project && claude -p "prompt"
```

**File Context** (via system prompt):
```bash
claude -p "Analyze this" --append-system-prompt "$(cat file.txt)"
```

**Session Context**:
```bash
# Resume existing session with full context
claude -p "Continue analysis" --resume "session-id"
```

**Additional Directories**:
```bash
claude -p "prompt" --add-dir /path/to/dir1 --add-dir /path/to/dir2
```

**Tool Restrictions**:
```bash
# Only allow specific tools
claude -p "prompt" --allowedTools "Read,Grep"

# Disable specific tools
claude -p "prompt" --disallowedTools "Bash,Write"
```

### 5. Error Handling

**Common Error Scenarios**:

1. **Claude Not Installed**: Exit code 127 or "command not found"
2. **Not Authenticated**: Exit code 1 with authentication error message
3. **Session Not Found**: Error when resuming non-existent session
4. **Permission Denied**: Tool permission errors
5. **Budget Exceeded**: When using `--max-budget-usd`
6. **API Errors**: Rate limits, overloaded models

**Detection Strategy**:
```php
// Check if CLI exists
$exists = Process::run('which claude')->successful();

// Check authentication
$authenticated = Process::run('claude --version')->successful();

// Parse JSON errors
$result = Process::run('claude -p "test" --output-format json');
$data = json_decode($result->output(), true);
if ($data['is_error'] ?? false) {
    // Handle error
}
```

---

## PHP Interface Design

### Core Contracts

```php
<?php

namespace App\Conduit\Contracts;

interface ClaudeProvider
{
    /**
     * Execute a prompt with Claude Code CLI
     *
     * @param PromptContext $context
     * @return ClaudeResult
     * @throws ClaudeNotInstalledException
     * @throws ClaudeAuthenticationException
     * @throws ClaudeExecutionException
     */
    public function execute(PromptContext $context): ClaudeResult;

    /**
     * Execute a prompt with streaming responses
     *
     * @param PromptContext $context
     * @param callable(ClaudeStreamEvent): void $callback
     * @return ClaudeResult
     */
    public function stream(PromptContext $context, callable $callback): ClaudeResult;

    /**
     * Check if Claude Code CLI is authenticated
     *
     * @return bool
     */
    public function isAuthenticated(): bool;

    /**
     * Check if Claude Code CLI is installed
     *
     * @return bool
     */
    public function isInstalled(): bool;

    /**
     * Get current session information
     *
     * @param string|null $sessionId
     * @return SessionInfo
     */
    public function getSessionInfo(?string $sessionId = null): SessionInfo;

    /**
     * Resume a previous session
     *
     * @param string $sessionId
     * @return SessionContext
     */
    public function resumeSession(string $sessionId): SessionContext;

    /**
     * Get CLI version information
     *
     * @return string
     */
    public function getVersion(): string;
}
```

### Value Objects

```php
<?php

namespace App\Conduit\DataTransferObjects;

use Illuminate\Support\Collection;

/**
 * Context for executing a Claude prompt
 */
class PromptContext
{
    public function __construct(
        public readonly string $prompt,
        public readonly ?string $workingDirectory = null,
        public readonly ?string $sessionId = null,
        public readonly ?string $systemPrompt = null,
        public readonly ?array $allowedTools = null,
        public readonly ?array $disallowedTools = null,
        public readonly ?float $maxBudgetUsd = null,
        public readonly bool $continueSession = false,
        public readonly bool $forkSession = false,
        public readonly array $additionalDirectories = [],
        public readonly array $mcpConfig = [],
        public readonly ?string $model = null,
        public readonly OutputFormat $outputFormat = OutputFormat::JSON,
    ) {}

    public static function make(string $prompt): self
    {
        return new self(prompt: $prompt);
    }

    public function withWorkingDirectory(string $directory): self
    {
        return new self(
            prompt: $this->prompt,
            workingDirectory: $directory,
            sessionId: $this->sessionId,
            systemPrompt: $this->systemPrompt,
            allowedTools: $this->allowedTools,
            disallowedTools: $this->disallowedTools,
            maxBudgetUsd: $this->maxBudgetUsd,
            continueSession: $this->continueSession,
            forkSession: $this->forkSession,
            additionalDirectories: $this->additionalDirectories,
            mcpConfig: $this->mcpConfig,
            model: $this->model,
            outputFormat: $this->outputFormat,
        );
    }

    public function withSession(string $sessionId, bool $fork = false): self
    {
        return new self(
            prompt: $this->prompt,
            workingDirectory: $this->workingDirectory,
            sessionId: $sessionId,
            systemPrompt: $this->systemPrompt,
            allowedTools: $this->allowedTools,
            disallowedTools: $this->disallowedTools,
            maxBudgetUsd: $this->maxBudgetUsd,
            continueSession: false,
            forkSession: $fork,
            additionalDirectories: $this->additionalDirectories,
            mcpConfig: $this->mcpConfig,
            model: $this->model,
            outputFormat: $this->outputFormat,
        );
    }

    public function withSystemPrompt(string $systemPrompt): self
    {
        return new self(
            prompt: $this->prompt,
            workingDirectory: $this->workingDirectory,
            sessionId: $this->sessionId,
            systemPrompt: $systemPrompt,
            allowedTools: $this->allowedTools,
            disallowedTools: $this->disallowedTools,
            maxBudgetUsd: $this->maxBudgetUsd,
            continueSession: $this->continueSession,
            forkSession: $this->forkSession,
            additionalDirectories: $this->additionalDirectories,
            mcpConfig: $this->mcpConfig,
            model: $this->model,
            outputFormat: $this->outputFormat,
        );
    }

    public function withTools(?array $allowed = null, ?array $disallowed = null): self
    {
        return new self(
            prompt: $this->prompt,
            workingDirectory: $this->workingDirectory,
            sessionId: $this->sessionId,
            systemPrompt: $this->systemPrompt,
            allowedTools: $allowed,
            disallowedTools: $disallowed,
            maxBudgetUsd: $this->maxBudgetUsd,
            continueSession: $this->continueSession,
            forkSession: $this->forkSession,
            additionalDirectories: $this->additionalDirectories,
            mcpConfig: $this->mcpConfig,
            model: $this->model,
            outputFormat: $this->outputFormat,
        );
    }

    public function withBudget(float $maxBudgetUsd): self
    {
        return new self(
            prompt: $this->prompt,
            workingDirectory: $this->workingDirectory,
            sessionId: $this->sessionId,
            systemPrompt: $this->systemPrompt,
            allowedTools: $this->allowedTools,
            disallowedTools: $this->disallowedTools,
            maxBudgetUsd: $maxBudgetUsd,
            continueSession: $this->continueSession,
            forkSession: $this->forkSession,
            additionalDirectories: $this->additionalDirectories,
            mcpConfig: $this->mcpConfig,
            model: $this->model,
            outputFormat: $this->outputFormat,
        );
    }

    public function withModel(string $model): self
    {
        return new self(
            prompt: $this->prompt,
            workingDirectory: $this->workingDirectory,
            sessionId: $this->sessionId,
            systemPrompt: $this->systemPrompt,
            allowedTools: $this->allowedTools,
            disallowedTools: $this->disallowedTools,
            maxBudgetUsd: $this->maxBudgetUsd,
            continueSession: $this->continueSession,
            forkSession: $this->forkSession,
            additionalDirectories: $this->additionalDirectories,
            mcpConfig: $this->mcpConfig,
            model: $model,
            outputFormat: $this->outputFormat,
        );
    }

    public function asStreaming(): self
    {
        return new self(
            prompt: $this->prompt,
            workingDirectory: $this->workingDirectory,
            sessionId: $this->sessionId,
            systemPrompt: $this->systemPrompt,
            allowedTools: $this->allowedTools,
            disallowedTools: $this->disallowedTools,
            maxBudgetUsd: $this->maxBudgetUsd,
            continueSession: $this->continueSession,
            forkSession: $this->forkSession,
            additionalDirectories: $this->additionalDirectories,
            mcpConfig: $this->mcpConfig,
            model: $this->model,
            outputFormat: OutputFormat::STREAM_JSON,
        );
    }

    public function buildCommand(): array
    {
        $command = ['claude', '-p', $this->prompt];

        if ($this->outputFormat === OutputFormat::STREAM_JSON) {
            $command[] = '--output-format';
            $command[] = 'stream-json';
            $command[] = '--verbose';
        } elseif ($this->outputFormat === OutputFormat::JSON) {
            $command[] = '--output-format';
            $command[] = 'json';
        }

        if ($this->sessionId) {
            $command[] = '--resume';
            $command[] = $this->sessionId;

            if ($this->forkSession) {
                $command[] = '--fork-session';
            }
        } elseif ($this->continueSession) {
            $command[] = '--continue';
        }

        if ($this->systemPrompt) {
            $command[] = '--append-system-prompt';
            $command[] = $this->systemPrompt;
        }

        if ($this->allowedTools) {
            $command[] = '--allowedTools';
            $command[] = implode(',', $this->allowedTools);
        }

        if ($this->disallowedTools) {
            $command[] = '--disallowedTools';
            $command[] = implode(',', $this->disallowedTools);
        }

        if ($this->maxBudgetUsd) {
            $command[] = '--max-budget-usd';
            $command[] = (string) $this->maxBudgetUsd;
        }

        if ($this->model) {
            $command[] = '--model';
            $command[] = $this->model;
        }

        foreach ($this->additionalDirectories as $dir) {
            $command[] = '--add-dir';
            $command[] = $dir;
        }

        if ($this->mcpConfig) {
            foreach ($this->mcpConfig as $config) {
                $command[] = '--mcp-config';
                $command[] = is_array($config) ? json_encode($config) : $config;
            }
        }

        return $command;
    }
}

/**
 * Result from Claude Code execution
 */
class ClaudeResult
{
    public function __construct(
        public readonly string $result,
        public readonly string $sessionId,
        public readonly bool $isError,
        public readonly int $durationMs,
        public readonly int $durationApiMs,
        public readonly int $numTurns,
        public readonly float $totalCostUsd,
        public readonly TokenUsage $usage,
        public readonly Collection $modelUsage,
        public readonly array $permissionDenials,
        public readonly ?string $errorMessage = null,
        public readonly ?int $exitCode = null,
    ) {}

    public static function fromJson(array $data, ?int $exitCode = null): self
    {
        return new self(
            result: $data['result'] ?? '',
            sessionId: $data['session_id'] ?? '',
            isError: $data['is_error'] ?? ($exitCode !== 0),
            durationMs: $data['duration_ms'] ?? 0,
            durationApiMs: $data['duration_api_ms'] ?? 0,
            numTurns: $data['num_turns'] ?? 0,
            totalCostUsd: $data['total_cost_usd'] ?? 0.0,
            usage: TokenUsage::fromArray($data['usage'] ?? []),
            modelUsage: collect($data['modelUsage'] ?? [])->map(
                fn($usage, $model) => ModelUsage::fromArray($model, $usage)
            ),
            permissionDenials: $data['permission_denials'] ?? [],
            errorMessage: $data['error_message'] ?? null,
            exitCode: $exitCode,
        );
    }

    public function successful(): bool
    {
        return !$this->isError && ($this->exitCode === null || $this->exitCode === 0);
    }

    public function failed(): bool
    {
        return !$this->successful();
    }
}

/**
 * Token usage information
 */
class TokenUsage
{
    public function __construct(
        public readonly int $inputTokens,
        public readonly int $outputTokens,
        public readonly int $cacheReadInputTokens,
        public readonly int $cacheCreationInputTokens,
        public readonly array $cacheCreation,
        public readonly string $serviceTier,
    ) {}

    public static function fromArray(array $data): self
    {
        return new self(
            inputTokens: $data['input_tokens'] ?? 0,
            outputTokens: $data['output_tokens'] ?? 0,
            cacheReadInputTokens: $data['cache_read_input_tokens'] ?? 0,
            cacheCreationInputTokens: $data['cache_creation_input_tokens'] ?? 0,
            cacheCreation: $data['cache_creation'] ?? [],
            serviceTier: $data['service_tier'] ?? 'standard',
        );
    }

    public function totalTokens(): int
    {
        return $this->inputTokens + $this->outputTokens;
    }
}

/**
 * Per-model usage information
 */
class ModelUsage
{
    public function __construct(
        public readonly string $model,
        public readonly int $inputTokens,
        public readonly int $outputTokens,
        public readonly int $cacheReadInputTokens,
        public readonly int $cacheCreationInputTokens,
        public readonly int $webSearchRequests,
        public readonly float $costUsd,
        public readonly int $contextWindow,
    ) {}

    public static function fromArray(string $model, array $data): self
    {
        return new self(
            model: $model,
            inputTokens: $data['inputTokens'] ?? 0,
            outputTokens: $data['outputTokens'] ?? 0,
            cacheReadInputTokens: $data['cacheReadInputTokens'] ?? 0,
            cacheCreationInputTokens: $data['cacheCreationInputTokens'] ?? 0,
            webSearchRequests: $data['webSearchRequests'] ?? 0,
            costUsd: $data['costUSD'] ?? 0.0,
            contextWindow: $data['contextWindow'] ?? 0,
        );
    }
}

/**
 * Session information
 */
class SessionInfo
{
    public function __construct(
        public readonly string $sessionId,
        public readonly string $cwd,
        public readonly array $tools,
        public readonly string $model,
        public readonly ?string $createdAt = null,
        public readonly ?string $updatedAt = null,
    ) {}
}

/**
 * Session context for resuming
 */
class SessionContext
{
    public function __construct(
        public readonly string $sessionId,
        public readonly array $messages,
        public readonly array $metadata,
    ) {}
}

/**
 * Streaming event from Claude Code
 */
class ClaudeStreamEvent
{
    public function __construct(
        public readonly string $type,
        public readonly ?string $subtype,
        public readonly array $data,
        public readonly string $raw,
    ) {}

    public static function fromJson(string $json): self
    {
        $data = json_decode($json, true);

        return new self(
            type: $data['type'] ?? 'unknown',
            subtype: $data['subtype'] ?? null,
            data: $data,
            raw: $json,
        );
    }

    public function isSystem(): bool
    {
        return $this->type === 'system';
    }

    public function isAssistant(): bool
    {
        return $this->type === 'assistant';
    }

    public function isResult(): bool
    {
        return $this->type === 'result';
    }

    public function isInit(): bool
    {
        return $this->type === 'system' && $this->subtype === 'init';
    }

    public function isSuccess(): bool
    {
        return $this->type === 'result' && $this->subtype === 'success';
    }

    public function isError(): bool
    {
        return $this->type === 'result' && $this->subtype === 'error';
    }

    public function getMessage(): ?string
    {
        if ($this->isAssistant()) {
            return $this->data['message']['content'][0]['text'] ?? null;
        }

        if ($this->isResult()) {
            return $this->data['result'] ?? null;
        }

        return null;
    }

    public function getSessionId(): ?string
    {
        return $this->data['session_id'] ?? null;
    }
}

/**
 * Output format enum
 */
enum OutputFormat: string
{
    case TEXT = 'text';
    case JSON = 'json';
    case STREAM_JSON = 'stream-json';
}
```

### Implementation

```php
<?php

namespace App\Conduit\Services;

use App\Conduit\Contracts\ClaudeProvider;
use App\Conduit\DataTransferObjects\{
    ClaudeResult,
    ClaudeStreamEvent,
    PromptContext,
    SessionInfo,
    SessionContext
};
use App\Conduit\Exceptions\{
    ClaudeAuthenticationException,
    ClaudeExecutionException,
    ClaudeNotInstalledException
};
use Illuminate\Process\Factory as ProcessFactory;
use Illuminate\Support\Facades\Process;
use Psr\Log\LoggerInterface;

class ClaudeCodeProvider implements ClaudeProvider
{
    private const CLAUDE_BINARY = 'claude';
    private const SESSION_DIR = '~/.claude';

    public function __construct(
        private readonly ProcessFactory $process,
        private readonly LoggerInterface $logger,
    ) {}

    public function execute(PromptContext $context): ClaudeResult
    {
        $this->ensureInstalled();
        $this->ensureAuthenticated();

        $command = $context->buildCommand();
        $workingDir = $context->workingDirectory ?? getcwd();

        $this->logger->debug('Executing Claude command', [
            'command' => implode(' ', $command),
            'working_directory' => $workingDir,
        ]);

        $result = $this->process
            ->path($workingDir)
            ->run($command);

        if (!$result->successful()) {
            throw new ClaudeExecutionException(
                "Claude execution failed: {$result->errorOutput()}",
                $result->exitCode()
            );
        }

        $output = $result->output();

        // Parse JSON output
        $data = json_decode($output, true);

        if (json_last_error() !== JSON_ERROR_NONE) {
            throw new ClaudeExecutionException(
                "Failed to parse Claude output: " . json_last_error_msg()
            );
        }

        $claudeResult = ClaudeResult::fromJson($data, $result->exitCode());

        $this->logger->info('Claude execution completed', [
            'session_id' => $claudeResult->sessionId,
            'duration_ms' => $claudeResult->durationMs,
            'cost_usd' => $claudeResult->totalCostUsd,
            'success' => $claudeResult->successful(),
        ]);

        if ($claudeResult->failed()) {
            throw new ClaudeExecutionException(
                $claudeResult->errorMessage ?? "Claude execution failed",
                $claudeResult->exitCode ?? 1
            );
        }

        return $claudeResult;
    }

    public function stream(PromptContext $context, callable $callback): ClaudeResult
    {
        $this->ensureInstalled();
        $this->ensureAuthenticated();

        $streamContext = $context->asStreaming();
        $command = $streamContext->buildCommand();
        $workingDir = $streamContext->workingDirectory ?? getcwd();

        $this->logger->debug('Streaming Claude command', [
            'command' => implode(' ', $command),
            'working_directory' => $workingDir,
        ]);

        $outputBuffer = '';
        $lastResult = null;

        $result = $this->process
            ->path($workingDir)
            ->run(
                $command,
                function (string $type, string $output) use ($callback, &$outputBuffer, &$lastResult) {
                    if ($type !== 'out') {
                        return;
                    }

                    // Stream JSON is newline-delimited
                    $outputBuffer .= $output;
                    $lines = explode("\n", $outputBuffer);

                    // Keep last incomplete line in buffer
                    $outputBuffer = array_pop($lines);

                    foreach ($lines as $line) {
                        $line = trim($line);
                        if (empty($line)) {
                            continue;
                        }

                        try {
                            $event = ClaudeStreamEvent::fromJson($line);

                            // Store result event for final return
                            if ($event->isResult()) {
                                $lastResult = $event->data;
                            }

                            $callback($event);
                        } catch (\JsonException $e) {
                            $this->logger->warning('Failed to parse stream event', [
                                'line' => $line,
                                'error' => $e->getMessage(),
                            ]);
                        }
                    }
                }
            );

        if (!$result->successful()) {
            throw new ClaudeExecutionException(
                "Claude streaming failed: {$result->errorOutput()}",
                $result->exitCode()
            );
        }

        // Return the final result
        if ($lastResult) {
            return ClaudeResult::fromJson($lastResult, $result->exitCode());
        }

        throw new ClaudeExecutionException(
            "No result received from Claude streaming"
        );
    }

    public function isAuthenticated(): bool
    {
        try {
            $result = $this->process->run([self::CLAUDE_BINARY, '--version']);
            return $result->successful();
        } catch (\Exception $e) {
            return false;
        }
    }

    public function isInstalled(): bool
    {
        try {
            $result = $this->process->run(['which', self::CLAUDE_BINARY]);
            return $result->successful();
        } catch (\Exception $e) {
            return false;
        }
    }

    public function getSessionInfo(?string $sessionId = null): SessionInfo
    {
        // This would need to parse the session files in ~/.claude
        // For now, return a basic implementation
        throw new \RuntimeException('Not yet implemented');
    }

    public function resumeSession(string $sessionId): SessionContext
    {
        // This would need to parse the session file
        throw new \RuntimeException('Not yet implemented');
    }

    public function getVersion(): string
    {
        $this->ensureInstalled();

        $result = $this->process->run([self::CLAUDE_BINARY, '--version']);

        if (!$result->successful()) {
            throw new ClaudeExecutionException(
                "Failed to get Claude version: {$result->errorOutput()}"
            );
        }

        return trim($result->output());
    }

    private function ensureInstalled(): void
    {
        if (!$this->isInstalled()) {
            throw new ClaudeNotInstalledException(
                'Claude Code CLI is not installed. Please install it from https://claude.com/product/claude-code'
            );
        }
    }

    private function ensureAuthenticated(): void
    {
        if (!$this->isAuthenticated()) {
            throw new ClaudeAuthenticationException(
                'Not authenticated with Claude Code. Please run: claude auth login'
            );
        }
    }
}
```

### Exceptions

```php
<?php

namespace App\Conduit\Exceptions;

class ClaudeException extends \RuntimeException {}

class ClaudeNotInstalledException extends ClaudeException {}

class ClaudeAuthenticationException extends ClaudeException {}

class ClaudeExecutionException extends ClaudeException {}
```

### Observable Events

```php
<?php

namespace App\Conduit\Events;

use App\Conduit\DataTransferObjects\ClaudeResult;
use App\Conduit\DataTransferObjects\ClaudeStreamEvent;
use Illuminate\Foundation\Events\Dispatchable;
use Illuminate\Queue\SerializesModels;

class ClaudeExecutionStarted
{
    use Dispatchable, SerializesModels;

    public function __construct(
        public readonly string $prompt,
        public readonly ?string $sessionId,
    ) {}
}

class ClaudeExecutionCompleted
{
    use Dispatchable, SerializesModels;

    public function __construct(
        public readonly ClaudeResult $result,
    ) {}
}

class ClaudeExecutionFailed
{
    use Dispatchable, SerializesModels;

    public function __construct(
        public readonly \Throwable $exception,
        public readonly string $prompt,
    ) {}
}

class ClaudeStreamEventReceived
{
    use Dispatchable, SerializesModels;

    public function __construct(
        public readonly ClaudeStreamEvent $event,
    ) {}
}
```

### Service Provider

```php
<?php

namespace App\Conduit\Providers;

use App\Conduit\Contracts\ClaudeProvider;
use App\Conduit\Services\ClaudeCodeProvider;
use Illuminate\Support\ServiceProvider;

class ClaudeServiceProvider extends ServiceProvider
{
    public function register(): void
    {
        $this->app->singleton(ClaudeProvider::class, ClaudeCodeProvider::class);
    }

    public function boot(): void
    {
        //
    }
}
```

---

## Usage Examples

### Basic Execution

```php
use App\Conduit\Contracts\ClaudeProvider;
use App\Conduit\DataTransferObjects\PromptContext;

$claude = app(ClaudeProvider::class);

$context = PromptContext::make('Analyze this codebase')
    ->withWorkingDirectory('/path/to/project')
    ->withTools(allowedTools: ['Read', 'Grep', 'Glob'])
    ->withBudget(0.50);

$result = $claude->execute($context);

echo $result->result;
echo "Cost: $" . $result->totalCostUsd;
echo "Session: " . $result->sessionId;
```

### Streaming Execution

```php
use App\Conduit\DataTransferObjects\ClaudeStreamEvent;

$context = PromptContext::make('Generate comprehensive tests')
    ->withWorkingDirectory('/path/to/project');

$result = $claude->stream($context, function (ClaudeStreamEvent $event) {
    if ($event->isAssistant()) {
        // Stream to user via websocket
        broadcast(new ClaudeMessageReceived($event->getMessage()));
    }

    if ($event->isSystem() && $event->subtype === 'tool_use') {
        Log::info('Claude used tool', ['tool' => $event->data['tool'] ?? null]);
    }
});
```

### Session Management

```php
// Initial execution
$context = PromptContext::make('Review this code');
$result = $claude->execute($context);
$sessionId = $result->sessionId;

// Continue in same session
$followUp = PromptContext::make('Now add tests')
    ->withSession($sessionId);
$result2 = $claude->execute($followUp);

// Fork session for alternative approach
$alternative = PromptContext::make('Try a different approach')
    ->withSession($sessionId, fork: true);
$result3 = $claude->execute($alternative);
```

### Error Handling

```php
use App\Conduit\Exceptions\{
    ClaudeNotInstalledException,
    ClaudeAuthenticationException,
    ClaudeExecutionException
};

try {
    $result = $claude->execute($context);
} catch (ClaudeNotInstalledException $e) {
    return response()->json([
        'error' => 'Claude Code is not installed',
        'install_url' => 'https://claude.com/product/claude-code',
    ], 424);
} catch (ClaudeAuthenticationException $e) {
    return response()->json([
        'error' => 'Not authenticated with Claude Code',
        'message' => 'Please run: claude auth login',
    ], 401);
} catch (ClaudeExecutionException $e) {
    Log::error('Claude execution failed', [
        'error' => $e->getMessage(),
        'code' => $e->getCode(),
    ]);

    return response()->json([
        'error' => 'Failed to execute Claude prompt',
    ], 500);
}
```

### Real-time Broadcasting

```php
use App\Conduit\Events\ClaudeStreamEventReceived;
use Illuminate\Support\Facades\Event;

Event::listen(ClaudeStreamEventReceived::class, function ($event) {
    if ($event->event->isAssistant()) {
        // Broadcast to frontend via Pusher/Laravel Echo
        broadcast(new ClaudeResponseChunk(
            message: $event->event->getMessage(),
            sessionId: $event->event->getSessionId(),
        ));
    }
});

$result = $claude->stream($context, function (ClaudeStreamEvent $event) {
    event(new ClaudeStreamEventReceived($event));
});
```

---

## Architecture Decisions

### 1. Immutable Value Objects
All DTOs are immutable with readonly properties and fluent methods that return new instances. This prevents accidental mutations and makes the code more predictable.

### 2. Symfony Process via Laravel Facade
Using Laravel's Process facade provides a clean API and works across Laravel 10, 11, and 12. It handles process execution, streaming, and error handling elegantly.

### 3. JSON Output by Default
Always use `--output-format json` for programmatic access. The JSON structure provides rich metadata including:
- Token usage and costs
- Session IDs
- Execution time
- Permission denials
- Per-model usage breakdown

### 4. Streaming via Callbacks
The streaming implementation uses PHP callbacks rather than generators or observables for simplicity and Laravel compatibility. Each newline-delimited JSON event is parsed and passed to the callback.

### 5. Exception Hierarchy
Specific exceptions for different failure modes:
- `ClaudeNotInstalledException` - CLI not found (HTTP 424)
- `ClaudeAuthenticationException` - Not logged in (HTTP 401)
- `ClaudeExecutionException` - Execution failed (HTTP 500)

### 6. Session Management
Session IDs are returned in every response and can be used to:
- Resume conversations with `--resume`
- Fork sessions for alternative approaches with `--fork-session`
- Continue most recent session with `--continue`

### 7. Working Directory Context
Claude Code reads CLAUDE.md from the working directory, so always set the working directory to the project root to provide proper context.

### 8. Tool Restrictions
Support for `--allowedTools` and `--disallowedTools` to control what Claude can do:
- Read-only mode: `['Read', 'Grep', 'Glob']`
- No writes: `disallowedTools: ['Write', 'Edit']`
- No bash: `disallowedTools: ['Bash']`

---

## Implementation Checklist

- [ ] Create contracts and interfaces
- [ ] Implement value objects and DTOs
- [ ] Implement ClaudeCodeProvider
- [ ] Create exception classes
- [ ] Define observable events
- [ ] Write unit tests for PromptContext command building
- [ ] Write integration tests for ClaudeCodeProvider
- [ ] Add service provider registration
- [ ] Create configuration file for defaults
- [ ] Add logging and monitoring
- [ ] Create documentation
- [ ] Add example usage in README

---

## References

### Documentation
- [Claude Code CLI Reference](https://code.claude.com/docs/en/cli-reference)
- [Run Claude Code Programmatically](https://code.claude.com/docs/en/headless)
- [Identity and Access Management](https://code.claude.com/docs/en/iam)
- [Laravel Process Documentation](https://laravel.com/docs/12.x/processes)
- [Symfony Process Component](https://symfony.com/doc/current/components/process.html)

### Blog Posts & Guides
- [Laravel's New Process Facade](https://beyondco.de/blog/laravel-10-new-process-facade)
- [Claude Code Cheat Sheet](https://shipyard.build/blog/claude-code-cheat-sheet/)
- [Tricks for Running Commands with Laravel Process](https://fly.io/laravel-bytes/run-commands-with-laravel-process/)

### Community Resources
- [Claude Code GitHub Issues](https://github.com/anthropics/claude-code/issues)
- [Print Mode Use Cases Discussion](https://github.com/anthropics/claude-code/issues/762)
- [Awesome Claude Code](https://github.com/hesreallyhim/awesome-claude-code)
