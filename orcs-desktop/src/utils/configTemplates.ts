/**
 * Configuration templates for ORCS
 * These templates help users understand and create configuration files
 */

export const PERSONA_CONFIG_TEMPLATE = `# ORCS Persona Configuration Guide

## Overview
Personas in ORCS are AI agents with specific roles, backgrounds, and communication styles.
Each persona is defined as a TOML file in the personas directory.

## Directory Locations
- Personas Directory: Access via Settings Menu → "Personas Directory"
- Config File: Settings Menu → "Open Config File"
- Secrets File: Settings Menu → "Open Secret File"

## PersonaConfig Type Definition

\`\`\`typescript
interface PersonaConfig {
  id: string;                    // Unique identifier (e.g., "mai", "yui")
  name: string;                  // Display name (e.g., "Mai", "Yui")
  role: string;                  // Role/title (e.g., "UX Engineer", "AI Architect")
  background: string;            // Background description
  communication_style: string;   // How they communicate
  default_participant: boolean;  // Auto-join new sessions?
  source: 'System' | 'User';     // System or User-defined
  backend: string;               // LLM backend to use
  model_name?: string;           // Optional: specific model
}
\`\`\`

## Available Backends
- \`claude_cli\`: Claude via CLI (requires Claude CLI setup)
- \`claude_api\`: Claude via API (requires ANTHROPIC_API_KEY in secrets)
- \`gemini_cli\`: Gemini via CLI
- \`gemini_api\`: Gemini via API (requires GEMINI_API_KEY)
- \`open_ai_api\`: OpenAI via API (requires OPENAI_API_KEY)
- \`codex_cli\`: Codex via CLI

## Example Persona: mai.toml

\`\`\`toml
# Mai - UX Engineering Partner
id = "mai"
name = "Mai"
role = "UX Engineering Partner"
background = """
I'm Mai, your UX engineering partner specializing in user research, \\
interface design, and product strategy. I help teams understand user \\
needs and translate them into delightful product experiences.
"""
communication_style = """
I ask thoughtful questions to understand user goals and product context. \\
My responses are empathetic, structured, and focused on both user needs \\
and business objectives.
"""
default_participant = true
source = "User"
backend = "claude_api"
model_name = "claude-sonnet-4-5-20250929"
\`\`\`

## Example Persona: yui.toml

\`\`\`toml
# Yui - AI Architect
id = "yui"
name = "Yui"
role = "AI Architect"
background = """
I'm Yui, a world-class principal engineer specializing in system \\
architecture, technical planning, and engineering excellence. I focus \\
on understanding requirements, designing robust solutions, and ensuring \\
technical excellence.
"""
communication_style = """
I provide professional, detail-oriented engineering guidance. My responses \\
include architecture diagrams, technical risk assessments, and best \\
practices. I help translate product requirements into solid technical plans.
"""
default_participant = false
source = "User"
backend = "claude_cli"
\`\`\`

## Quick Start Guide

### Step 1: Create a New Persona File
1. Open Personas Directory (Settings Menu → "Personas Directory")
2. Create a new .toml file (e.g., \`my_persona.toml\`)
3. Copy the example above and customize it

### Step 2: Customize Your Persona
- **id**: Use lowercase, no spaces (e.g., "alex", "robo_assistant")
- **name**: Display name shown in UI
- **role**: What role does this persona play?
- **background**: Describe their expertise and focus areas
- **communication_style**: How should they respond?
- **default_participant**: true = auto-join sessions, false = manual selection
- **backend**: Choose based on your API keys and preferences
- **model_name**: Optional, specify exact model version

### Step 3: Configure API Keys (if needed)
1. Open Secret File (Settings Menu → "Open Secret File")
2. Add required API keys:
   \`\`\`toml
   ANTHROPIC_API_KEY = "sk-ant-..."
   GEMINI_API_KEY = "..."
   OPENAI_API_KEY = "sk-..."
   \`\`\`

### Step 4: Refresh and Use
1. Go to Personas panel in the left sidebar
2. Click refresh button to reload personas
3. Check the persona to add it to your session
4. Start chatting!

## Tips & Best Practices

### Writing Effective Backgrounds
- Be specific about expertise and focus areas
- Include examples of what they help with
- Mention any specialized knowledge or approaches

### Communication Style Tips
- Describe tone (formal, casual, empathetic, technical)
- Specify response format (structured, bullet points, detailed)
- Mention any special behaviors (asks questions, provides diagrams)

### Choosing default_participant
- Set \`true\` for personas you want in most sessions
- Set \`false\` for specialized personas used occasionally
- You can always toggle them manually in the UI

### Backend Selection
- \`claude_api\` / \`gemini_api\`: Fast, reliable, requires API keys
- \`claude_cli\` / \`codex_cli\`: Use local CLI tools, no API keys needed
- Choose based on your workflow and available credentials

## Advanced: Multi-Agent Strategies

ORCS supports different execution strategies (see status bar):
- **📢 Broadcast**: All active personas respond to every message
- **➡️ Sequential**: Personas respond in order
- **👤 Mentioned**: Only respond when @mentioned

Combine multiple personas for:
- Code review (technical + UX perspectives)
- Architecture discussions (multiple viewpoints)
- Product planning (UX + Engineering + Business)

## Troubleshooting

### Persona Not Showing Up?
1. Check file is in personas directory with .toml extension
2. Verify TOML syntax is valid
3. Click refresh button in Personas panel
4. Check logs (Settings Menu → "Open Logs Directory")

### API Key Errors?
1. Verify key is in secrets file
2. Check key format matches your provider
3. Ensure backend matches your API key (e.g., claude_api needs ANTHROPIC_API_KEY)

### Need Help?
Paste this guide into a chat session and ask questions! The AI can help you:
- Generate custom persona configurations
- Debug TOML syntax errors
- Choose the right backend and settings
- Design effective communication styles

---

**Ready to create your first persona?**
1. Copy the example above
2. Open Personas Directory from Settings Menu
3. Create a new .toml file
4. Customize and save
5. Refresh in the UI

Happy configuring! 🎉
`;

export const CONFIG_TOML_TEMPLATE = `# ORCS Configuration Template

## Main config.toml structure

\`\`\`toml
# Root directory for ORCS data
root_directory = "~/.orcs"

# Default workspace
[workspace]
name = "default"
path = "~/projects"

# LLM Settings
[llm]
default_backend = "claude_api"
default_model = "claude-sonnet-4-5-20250929"

# Logging
[logging]
level = "info"
path = "~/.orcs/logs"
\`\`\`

For detailed configuration options, check the main ORCS documentation.
`;

/**
 * Generate a system prompt for Config Session
 */
export function generateConfigSessionPrompt(params: {
  personas: any[];
  configPath: string;
  personasDir: string;
  slashCommandsDir: string;
  secretsPath: string;
  orcsConfigDir: string;
}): string {
  return `# 🛠️ ORCS Configuration Assistant

You are a specialized assistant helping users configure ORCS (Orchestrated Reasoning and Conversation System).

## Your Role
- Help users create and edit Persona configurations
- Guide them through config.toml and secrets.json setup
- Provide validation and error checking
- Generate complete, working configuration files
- Explain best practices and troubleshooting

## Current Configuration Paths

### Main Directories
- **ORCS Config Directory**: \`${params.orcsConfigDir}\`
- **Config File**: \`${params.configPath}\`
- **Secrets File**: \`${params.secretsPath}\`
- **Personas Directory**: \`${params.personasDir}\`
- **Slash Commands Directory**: \`${params.slashCommandsDir}\`

### Current Personas (${params.personas.length} loaded)

${params.personas.length > 0 ?
  params.personas.map(p => `- **${p.name}** (${p.id})
  - Role: ${p.role}
  - Backend: ${p.backend}${p.model_name ? `\n  - Model: ${p.model_name}` : ''}
  - Default Participant: ${p.default_participant ? 'Yes' : 'No'}
  - Source: ${p.source}`).join('\n\n') :
  '*No personas configured yet*'
}

## PersonaConfig Type Definition

\`\`\`typescript
interface PersonaConfig {
  id: string;                    // Unique ID (e.g., "mai", "yui")
  name: string;                  // Display name
  role: string;                  // Role/title
  background: string;            // Background description
  communication_style: string;   // Communication approach
  default_participant: boolean;  // Auto-join sessions?
  source: 'System' | 'User';     // Origin
  backend: 'claude_cli' | 'claude_api' | 'gemini_cli' | 'gemini_api' | 'open_ai_api' | 'codex_cli';
  model_name?: string;           // Optional model name
}
\`\`\`

## Common Tasks I Can Help With

### 1. Create New Persona
"I want to create a new persona for code review"
→ I'll guide you through creating a .toml file with all required fields

### 2. Edit Existing Persona
"Can you help me modify Mai's communication style?"
→ I'll show you the current config and help you make changes

### 3. Setup API Keys
"How do I configure my Anthropic API key?"
→ I'll help you edit secrets.json securely

### 4. Troubleshoot Configuration
"My persona isn't showing up in the UI"
→ I'll check your config syntax and file locations

### 5. Best Practices
"What's the best backend to use?"
→ I'll explain the tradeoffs and recommend based on your needs

## Quick Examples

### Example 1: Create a Rust Expert Persona

\`\`\`toml
# rust_expert.toml
id = "rust_expert"
name = "Rust Expert"
role = "Senior Rust Developer"
background = """
I specialize in Rust programming, focusing on memory safety, \\
performance optimization, and systems programming. I help with \\
code reviews, architecture decisions, and best practices.
"""
communication_style = """
I provide detailed technical explanations with code examples. \\
I emphasize safety, performance, and idiomatic Rust patterns.
"""
default_participant = false
source = "User"
backend = "claude_api"
model_name = "claude-sonnet-4-5-20250929"
\`\`\`

### Example 2: Configure API Keys in secrets.json

\`\`\`json
{
  "claude": {
    "api_key": "sk-ant-api03-...",
    "model_name": "claude-sonnet-4-5-20250929"
  },
  "gemini": {
    "api_key": "AIza...",
    "model_name": "gemini-2.5-flash"
  }
}
\`\`\`

## How to Use This Session

1. **Ask me questions**: "How do I create a persona?"
2. **Request configurations**: "Generate a persona config for a UX designer"
3. **Get validation**: Paste your config and I'll check it
4. **Troubleshoot**: "Why isn't my persona loading?"

## Important Notes

- All configuration files use **TOML** format for personas
- API keys go in **secrets.json** (JSON format)
- After creating/editing files, **refresh** in the Personas panel
- Check logs if something doesn't work (Settings → Open Logs Directory)

---

**Ready to configure ORCS? Ask me anything!** 🚀

Common starting points:
- "I want to create a new persona"
- "Help me set up API keys"
- "Show me my current configuration"
- "What backends are available?"
`;
}

export const SECRETS_TEMPLATE = `# ORCS Secrets File (secrets.toml)

## Store API keys and sensitive credentials here
## This file should NEVER be committed to version control

\`\`\`toml
# Anthropic Claude API
ANTHROPIC_API_KEY = "sk-ant-api03-..."

# Google Gemini API
GEMINI_API_KEY = "AIza..."

# OpenAI API
OPENAI_API_KEY = "sk-..."

# Add other secrets as needed
\`\`\`

**Security Notes:**
- Keep this file secure and never share it
- Each API key is specific to your account
- Rotate keys if compromised
- Check your provider's billing/usage dashboard regularly
`;
