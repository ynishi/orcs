# ORCS

Orchestrated Reasoning and Collaboration System

## Development

### TypeScript Type Generation

After modifying Rust enum types in `crates/orcs-core/src/schema.rs`, regenerate TypeScript types:

```bash
make schema-generate
```

Or directly:
```bash
cargo run --bin orcs -- schema generate
```

This will update `orcs-desktop/src/types/generated/schema.ts` automatically.

**Note:** Run this command whenever you:
- Add new enum variants (e.g., `ConversationMode::Detailed`)
- Modify existing enum definitions
- Add new schema types

### Common Development Commands

```bash
make help             # Show all available commands
make schema-generate  # Generate TypeScript types from Rust schemas
make dev              # Run Tauri app in development mode
make check            # Type check Rust code
make fmt              # Format all code
make test             # Run all tests
```
