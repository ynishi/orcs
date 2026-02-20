//! Supported LLM model versions and update procedures.
//!
//! This module documents all supported LLM model IDs and provides a checklist
//! for adding or updating model versions when providers release new models.
//!
//! # Supported Models (as of 2026-02-20)
//!
//! ## Anthropic Claude
//!
//! | Model ID | Tier | Notes |
//! |----------|------|-------|
//! | `claude-opus-4-6` | Latest flagship | Best for complex reasoning and coding |
//! | `claude-sonnet-4-6` | Latest balanced | Best speed/intelligence ratio (default) |
//! | `claude-haiku-4-5-20251001` | Fast | Fastest, near-frontier intelligence |
//! | `claude-opus-4-5-20251101` | Legacy | Previous flagship |
//! | `claude-sonnet-4-5-20250929` | Legacy | Previous balanced |
//! | `claude-opus-4-1-20250805` | Legacy | |
//! | `claude-sonnet-4-20250514` | Legacy | |
//! | `claude-opus-4-20250514` | Legacy | |
//!
//! Reference: <https://platform.claude.com/docs/en/about-claude/models/overview>
//!
//! ## Google Gemini
//!
//! | Model ID | Tier | Notes |
//! |----------|------|-------|
//! | `gemini-3.1-pro-preview` | Latest flagship | Best reasoning, agentic workflows |
//! | `gemini-3-pro-preview` | Preview | Previous 3.x flagship |
//! | `gemini-3-flash-preview` | Preview fast | Pro-level intelligence at Flash speed |
//! | `gemini-2.5-pro` | Stable | Production-ready |
//! | `gemini-2.5-flash` | Stable default | Fast, budget-friendly (default) |
//! | `gemini-2.5-flash-lite` | Stable lite | Lightest option |
//!
//! Reference: <https://ai.google.dev/gemini-api/docs/models>
//!
//! ## OpenAI
//!
//! | Model ID | Tier | Notes |
//! |----------|------|-------|
//! | `gpt-5.2` | Latest | Latest general-purpose |
//! | `gpt-5.2-pro` | Latest pro | Higher precision |
//! | `gpt-5` | Flagship | Default model (default) |
//! | `gpt-5-mini` | Mini | Smaller, faster |
//! | `gpt-4.1` | Legacy | Being deprecated |
//! | `gpt-4.1-mini` | Legacy mini | Being deprecated |
//!
//! Reference: <https://platform.openai.com/docs/models>
//!
//! # How to Add or Update Model Versions
//!
//! When a provider releases a new model, update these locations:
//!
//! ## Checklist
//!
//! 1. **Frontend model selector** (`orcs-desktop/src/components/personas/PersonaEditorModal.tsx`)
//!    - `CLAUDE_MODEL_OPTIONS` - Claude model dropdown
//!    - `GEMINI_MODEL_OPTIONS` - Gemini model dropdown
//!    - `OPENAI_MODEL_OPTIONS` - OpenAI model dropdown
//!    - Update the `Default (...)` label in the first entry if the default changes
//!
//! 2. **Backend default constants** (one per agent file in `orcs-interaction/src/`)
//!    - `claude_api_agent.rs` → `DEFAULT_CLAUDE_MODEL`
//!    - `kaiba_api_agent.rs` → `DEFAULT_CLAUDE_MODEL` (Kaiba uses Claude for execution)
//!    - `gemini_api_agent.rs` → `DEFAULT_GEMINI_MODEL`, `GEMINI_3_PRO_MODEL`
//!    - `openai_api_agent.rs` → `DEFAULT_OPENAI_MODEL`
//!
//! 3. **Config defaults** (`orcs-core/src/config.rs`)
//!    - `ClaudeModelConfig::default()` → `model_name`
//!    - `GeminiModelConfig::default()` → `model_name`
//!    - `OpenAIModelConfig::default()` → `model_name`
//!
//! 4. **Web search agent** (`orcs-core/src/agent/web_search.rs`)
//!    - `DEFAULT_MODEL` — uses Gemini for web search; keep on stable version
//!
//! 5. **This documentation** (`orcs-interaction/src/supported_models.rs`)
//!    - Update the model table above
//!
//! ## Verification
//!
//! After updating, run:
//!
//! ```bash
//! # Ensure compilation succeeds
//! cargo check --workspace
//!
//! # Grep for stale model IDs to catch missed references
//! rg "claude-sonnet-4-20250514|gpt-4o" --type rust --type ts
//! ```
//!
//! ## Notes
//!
//! - `DEFAULT_*_MODEL` constants are used when no `model_name` is specified in the Persona config.
//! - Users can override the model per-persona via the Persona Editor UI.
//! - The `config.toml` `model_name` field also overrides defaults if present.
//! - Stable versions (e.g., `gemini-2.5-flash`) are preferred for defaults over preview versions.
//! - For Gemini, keep `DEFAULT_GEMINI_MODEL` on a stable version; the `GEMINI_3_PRO_MODEL`
//!   constant is used by `try_gemini_3_from_env()` for explicit Gemini 3 usage.
