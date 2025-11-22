# ORCS

Orchestrated Reasoning and Collaboration System

## Development

### TypeScript Type Generation

After modifying Rust enum types in `crates/orcs-core/src/schema.rs`, regenerate TypeScript types:

```bash
cargo run --bin orcs -- schema generate
```

This will update `orcs-desktop/src/types/generated/schema.ts` automatically.

**Note:** Run this command whenever you:
- Add new enum variants (e.g., `ConversationMode::Detailed`)
- Modify existing enum definitions
- Add new schema types
