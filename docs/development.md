# Development Guidelines

## Prerequisites

- Rust 1.70 or later
- JankenSQLHub library (located at `../jankensqlhub`)

## Running the Service

```bash
cargo run
```

The service starts on `http://localhost:3000`.

## Testing

```bash
cargo test
```

## Code Quality

After making changes, always run:

```bash
cargo clippy --fix --allow-dirty  # Fix compiler warnings
cargo fmt                          # Format code
```

## Testing Principles

- Do not use conditional assertions; if wrapped in a callback function, ensure the wrapper executes 100% of the time
- Ensure tests are simple and straightforward to read
- Aim for broader coverage with fewer, well-designed tests
- Prioritize testing for security issues, particularly SQL injection vulnerabilities
- Avoid loops in tests, except when iterating over arrays with obvious elements
- Do not over-engineer tests
- Always assert exact values; avoid vague assertions like `option.is_some` or `result.is_ok`
- Do not use special numbers like PI or E (triggers unnecessary linting errors)
- Test HTTP endpoints with various parameter combinations
- Test error handling for invalid table names, missing fields, and malformed requests

## Code Quality Standards

- Apply Occam's Razor; be minimalist
- Follow Rust API design principles and idioms
- Prefer explicit error handling over panics
- Use meaningful names for functions, variables, and types
- Add documentation comments for public APIs
- Keep functions small and focused on single responsibilities
- Use pattern matching for error handling and control flow
- If a variable has no possibility of being None within its flow, do not define it as an Option
- Follow Axum best practices for handler functions and middleware
- Use proper HTTP status codes for different error scenarios

## Task Management

Progress files use the naming convention: `_progress_<task_name>.md`

When starting a new task:
1. Read the progress file to review completed work; if it doesn't exist, create one
2. If exceeding context window capacity, save progress to the progress file, ensuring content is clear and actionable for another LLM agent to resume

## JankenSQLHub Integration

- Use JankenSQLHub's parameter validation to prevent SQL injection
- Leverage `#[table]` parameter support for dynamic table names
- Use list parameters for the fields parameter
- Build queries using JankenSQLHub's `QueryDef`
- Execute queries using the appropriate runner (SQLite in this case)

## Documentation Updates

When asked to update documents, check if any of the following have out-of-sync facts:
- README.md
- .clinerules
- docs/*.md