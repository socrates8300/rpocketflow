# RPocketFlow Developer Guidelines

## Commands
- Build: `cargo build`
- Run tests: `cargo test`
- Run single test: `cargo test test_name`
- Run specific test module: `cargo test module_name`
- Run test with output: `cargo test -- --nocapture`
- Format code: `cargo fmt`
- Lint check: `cargo clippy`

## Code Style
- Follow Rust naming conventions (snake_case for variables/functions, CamelCase for types)
- Required type annotations in closures (ex: `|data: &Value| {}`)
- Clone `Arc<Mutex<>>` node references when reusing in flow connections
- Handle JSON number values safely, considering both f64 and i64 representations
- Use `.unwrap_or(default)` instead of `.unwrap()` when possible
- Keep error messages descriptive and context-specific
- Use macros (node_impl!, flow!, etc.) to reduce boilerplate
- Ensure proper imports: `use rpocketflow::*` for library usage

## Thread Safety
- Avoid manual `Send`/`Sync` trait implementations; let Rust's type system handle them
- Use thread-safe primitives like `Arc<Mutex<>>` for sharing data between threads
- Ensure all node implementations naturally implement `Send` and `Sync` through their fields
- Handle mutex poisoning gracefully by recovering the poisoned mutex when appropriate

## Architecture
- Nodes are basic building blocks - implement Node trait plus SyncNode or AsyncNode
- Flows orchestrate nodes, maintain shared state, and handle transitions
- Use shared HashMap<String, Value> for state between nodes
- Process node returns Action (default, named, terminate) to direct flow