<?php

/**
 * Practical Examples of Claude Provider Integration
 *
 * This file demonstrates real-world usage patterns for integrating
 * Claude Code CLI with Laravel/PHP applications.
 */

namespace App\Examples;

use App\Conduit\Contracts\ClaudeProvider;
use App\Conduit\DataTransferObjects\{PromptContext, ClaudeStreamEvent};
use App\Conduit\Events\{ClaudeExecutionStarted, ClaudeExecutionCompleted};
use App\Conduit\Exceptions\{ClaudeAuthenticationException, ClaudeExecutionException};
use Illuminate\Support\Facades\{Log, DB, Cache, Event};

class ClaudeProviderExamples
{
    public function __construct(
        private readonly ClaudeProvider $claude,
    ) {}

    /**
     * Example 1: Basic code analysis
     */
    public function analyzeCodebase(string $projectPath): array
    {
        $context = PromptContext::make('Analyze this codebase for security vulnerabilities')
            ->withWorkingDirectory($projectPath)
            ->withTools(
                allowed: ['Read', 'Grep', 'Glob'],
                disallowed: ['Bash', 'Write', 'Edit']
            )
            ->withBudget(0.50);

        $result = $this->claude->execute($context);

        return [
            'analysis' => $result->result,
            'cost' => $result->totalCostUsd,
            'session_id' => $result->sessionId,
            'duration_seconds' => $result->durationMs / 1000,
        ];
    }

    /**
     * Example 2: Generate tests with streaming progress
     */
    public function generateTests(string $filePath, callable $onProgress): array
    {
        $context = PromptContext::make("Generate comprehensive tests for {$filePath}")
            ->withWorkingDirectory(dirname($filePath))
            ->withTools(allowed: ['Read', 'Write', 'Grep']);

        $messages = [];

        $result = $this->claude->stream($context, function (ClaudeStreamEvent $event) use ($onProgress, &$messages) {
            if ($event->isAssistant()) {
                $message = $event->getMessage();
                $messages[] = $message;
                $onProgress([
                    'type' => 'message',
                    'content' => $message,
                ]);
            }

            if ($event->isInit()) {
                $onProgress([
                    'type' => 'init',
                    'session_id' => $event->getSessionId(),
                    'tools' => $event->data['tools'] ?? [],
                ]);
            }

            if ($event->isSuccess()) {
                $onProgress([
                    'type' => 'complete',
                    'cost' => $event->data['total_cost_usd'] ?? 0,
                ]);
            }
        });

        return [
            'result' => $result->result,
            'messages' => $messages,
            'session_id' => $result->sessionId,
            'cost' => $result->totalCostUsd,
        ];
    }

    /**
     * Example 3: Multi-turn conversation for code review
     */
    public function interactiveCodeReview(string $projectPath): array
    {
        $reviews = [];

        // Initial review
        $context1 = PromptContext::make('Review this code for best practices')
            ->withWorkingDirectory($projectPath)
            ->withTools(allowed: ['Read', 'Grep', 'Glob']);

        $result1 = $this->claude->execute($context1);
        $sessionId = $result1->sessionId;

        $reviews[] = [
            'question' => 'Initial review',
            'response' => $result1->result,
            'cost' => $result1->totalCostUsd,
        ];

        // Follow-up: Focus on specific areas
        $context2 = PromptContext::make('Now focus specifically on security concerns')
            ->withSession($sessionId)
            ->withWorkingDirectory($projectPath);

        $result2 = $this->claude->execute($context2);

        $reviews[] = [
            'question' => 'Security focus',
            'response' => $result2->result,
            'cost' => $result2->totalCostUsd,
        ];

        // Follow-up: Suggest improvements
        $context3 = PromptContext::make('Suggest specific improvements')
            ->withSession($sessionId)
            ->withWorkingDirectory($projectPath);

        $result3 = $this->claude->execute($context3);

        $reviews[] = [
            'question' => 'Improvements',
            'response' => $result3->result,
            'cost' => $result3->totalCostUsd,
        ];

        $totalCost = $result1->totalCostUsd + $result2->totalCostUsd + $result3->totalCostUsd;

        return [
            'session_id' => $sessionId,
            'reviews' => $reviews,
            'total_cost' => $totalCost,
            'num_turns' => 3,
        ];
    }

    /**
     * Example 4: Cached analysis to avoid redundant executions
     */
    public function cachedAnalysis(string $filePath, int $cacheTtlMinutes = 60): string
    {
        $cacheKey = 'claude:analysis:' . md5($filePath . filemtime($filePath));

        return Cache::remember($cacheKey, now()->addMinutes($cacheTtlMinutes), function () use ($filePath) {
            $context = PromptContext::make("Analyze {$filePath} and suggest improvements")
                ->withWorkingDirectory(dirname($filePath))
                ->withTools(allowed: ['Read']);

            $result = $this->claude->execute($context);

            Log::info('Claude analysis executed', [
                'file' => $filePath,
                'cost' => $result->totalCostUsd,
                'session_id' => $result->sessionId,
            ]);

            return $result->result;
        });
    }

    /**
     * Example 5: Real-time broadcasting for collaborative sessions
     */
    public function collaborativeAnalysis(string $projectPath, string $channelName): array
    {
        $context = PromptContext::make('Perform comprehensive architecture analysis')
            ->withWorkingDirectory($projectPath)
            ->withTools(allowed: ['Read', 'Grep', 'Glob']);

        Event::dispatch(new ClaudeExecutionStarted(
            prompt: 'Architecture analysis',
            sessionId: null,
        ));

        $events = [];

        $result = $this->claude->stream($context, function (ClaudeStreamEvent $event) use ($channelName, &$events) {
            $events[] = [
                'type' => $event->type,
                'subtype' => $event->subtype,
            ];

            if ($event->isAssistant()) {
                broadcast(new \App\Events\ClaudeMessageChunk(
                    channel: $channelName,
                    message: $event->getMessage(),
                    sessionId: $event->getSessionId(),
                ));
            }

            if ($event->isResult()) {
                broadcast(new \App\Events\ClaudeAnalysisComplete(
                    channel: $channelName,
                    result: $event->getMessage(),
                    cost: $event->data['total_cost_usd'] ?? 0,
                ));
            }
        });

        Event::dispatch(new ClaudeExecutionCompleted($result));

        return [
            'result' => $result->result,
            'session_id' => $result->sessionId,
            'events_count' => count($events),
            'cost' => $result->totalCostUsd,
        ];
    }

    /**
     * Example 6: Budget-aware batch processing
     */
    public function batchAnalysis(array $files, float $totalBudget): array
    {
        $results = [];
        $remainingBudget = $totalBudget;

        foreach ($files as $file) {
            if ($remainingBudget <= 0.01) {
                Log::warning('Budget exhausted during batch processing', [
                    'processed' => count($results),
                    'remaining' => count($files) - count($results),
                ]);
                break;
            }

            try {
                $context = PromptContext::make("Review {$file}")
                    ->withWorkingDirectory(dirname($file))
                    ->withTools(allowed: ['Read'])
                    ->withBudget(min($remainingBudget, 0.25)); // Max $0.25 per file

                $result = $this->claude->execute($context);

                $results[] = [
                    'file' => $file,
                    'analysis' => $result->result,
                    'cost' => $result->totalCostUsd,
                    'session_id' => $result->sessionId,
                ];

                $remainingBudget -= $result->totalCostUsd;

            } catch (ClaudeExecutionException $e) {
                Log::error('Failed to analyze file', [
                    'file' => $file,
                    'error' => $e->getMessage(),
                ]);

                $results[] = [
                    'file' => $file,
                    'error' => $e->getMessage(),
                    'cost' => 0,
                ];
            }
        }

        return [
            'results' => $results,
            'total_cost' => $totalBudget - $remainingBudget,
            'budget_remaining' => $remainingBudget,
            'files_processed' => count($results),
        ];
    }

    /**
     * Example 7: Usage tracking and analytics
     */
    public function executeWithTracking(PromptContext $context, string $userId): array
    {
        $startTime = microtime(true);

        try {
            $result = $this->claude->execute($context);

            DB::table('claude_usage')->insert([
                'user_id' => $userId,
                'session_id' => $result->sessionId,
                'prompt' => $context->prompt,
                'result' => $result->result,
                'cost_usd' => $result->totalCostUsd,
                'duration_ms' => $result->durationMs,
                'input_tokens' => $result->usage->inputTokens,
                'output_tokens' => $result->usage->outputTokens,
                'cache_read_tokens' => $result->usage->cacheReadInputTokens,
                'cache_creation_tokens' => $result->usage->cacheCreationInputTokens,
                'success' => true,
                'error_message' => null,
                'created_at' => now(),
            ]);

            // Update user's monthly usage
            DB::table('user_monthly_usage')
                ->where('user_id', $userId)
                ->where('month', now()->format('Y-m'))
                ->increment('total_cost_usd', $result->totalCostUsd);

            return [
                'success' => true,
                'result' => $result->result,
                'session_id' => $result->sessionId,
                'cost' => $result->totalCostUsd,
            ];

        } catch (ClaudeExecutionException $e) {
            DB::table('claude_usage')->insert([
                'user_id' => $userId,
                'session_id' => null,
                'prompt' => $context->prompt,
                'result' => null,
                'cost_usd' => 0,
                'duration_ms' => (int)((microtime(true) - $startTime) * 1000),
                'input_tokens' => 0,
                'output_tokens' => 0,
                'success' => false,
                'error_message' => $e->getMessage(),
                'created_at' => now(),
            ]);

            throw $e;
        }
    }

    /**
     * Example 8: Alternative approaches using session forking
     */
    public function exploreAlternatives(string $problem, string $projectPath): array
    {
        // Initial analysis
        $context = PromptContext::make($problem)
            ->withWorkingDirectory($projectPath);

        $result1 = $this->claude->execute($context);
        $baseSessionId = $result1->sessionId;

        $approaches = [
            [
                'name' => 'Original approach',
                'result' => $result1->result,
                'cost' => $result1->totalCostUsd,
                'session_id' => $result1->sessionId,
            ],
        ];

        // Fork 1: Object-oriented approach
        $context2 = PromptContext::make('Now try an object-oriented approach')
            ->withSession($baseSessionId, fork: true)
            ->withWorkingDirectory($projectPath);

        $result2 = $this->claude->execute($context2);

        $approaches[] = [
            'name' => 'Object-oriented',
            'result' => $result2->result,
            'cost' => $result2->totalCostUsd,
            'session_id' => $result2->sessionId,
        ];

        // Fork 2: Functional approach
        $context3 = PromptContext::make('Now try a functional programming approach')
            ->withSession($baseSessionId, fork: true)
            ->withWorkingDirectory($projectPath);

        $result3 = $this->claude->execute($context3);

        $approaches[] = [
            'name' => 'Functional',
            'result' => $result3->result,
            'cost' => $result3->totalCostUsd,
            'session_id' => $result3->sessionId,
        ];

        return [
            'base_session_id' => $baseSessionId,
            'approaches' => $approaches,
            'total_cost' => array_sum(array_column($approaches, 'cost')),
        ];
    }

    /**
     * Example 9: Health check and diagnostics
     */
    public function healthCheck(): array
    {
        $health = [
            'claude_installed' => false,
            'claude_authenticated' => false,
            'version' => null,
            'status' => 'unhealthy',
            'errors' => [],
        ];

        try {
            $health['claude_installed'] = $this->claude->isInstalled();

            if (!$health['claude_installed']) {
                $health['errors'][] = 'Claude Code CLI is not installed';
                return $health;
            }

            $health['claude_authenticated'] = $this->claude->isAuthenticated();

            if (!$health['claude_authenticated']) {
                $health['errors'][] = 'Not authenticated with Claude Code';
                return $health;
            }

            $health['version'] = $this->claude->getVersion();

            // Try a simple execution
            $testContext = PromptContext::make('What is 2+2?')
                ->withBudget(0.05);

            $result = $this->claude->execute($testContext);

            if ($result->successful() && str_contains($result->result, '4')) {
                $health['status'] = 'healthy';
                $health['test_cost'] = $result->totalCostUsd;
            } else {
                $health['errors'][] = 'Test execution did not return expected result';
            }

        } catch (\Exception $e) {
            $health['errors'][] = $e->getMessage();
        }

        return $health;
    }

    /**
     * Example 10: Graceful degradation and fallbacks
     */
    public function analyzeWithFallback(string $filePath): string
    {
        try {
            // Try Claude first
            $context = PromptContext::make("Analyze {$filePath}")
                ->withWorkingDirectory(dirname($filePath))
                ->withTools(allowed: ['Read'])
                ->withBudget(0.10);

            $result = $this->claude->execute($context);

            return $result->result;

        } catch (ClaudeAuthenticationException $e) {
            Log::warning('Claude not authenticated, using fallback analysis');

            return $this->basicStaticAnalysis($filePath);

        } catch (ClaudeExecutionException $e) {
            Log::error('Claude execution failed, using fallback', [
                'error' => $e->getMessage(),
            ]);

            return $this->basicStaticAnalysis($filePath);
        }
    }

    /**
     * Simple fallback analysis (placeholder)
     */
    private function basicStaticAnalysis(string $filePath): string
    {
        $lines = count(file($filePath));
        $size = filesize($filePath);

        return "Basic analysis: {$lines} lines, {$size} bytes. Claude Code integration not available.";
    }
}
