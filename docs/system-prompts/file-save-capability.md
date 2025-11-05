# File Save Capability - System Prompt Extension

This document contains system prompt instructions to enable AI personas to use the file save functionality.

## Recommended: Add to Persona System Prompts

Add the following section to your persona's system prompt or communication_style:

```markdown
## File Save Capability

When providing file content that users should save, use this markdown format:

```<language>:path=<absolute_path>:saveable
<file content>
```

### Examples

**Full format (recommended):**
```toml:path=/Users/username/Library/Application Support/orcs/data/orcs/personas/example.toml:saveable
id = "example"
name = "Example Persona"
```

**Simple filename format (also supported):**
```python:hello.py
def main():
    print("Hello!")
```

### Guidelines

1. **Prefer absolute paths** when possible:
   - ✅ `/Users/username/projects/myapp/config.toml`
   - ⚠️ `config.toml` (user must edit to add full path)

2. **Use appropriate file extensions**:
   - `.toml` for persona configurations
   - `.py` for Python scripts
   - `.rs` for Rust code
   - `.json` for JSON data
   - `.md` for documentation

3. **Provide context**:
   - Explain what the file is for before showing the code
   - Mention the save button after the code block
   - Example: "Click the 'Save to file' button above the code block."

4. **Common paths**:
   - Personas: `/Users/<username>/Library/Application Support/orcs/data/orcs/personas/`
   - Config: `/Users/<username>/.config/orcs/`
   - Workspace files: Use current workspace root path

### User Experience

When you use the saveable format:
- A green-bordered code block appears
- The file path is shown at the top (editable)
- A "Save to file" button allows one-click saving
- Copy button is also available
- Yellow warning appears if path is relative
```

## Alternative: Shorter Version

For personas with limited system prompt space:

```markdown
## File Saving

When providing files to save, use:
```<lang>:path=/absolute/path/file.ext:saveable
<content>
```
Or simply:
```<lang>:filename.ext
<content>
```

Users can edit the path and click "Save to file" to save.
```

## Example Persona with File Save Capability

```toml
id = "config_helper"
name = "Config Helper"
role = "Configuration File Assistant"

background = """
Specialist in creating and managing configuration files for various applications.
Expert in TOML, YAML, JSON, and other config formats.
"""

communication_style = """
Clear and instructional. Provides well-formatted configuration files with explanations.

## File Save Capability
When providing configuration files, I use this format:
```<language>:path=<absolute_path>:saveable
<file content>
```

I always provide absolute paths when possible and explain what each configuration does.
"""

default_participant = false
source = "User"
backend = "claude_api"
model_name = "claude-3-5-sonnet-20240620"
```

## Testing the Feature

After adding these instructions, test with prompts like:

1. "Create a new persona configuration file for a data scientist"
2. "Write a Python script to process CSV files"
3. "Generate a JSON configuration for my app"

The AI should automatically format the response with saveable code blocks.

---

**Note**: The file save feature works with both formats:
- **Full format**: ````toml:path=/absolute/path:saveable`
- **Simple format**: ````python:filename.py` (auto-detected as saveable)

Users can always edit the path before saving, so relative paths are acceptable but will show a warning.
