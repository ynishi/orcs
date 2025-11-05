# AI File Save Feature - Usage Guide

## Overview

AIペルソナがファイル内容を提示する際、ユーザーがワンクリックで保存できる機能です。

## For AI Personas (System Prompt Instructions)

### How to Enable File Save Button

When providing file content that should be saveable, use one of these markdown formats:

**Full format (recommended):**
````markdown
```<language>:path=<absolute_path>:saveable
<file content here>
```
````

**Simple format (also supported):**
````markdown
```<language>:filename.ext
<file content here>
```
````

The simple format is easier to use but requires the user to edit the path to add the full directory.

### Examples

#### Example 1: TOML Configuration File

````markdown
Here's your persona configuration file:

```toml:path=/Users/username/Library/Application Support/orcs/data/orcs/personas/ai_architect.toml:saveable
id = "ai_architect"
name = "Ai"
role = "AI Technology Architect"

background = """
I'm a specialist in AI/ML systems architecture...
"""

communication_style = """
Technical yet accessible...
"""

default_participant = false
source = "User"
backend = "claude_api"
model_name = "claude-3-5-sonnet-20240620"
```

Click the "Save to file" button above to save this configuration.
````

#### Example 2: Python Script

````markdown
Here's a utility script:

```python:path=/Users/username/projects/my-project/utils/helper.py:saveable
def process_data(data):
    """Process the input data."""
    return [x * 2 for x in data]

if __name__ == "__main__":
    result = process_data([1, 2, 3, 4, 5])
    print(result)
```
````

#### Example 3: JSON Configuration

````markdown
```json:path=/Users/username/.config/myapp/config.json:saveable
{
  "apiKey": "your-api-key-here",
  "endpoint": "https://api.example.com",
  "timeout": 30000
}
```
````

#### Example 4: Simple Format (Filename Only)

````markdown
Quick script for you:

```python:hello.py
def greet(name):
    print(f"Hello, {name}!")

greet("World")
```

The path will default to "hello.py" - you can edit it to add the full directory path.
````

### Syntax Breakdown

#### Full Format

1. **Language**: Specify the programming language for syntax highlighting
   - Example: `toml`, `python`, `json`, `yaml`, `rust`, `javascript`, etc.

2. **Path**: Absolute path where the file should be saved
   - Format: `:path=/absolute/path/to/file.ext`
   - Recommended: Use absolute paths (starting with `/` on Unix or `C:\` on Windows)
   - Parent directories will be created automatically

3. **Saveable Flag**: Indicates this code block should show a save button
   - Format: `:saveable`
   - Can be omitted if using `:path=...`

#### Simple Format

1. **Language**: Programming language
2. **Filename**: Just the filename with extension
   - Format: `:filename.ext`
   - Example: `:hello.py`, `:config.toml`
   - Automatically enables save button
   - User must edit to add full directory path (yellow warning will appear)

### Best Practices

1. **Prefer absolute paths when possible**
   ```
   ✅ Best:  /Users/username/projects/orcs/config.toml
   ⚠️ OK:    config.toml (user will edit to add path)
   ❌ Bad:   ~/config.toml  (won't expand, use full path)
   ```

2. **Expand user paths manually**
   - If suggesting user-specific paths, use the actual expanded path
   - Example: `/Users/yutakanishimura/...` instead of `~/...`

3. **Provide context before the code block**
   - Explain what the file is for
   - Mention any prerequisites or next steps
   - Example:
     ```markdown
     Save this configuration file to your personas directory:

     ```toml:path=/path/to/file.toml:saveable
     ...
     ```

     After saving, refresh the personas list in Settings.
     ```

4. **Use appropriate file extensions**
   - `.toml` for persona configs
   - `.py` for Python scripts
   - `.json` for JSON data
   - `.rs` for Rust code
   - etc.

### Full Example Response

````markdown
Great! Let me help you create a new persona configuration.

Here's the complete configuration file for your "DataScientist" persona:

```toml:path=/Users/yutakanishimura/Library/Application Support/orcs/data/orcs/personas/data_scientist.toml:saveable
id = "data_scientist"
name = "Data Scientist Dan"
role = "Data Science & ML Engineering Specialist"

background = """
Expert in data analysis, machine learning, and statistical modeling. \
Proficient in Python, R, and various ML frameworks. Experienced in \
building and deploying production ML systems.
"""

communication_style = """
Analytical and detail-oriented. Explains statistical concepts clearly \
with practical examples. Provides code snippets and visualizations \
to support recommendations.
"""

default_participant = false
source = "User"
backend = "claude_api"
model_name = "claude-3-5-sonnet-20240620"
```

### Next Steps

1. Click the **"Save to file"** button above
2. Go to Settings → Personas
3. Click the refresh icon to reload personas
4. Find "Data Scientist Dan" in the list
5. Click "Add to Session" to start collaborating!

You can edit the path before saving if you want to use a different location.
````

## For Users

### How to Save Files from AI Responses

1. **Look for the green-bordered code block**
   - AI responses with saveable files will show a special code block with a green border
   - The file path is displayed at the top

2. **Edit the path (optional)**
   - Click the edit icon (✏️) next to the path
   - Modify the destination path
   - Click the checkmark (✓) to confirm or X to cancel

3. **Click "Save to file"**
   - Click the green "Save to file" button
   - A notification will confirm the save was successful
   - The file and any parent directories will be created automatically

4. **File is ready!**
   - The file is now saved to your filesystem
   - The operation is logged in your session history

### Features

- ✅ **Syntax highlighting** - Code is displayed with proper syntax highlighting
- ✅ **Copy button** - Copy the code to clipboard without saving
- ✅ **Path editing** - Modify the destination path before saving
- ✅ **Auto-create directories** - Parent folders are created automatically
- ✅ **Session logging** - Save operations are recorded in session history
- ✅ **Error handling** - Clear error messages if save fails

### Keyboard Shortcuts

- **Copy code**: Click the copy button (📋)
- **Save file**: Click "Save to file" button (no keyboard shortcut yet)

### Troubleshooting

#### "Path must be absolute" error
- Make sure the path starts with `/` (Unix) or drive letter (Windows)
- Example: `/Users/name/file.txt` not `~/file.txt`

#### "Failed to create directory" error
- Check that you have write permissions to the parent directory
- Verify the path is valid for your operating system

#### "Failed to write file" error
- Check disk space
- Verify you have write permissions
- Make sure the file isn't locked by another application

## Implementation Details

### Architecture

```
User sees AI response
    ↓
MarkdownRenderer parses code blocks
    ↓
Detects :saveable flag and :path metadata
    ↓
Renders SaveableCodeBlock component
    ↓
User clicks "Save to file"
    ↓
Frontend calls save_code_snippet Tauri command
    ↓
Backend validates path and writes file
    ↓
Session history is updated
    ↓
Success notification shown to user
```

### Security

- **Path validation**: Only absolute paths are accepted
- **Directory creation**: Parent directories created with appropriate permissions
- **Session logging**: All file saves are logged for audit trail
- **No execution**: Saved files do not automatically execute

---

**Last Updated**: 2025-11-05
