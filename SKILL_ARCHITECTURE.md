# Skill Architecture Design
*For Laravel Code Generation Automation*

## Philosophy

**Separation of Concerns:**
- **AI decides** which pattern to use (understanding user intent)
- **Skill executes** the standardized implementation
- **User talks** in natural language (voice or text)

**Example Flow:**
```
User: "I need an API endpoint for user preferences"
  ↓
AI: Recognizes API endpoint pattern
  ↓
AI: Calls /scaffold:api-endpoint user-preferences
  ↓
Skill: Generates controller + route + test
  ↓
User: Gets working code in 5 seconds
```

---

## Skill Design Pattern

### Skill Template Structure

```markdown
# /scaffold:api-endpoint

Generate a Laravel API endpoint following project standards.

## Usage
/scaffold:api-endpoint {name} [options]

## Options
--read-only          Only generate index() and show() methods
--write-only         Only generate store() and update() methods
--controller-only    Skip route and test generation
--resource           Generate with API Resource class

## Examples
/scaffold:api-endpoint session-recovery
/scaffold:api-endpoint user-preferences --read-only
/scaffold:api-endpoint posts --resource

## What It Generates
1. app/Http/Controllers/Api/{Name}Controller.php
2. Route registration in routes/api.php
3. tests/Feature/Api/{Name}Test.php
4. app/Http/Resources/{Name}Resource.php (if --resource)

## Standards Applied
- declare(strict_types=1);
- JsonResponse return types
- Request validation
- Factory-based test data
- Pest describe/it blocks
- Knowledge API token middleware pattern
```

### Implementation (Example: API Endpoint Skill)

```bash
#!/usr/bin/env bash
# ~/.claude/commands/scaffold-api-endpoint.md

# Parse arguments
NAME=$1
OPTIONS="${@:2}"

# Validate
if [ -z "$NAME" ]; then
  echo "❌ Error: Name required"
  echo "Usage: /scaffold:api-endpoint {name}"
  exit 1
fi

# Check if in Laravel project
if [ ! -f "artisan" ]; then
  echo "❌ Error: Not in a Laravel project"
  exit 1
fi

# Convert name to proper case
CONTROLLER_NAME="$(echo $NAME | sed 's/.*/\u&/')Controller"
TEST_NAME="$(echo $NAME | sed 's/.*/\u&/')Test"

echo "🏗️  Scaffolding API endpoint: $NAME"
echo ""

# Generate Controller
cat > "app/Http/Controllers/Api/${CONTROLLER_NAME}.php" << 'CONTROLLER'
<?php

declare(strict_types=1);

namespace App\Http\Controllers\Api;

use App\Http\Controllers\Controller;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;

class __NAME__Controller extends Controller
{
    public function index(Request $request): JsonResponse
    {
        // TODO: Implement index logic
        return response()->json([
            'data' => [],
            'meta' => [
                'total' => 0,
            ],
        ]);
    }

    public function show(Request $request, string $id): JsonResponse
    {
        // TODO: Implement show logic
        return response()->json([
            'data' => null,
        ]);
    }

    public function store(Request $request): JsonResponse
    {
        // TODO: Add validation
        // TODO: Implement store logic
        return response()->json([
            'data' => null,
        ], 201);
    }

    public function update(Request $request, string $id): JsonResponse
    {
        // TODO: Add validation
        // TODO: Implement update logic
        return response()->json([
            'data' => null,
        ]);
    }

    public function destroy(string $id): JsonResponse
    {
        // TODO: Implement destroy logic
        return response()->json(null, 204);
    }
}
CONTROLLER

# Replace placeholders
sed -i '' "s/__NAME__/${NAME^}/g" "app/Http/Controllers/Api/${CONTROLLER_NAME}.php"

echo "✅ Created app/Http/Controllers/Api/${CONTROLLER_NAME}.php"

# Add route to routes/api.php
# (In practice, use a more robust method to inject routes)
echo ""
echo "📝 Add this route to routes/api.php:"
echo ""
echo "Route::middleware(['knowledge.api'])->prefix('api/v1')->group(function () {"
echo "    Route::apiResource('${NAME}', App\\Http\\Controllers\\Api\\${CONTROLLER_NAME}::class);"
echo "});"
echo ""

# Generate Test
cat > "tests/Feature/Api/${TEST_NAME}.php" << 'TEST'
<?php

declare(strict_types=1);

use Illuminate\Foundation\Testing\RefreshDatabase;

uses(RefreshDatabase::class);

beforeEach(function (): void {
    config(['services.knowledge.api_token' => 'test-token']);
});

describe('__NAME__ API', function (): void {
    it('returns list of items', function (): void {
        $response = $this->withHeaders([
            'Authorization' => 'Bearer test-token',
        ])->getJson('/api/v1/__ROUTE__');

        $response->assertOk()
            ->assertJsonStructure([
                'data',
                'meta' => ['total'],
            ]);
    });

    it('returns single item', function (): void {
        // TODO: Create test data
        $response = $this->withHeaders([
            'Authorization' => 'Bearer test-token',
        ])->getJson('/api/v1/__ROUTE__/1');

        $response->assertOk();
    });

    it('creates new item', function (): void {
        $data = [
            // TODO: Add test data
        ];

        $response = $this->withHeaders([
            'Authorization' => 'Bearer test-token',
        ])->postJson('/api/v1/__ROUTE__', $data);

        $response->assertCreated();
    });

    it('updates existing item', function (): void {
        // TODO: Create test data
        $data = [
            // TODO: Add update data
        ];

        $response = $this->withHeaders([
            'Authorization' => 'Bearer test-token',
        ])->putJson('/api/v1/__ROUTE__/1', $data);

        $response->assertOk();
    });

    it('deletes item', function (): void {
        // TODO: Create test data
        $response = $this->withHeaders([
            'Authorization' => 'Bearer test-token',
        ])->deleteJson('/api/v1/__ROUTE__/1');

        $response->assertNoContent();
    });

    it('requires authentication', function (): void {
        $response = $this->getJson('/api/v1/__ROUTE__');

        $response->assertUnauthorized();
    });
});
TEST

# Replace placeholders
ROUTE_NAME=$(echo "$NAME" | sed 's/-/_/g')
sed -i '' "s/__NAME__/${NAME^}/g" "tests/Feature/Api/${TEST_NAME}.php"
sed -i '' "s/__ROUTE__/${NAME}/g" "tests/Feature/Api/${TEST_NAME}.php"

echo "✅ Created tests/Feature/Api/${TEST_NAME}.php"
echo ""

# Run tests
echo "🧪 Running tests..."
php artisan test --filter="${TEST_NAME}"

echo ""
echo "✅ API endpoint scaffolded successfully!"
echo ""
echo "Next steps:"
echo "1. Add route to routes/api.php (see above)"
echo "2. Implement TODO items in controller"
echo "3. Add proper validation"
echo "4. Update test data in test file"
```

---

## Required Skills (Priority Order)

### 1. `/scaffold:api-endpoint`
**Purpose:** Generate API controller + route + test
**Template:** See above
**Estimated build time:** 2-3 hours

### 2. `/scaffold:model`
**Purpose:** Generate model + migration + factory + test
**Template:** Similar pattern to api-endpoint
**Generates:**
- Model with relationships, casts, fillable
- Migration with schema
- Factory with realistic data
- Model test with relationship tests

### 3. `/generate:tests`
**Purpose:** Generate tests for existing code
**Special:** Reads existing file, analyzes, generates comprehensive tests
**Uses:** AST parsing or Claude analysis to understand code structure

### 4. `/scaffold:filament-widget`
**Purpose:** Generate Filament widget + test
**Generates:**
- Widget class
- Test file
- Auto-registers in panel

### 5. `/scaffold:filament-resource`
**Purpose:** Full CRUD resource
**Generates:**
- Resource class
- Form/Table schemas
- Page classes
- Tests

---

## AI Integration Layer

### How AI Chooses Skills

```yaml
# Skill selection logic (handled by Claude)

User Intent: "create an API endpoint"
→ Match: /scaffold:api-endpoint

User Intent: "I need a model for tracking nodes"
→ Match: /scaffold:model

User Intent: "write tests for this service"
→ Match: /generate:tests

User Intent: "add a dashboard widget for stats"
→ Match: /scaffold:filament-widget
```

### Voice Integration

```bash
User (voice): "I need an API endpoint for dashboard statistics"

Claude hears → transcribes → analyzes intent:
  - Identifies: API endpoint pattern
  - Extracts: resource name "dashboard statistics"
  - Normalizes: "dashboard-stats"

Claude executes:
  /scaffold:api-endpoint dashboard-stats --read-only

Skill runs → generates code → reports back

Claude speaks:
  "Created API endpoint for dashboard stats with controller,
   routes, and tests. The index method is ready for your logic."
```

---

## Implementation Plan

### Phase 1: Core Skills (Week 1)
- [ ] Build `/scaffold:api-endpoint` skill
- [ ] Build `/scaffold:model` skill
- [ ] Test manually with Claude Code
- [ ] Document usage patterns

### Phase 2: Testing (Week 2)
- [ ] Build `/generate:tests` skill
- [ ] Test all skills together
- [ ] Refine based on usage

### Phase 3: Filament (Week 3)
- [ ] Build `/scaffold:filament-widget`
- [ ] Build `/scaffold:filament-resource`
- [ ] Integration testing

### Phase 4: Voice Integration (Week 4)
- [ ] Connect skills to voice interface
- [ ] Test natural language → skill execution
- [ ] Optimize latency (goal: <5s from voice to generated code)

---

## Success Metrics

### Quantitative
- **Time per feature:** <5 minutes (down from 30-45 min)
- **Lines of code generated:** 100-200 per skill execution
- **Test success rate:** 100% (generated code must pass tests)
- **Voice to code latency:** <5 seconds

### Qualitative
- Code follows 100% of project standards
- No manual file creation needed
- Tests are comprehensive and maintainable
- Natural language input works consistently

---

## Technical Notes

### Skill Format
Skills are markdown files in `~/.claude/commands/` with embedded bash/python

### Standards to Encode
1. `declare(strict_types=1);` in all PHP files
2. JsonResponse return types for APIs
3. Pest describe/it test structure
4. Factory-based test data
5. Knowledge API token middleware
6. RefreshDatabase for tests

### Template Variables
- `__NAME__` - Controller/Model name
- `__ROUTE__` - API route name
- `__NAMESPACE__` - PHP namespace
- `__TABLE__` - Database table name
