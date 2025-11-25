# ORCS Desktop

Orchestrated Reasoning and Collaboration System - AI-powered workspace management application

## Features

- Multi-workspace session management
- AI-powered conversation interface
- Local-first architecture with filesystem-based storage
- Cross-platform support (macOS, Windows, Linux)
- Auto-update functionality

## Installation

### From Release

Download the latest release for your platform:
- **macOS**: `ORCS-Desktop_<version>_aarch64.dmg` or `ORCS-Desktop_<version>_x64.dmg`
- **Windows**: `ORCS-Desktop_<version>_x64-setup.exe`
- **Linux**: `ORCS-Desktop_<version>_amd64.AppImage` or `.deb`

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yutakanishimura/orcs.git
cd orcs

# Install dependencies
cd orcs-desktop
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

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

## License

MIT License - see [LICENSE](LICENSE) for details

## Author

Yutaka Nishimura
