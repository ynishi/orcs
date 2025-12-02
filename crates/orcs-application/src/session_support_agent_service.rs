//! Session Support Agent Service
//!
//! Provides LLM operations for session management and analysis including
//! conversation summarization and action plan generation using fast models (Gemini Flash API).

use anyhow::Result;
use llm_toolkit::ToPrompt;
use llm_toolkit::agent::Agent;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Response structure for conversation summary
#[derive(Debug, Clone, Serialize, Deserialize, ToPrompt)]
#[prompt(mode = "full")]
pub struct SummaryResponse {
    /// Concise summary of the conversation
    pub summary: String,

    /// Key points discussed (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_points: Option<Vec<String>>,

    /// Suggested next actions (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_actions: Option<Vec<String>>,
}

/// Response structure for action plan
#[derive(Debug, Clone, Serialize, Deserialize, ToPrompt)]
#[prompt(mode = "full")]
pub struct ActionPlanResponse {
    /// Title of the action plan
    pub title: String,

    /// Detailed description
    pub description: String,

    /// Implementation steps
    pub steps: Vec<ActionStep>,

    /// Estimated complexity (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity: Option<String>,

    /// Required technologies/tools (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technologies: Option<Vec<String>>,
}

/// Individual action step in the plan
#[derive(Debug, Clone, Serialize, Deserialize, ToPrompt)]
#[prompt(mode = "full")]
pub struct ActionStep {
    /// Step number
    pub step_number: usize,

    /// Step title
    pub title: String,

    /// Detailed description
    pub description: String,

    /// Acceptance criteria (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acceptance_criteria: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, ToPrompt, Default)]
#[prompt(
    template = r#"Analyze this conversation and generate a comprehensive summary:

{{ conversation }}

Requirements:
- Provide a concise summary of the entire conversation
- Extract key points discussed
- Suggest logical next actions based on the conversation flow

Output a JSON object matching this schema:
{{ output_schema }}

IMPORTANT: Output ONLY valid JSON, no markdown formatting or code blocks."#
)]
struct SummaryGenerationRequest {
    /// The conversation content to summarize
    conversation: String,

    /// Output schema for SummaryResponse
    output_schema: String,
}

#[derive(Debug, Clone, Serialize, ToPrompt, Default)]
#[prompt(
    template = r#"Based on this conversation, create a detailed implementation plan in Markdown format:

{{ conversation }}

Requirements:
- Generate a clear, actionable title
- Provide a comprehensive description
- Break down into concrete implementation steps with detailed descriptions
- Include acceptance criteria for each step (as checkboxes)
- Estimate complexity level (Low/Medium/High/Very High)
- List required technologies and tools

Focus on practical, implementable actions. Each step should be specific and measurable.
This plan will be used for Task execution, so make it easy to copy and execute.

Output format (GitHub Issue style Markdown):
# 📋 [Title]

## Description

[Detailed description of what needs to be implemented]

## Implementation Steps

### Step 1: [Step Title]

[Step description]

**Acceptance Criteria:**
- [ ] Criterion 1
- [ ] Criterion 2

### Step 2: [Step Title]

[Step description]

**Acceptance Criteria:**
- [ ] Criterion 1
- [ ] Criterion 2

## Complexity

[Low/Medium/High/Very High]

## Technologies/Tools

- Technology 1
- Technology 2

IMPORTANT: Output ONLY Markdown, no JSON or code blocks wrapping."#
)]
struct ActionPlanGenerationRequest {
    /// The conversation content to create action plan from
    conversation: String,

    /// Output schema for ActionPlanResponse (for future JSON parsing)
    output_schema: String,
}

#[derive(Debug, Clone, Serialize, ToPrompt, Default)]
#[prompt(
    template = r#"Extract reusable expertise and patterns from this conversation for future use:

{{ conversation }}

Requirements:
- Identify the core skill or knowledge demonstrated
- Extract generalizable patterns and best practices
- Create a reusable prompt template
- Specify when and how to apply this expertise
- List relevant technologies and concepts

Focus on creating a reference that can be reused as a SlashCommand, Persona, or general knowledge base entry.

Output format (Markdown):
# 💡 [Skill/Expertise Name]

## Overview

[Brief description of what this expertise covers]

## Core Concepts

[Key concepts and patterns extracted from the conversation]

## When to Use

- [Use case 1]
- [Use case 2]
- [Use case 3]

## Reusable Prompt Template

```
[Template with placeholders for reuse]
```

## Example Usage

[Concrete example showing how to apply this expertise]

## Related Technologies

- [Technology 1]
- [Technology 2]

## Best Practices

- [Practice 1]
- [Practice 2]

IMPORTANT: Output ONLY Markdown, no JSON or code blocks wrapping."#
)]
struct ExpertiseGenerationRequest {
    /// The conversation content to extract expertise from
    conversation: String,
}

#[derive(Debug, Clone, Serialize, ToPrompt, Default)]
#[prompt(
    template = r#"Create a high-quality, comprehensive Concept/Design Issue document from this conversation.

{{ conversation }}

Requirements:
- DO NOT summarize or omit details - include ALL technical decisions and reasoning discussed
- Capture the complete evolution of the discussion, including iterations and refinements
- Document the final decisions with full context and rationale
- Include implementation details, edge cases, and considerations discussed
- Preserve technical depth - this is NOT a summary, this is a complete reference document
- Use professional technical writing style suitable for design documentation

This document should serve as the authoritative reference for the concept/design, capturing everything discussed without losing information through summarization.

Output format (Markdown):
# 📋 [Concept/Design Title]

## Overview

[Comprehensive description of what this concept/design addresses]

## Background & Context

[Why this was needed, what problem it solves, relevant context]

## Design Discussion & Evolution

[Complete record of how the design evolved through the conversation:
- Initial proposals and their reasoning
- Iterations and refinements discussed
- Alternative approaches considered and why they were rejected
- Key insights that shaped the final design]

## Final Design Decision

[The complete, detailed final design decision with:
- Architecture/structure chosen
- Technical implementation details
- Data models, interfaces, or APIs
- Integration points and dependencies]

## Implementation Details

[Specific implementation guidance:
- Code patterns to use
- Configuration requirements
- Migration/deployment considerations
- Performance implications]

## Edge Cases & Considerations

[All edge cases, limitations, and special considerations discussed:
- Known limitations
- Potential issues and mitigations
- Future enhancement opportunities]

## Testing Strategy

[How to verify the implementation works correctly]

## References

[Related documentation, similar patterns, or resources mentioned]

IMPORTANT: Output ONLY Markdown, no JSON or code blocks wrapping. Capture EVERYTHING - do not summarize."#
)]
struct ConceptIssueGenerationRequest {
    /// The conversation content to create concept/design issue from
    conversation: String,
}

/// Lightweight agent for generating conversation summaries using Gemini Flash API
#[derive(llm_toolkit::Agent)]
#[agent(
    expertise = "Analyze conversations and generate comprehensive summaries with key points and next actions. Focus on clarity and actionability.",
    output = "SummaryResponse",
    default_inner = "orcs_interaction::GeminiApiAgent"
)]
struct SummaryGeneratorAgent;

/// Lightweight agent for generating action plans using Gemini Flash API
#[derive(llm_toolkit::Agent)]
#[agent(
    expertise = "Create detailed, implementable action plans from conversations. Break down complex tasks into concrete steps with acceptance criteria.",
    output = "String",
    default_inner = "orcs_interaction::GeminiApiAgent"
)]
struct ActionPlanGeneratorAgent;

/// Lightweight agent for generating expertise from Gemini Flash API
#[derive(llm_toolkit::Agent)]
#[agent(
    expertise = "Extract reusable expertise and patterns from conversations. Create generalizable knowledge for SlashCommands and Personas.",
    output = "String",
    default_inner = "orcs_interaction::GeminiApiAgent"
)]
struct ExpertiseGeneratorAgent;

/// Lightweight agent for generating comprehensive concept/design issue documents
#[derive(llm_toolkit::Agent)]
#[agent(
    expertise = "Create comprehensive, high-quality Concept/Design Issue documents from conversations. Capture ALL technical details, design evolution, and implementation considerations without summarization.",
    output = "String",
    default_inner = "orcs_interaction::GeminiApiAgent"
)]
struct ConceptIssueGeneratorAgent;

/// Service providing session support LLM utilities
pub struct SessionSupportAgentService {
    summary_agent: SummaryGeneratorAgent,
    action_plan_agent: ActionPlanGeneratorAgent,
    expertise_agent: ExpertiseGeneratorAgent,
    concept_issue_agent: ConceptIssueGeneratorAgent,
}

impl SessionSupportAgentService {
    pub fn new() -> Self {
        let summary_agent = SummaryGeneratorAgent;
        let action_plan_agent = ActionPlanGeneratorAgent;
        let expertise_agent = ExpertiseGeneratorAgent;
        let concept_issue_agent = ConceptIssueGeneratorAgent;
        Self {
            summary_agent,
            action_plan_agent,
            expertise_agent,
            concept_issue_agent,
        }
    }

    /// Generate summary with custom agent configuration
    ///
    /// # Arguments
    ///
    /// * `thread_content` - The conversation thread to summarize
    /// * `backend` - Backend type (e.g., "gemini_api", "claude_api", "open_ai_api")
    /// * `model_name` - Optional model name override
    /// * `gemini_thinking_level` - Optional Gemini thinking level (LOW/MEDIUM/HIGH)
    /// * `gemini_google_search` - Optional Google Search enablement
    /// * `cancel_flag` - Optional atomic flag to check for cancellation
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Generated summary in markdown format
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let summary = service.generate_summary_with_config(
    ///     thread_content,
    ///     "gemini_api",
    ///     Some("gemini-3-pro-preview"),
    ///     Some("HIGH"),
    ///     Some(true),
    ///     None,
    /// ).await?;
    /// ```
    pub async fn generate_summary_with_config(
        thread_content: &str,
        backend: &str,
        model_name: Option<&str>,
        gemini_thinking_level: Option<&str>,
        gemini_google_search: Option<bool>,
        cancel_flag: Option<Arc<AtomicBool>>,
    ) -> Result<String> {
        use llm_toolkit::agent::Agent;
        use llm_toolkit::prompt::ToPrompt;
        use orcs_interaction::{ClaudeApiAgent, GeminiApiAgent, OpenAIApiAgent};

        // Check cancellation before starting
        if let Some(ref flag) = cancel_flag {
            if flag.load(Ordering::SeqCst) {
                return Err(anyhow::anyhow!("Operation cancelled"));
            }
        }

        // Create typed request
        let request = SummaryGenerationRequest {
            conversation: thread_content.to_string(),
            output_schema: SummaryResponse::prompt_schema(),
        };
        let prompt = request.to_prompt();

        // Execute with configured agent and parse JSON response
        let response_str: String = match backend {
            "gemini_api" => {
                let mut agent = GeminiApiAgent::try_from_env().await?;
                if let Some(model) = model_name {
                    agent = agent.with_model(model);
                }
                if let Some(level) = gemini_thinking_level {
                    agent = agent.with_thinking_level(level);
                }
                if let Some(search) = gemini_google_search {
                    agent = agent.with_google_search(search);
                }
                agent.execute(prompt.as_str().into()).await?
            }
            "claude_api" => {
                let mut agent = ClaudeApiAgent::try_from_env().await?;
                if let Some(model) = model_name {
                    agent = agent.with_model(model);
                }
                agent.execute(prompt.as_str().into()).await?
            }
            "open_ai_api" => {
                let mut agent = OpenAIApiAgent::try_from_env().await?;
                if let Some(model) = model_name {
                    agent = agent.with_model(model);
                }
                agent.execute(prompt.as_str().into()).await?
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported backend: {}. Supported: gemini_api, claude_api, open_ai_api",
                    backend
                ));
            }
        };

        // Check cancellation after LLM execution
        if let Some(ref flag) = cancel_flag {
            if flag.load(Ordering::SeqCst) {
                return Err(anyhow::anyhow!("Operation cancelled"));
            }
        }

        // Parse JSON response
        let response: SummaryResponse = serde_json::from_str(&response_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse summary response: {}", e))?;

        // Format response as markdown
        let mut markdown = format!("# 📝 Conversation Summary\n\n{}\n", response.summary);

        if let Some(key_points) = response.key_points {
            markdown.push_str("\n## Key Points\n\n");
            for point in key_points {
                markdown.push_str(&format!("- {}\n", point));
            }
        }

        if let Some(next_actions) = response.next_actions {
            markdown.push_str("\n## Suggested Next Actions\n\n");
            for (idx, action) in next_actions.iter().enumerate() {
                markdown.push_str(&format!("{}. {}\n", idx + 1, action));
            }
        }

        Ok(markdown)
    }

    /// Generate action plan with custom agent configuration
    pub async fn generate_action_plan_with_config(
        thread_content: &str,
        backend: &str,
        model_name: Option<&str>,
        gemini_thinking_level: Option<&str>,
        gemini_google_search: Option<bool>,
        cancel_flag: Option<Arc<AtomicBool>>,
    ) -> Result<String> {
        use llm_toolkit::agent::Agent;
        use llm_toolkit::prompt::ToPrompt;
        use orcs_interaction::{ClaudeApiAgent, GeminiApiAgent, OpenAIApiAgent};

        // Check cancellation before starting
        if let Some(ref flag) = cancel_flag {
            if flag.load(Ordering::SeqCst) {
                return Err(anyhow::anyhow!("Operation cancelled"));
            }
        }

        let request = ActionPlanGenerationRequest {
            conversation: thread_content.to_string(),
            output_schema: ActionPlanResponse::prompt_schema(),
        };
        let prompt = request.to_prompt();

        let markdown: String = match backend {
            "gemini_api" => {
                let mut agent = GeminiApiAgent::try_from_env().await?;
                if let Some(model) = model_name {
                    agent = agent.with_model(model);
                }
                if let Some(level) = gemini_thinking_level {
                    agent = agent.with_thinking_level(level);
                }
                if let Some(search) = gemini_google_search {
                    agent = agent.with_google_search(search);
                }
                agent.execute(prompt.as_str().into()).await?
            }
            "claude_api" => {
                let mut agent = ClaudeApiAgent::try_from_env().await?;
                if let Some(model) = model_name {
                    agent = agent.with_model(model);
                }
                agent.execute(prompt.as_str().into()).await?
            }
            "open_ai_api" => {
                let mut agent = OpenAIApiAgent::try_from_env().await?;
                if let Some(model) = model_name {
                    agent = agent.with_model(model);
                }
                agent.execute(prompt.as_str().into()).await?
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported backend: {}. Supported: gemini_api, claude_api, open_ai_api",
                    backend
                ));
            }
        };

        // Check cancellation after LLM execution
        if let Some(ref flag) = cancel_flag {
            if flag.load(Ordering::SeqCst) {
                return Err(anyhow::anyhow!("Operation cancelled"));
            }
        }

        Ok(markdown)
    }

    /// Generate expertise with custom agent configuration
    pub async fn generate_expertise_with_config(
        thread_content: &str,
        backend: &str,
        model_name: Option<&str>,
        gemini_thinking_level: Option<&str>,
        gemini_google_search: Option<bool>,
        cancel_flag: Option<Arc<AtomicBool>>,
    ) -> Result<String> {
        use llm_toolkit::agent::Agent;
        use llm_toolkit::prompt::ToPrompt;
        use orcs_interaction::{ClaudeApiAgent, GeminiApiAgent, OpenAIApiAgent};

        // Check cancellation before starting
        if let Some(ref flag) = cancel_flag {
            if flag.load(Ordering::SeqCst) {
                return Err(anyhow::anyhow!("Operation cancelled"));
            }
        }

        let request = ExpertiseGenerationRequest {
            conversation: thread_content.to_string(),
        };
        let prompt = request.to_prompt();

        let markdown: String = match backend {
            "gemini_api" => {
                let mut agent = GeminiApiAgent::try_from_env().await?;
                if let Some(model) = model_name {
                    agent = agent.with_model(model);
                }
                if let Some(level) = gemini_thinking_level {
                    agent = agent.with_thinking_level(level);
                }
                if let Some(search) = gemini_google_search {
                    agent = agent.with_google_search(search);
                }
                agent.execute(prompt.as_str().into()).await?
            }
            "claude_api" => {
                let mut agent = ClaudeApiAgent::try_from_env().await?;
                if let Some(model) = model_name {
                    agent = agent.with_model(model);
                }
                agent.execute(prompt.as_str().into()).await?
            }
            "open_ai_api" => {
                let mut agent = OpenAIApiAgent::try_from_env().await?;
                if let Some(model) = model_name {
                    agent = agent.with_model(model);
                }
                agent.execute(prompt.as_str().into()).await?
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported backend: {}. Supported: gemini_api, claude_api, open_ai_api",
                    backend
                ));
            }
        };

        // Check cancellation after LLM execution
        if let Some(ref flag) = cancel_flag {
            if flag.load(Ordering::SeqCst) {
                return Err(anyhow::anyhow!("Operation cancelled"));
            }
        }

        Ok(markdown)
    }

    /// Generate concept/design issue with custom agent configuration
    ///
    /// # Arguments
    ///
    /// * `thread_content` - The conversation thread to create concept/design issue from
    /// * `backend` - Backend type (e.g., "gemini_api", "claude_api", "open_ai_api")
    /// * `model_name` - Optional model name override
    /// * `gemini_thinking_level` - Optional Gemini thinking level (LOW/MEDIUM/HIGH)
    /// * `gemini_google_search` - Optional Google Search enablement
    /// * `cancel_flag` - Optional atomic flag to check for cancellation
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Generated concept/design issue in markdown format
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let concept_issue = SessionSupportAgentService::generate_concept_issue_with_config(
    ///     thread_content,
    ///     "gemini_api",
    ///     Some("gemini-3-pro-preview"),
    ///     Some("HIGH"),
    ///     Some(true),
    ///     None,
    /// ).await?;
    /// ```
    pub async fn generate_concept_issue_with_config(
        thread_content: &str,
        backend: &str,
        model_name: Option<&str>,
        gemini_thinking_level: Option<&str>,
        gemini_google_search: Option<bool>,
        cancel_flag: Option<Arc<AtomicBool>>,
    ) -> Result<String> {
        use llm_toolkit::agent::Agent;
        use llm_toolkit::prompt::ToPrompt;
        use orcs_interaction::{ClaudeApiAgent, GeminiApiAgent, OpenAIApiAgent};

        // Check cancellation before starting
        if let Some(ref flag) = cancel_flag {
            if flag.load(Ordering::SeqCst) {
                return Err(anyhow::anyhow!("Operation cancelled"));
            }
        }

        let request = ConceptIssueGenerationRequest {
            conversation: thread_content.to_string(),
        };
        let prompt = request.to_prompt();

        let markdown: String = match backend {
            "gemini_api" => {
                let mut agent = GeminiApiAgent::try_from_env().await?;
                if let Some(model) = model_name {
                    agent = agent.with_model(model);
                }
                if let Some(level) = gemini_thinking_level {
                    agent = agent.with_thinking_level(level);
                }
                if let Some(search) = gemini_google_search {
                    agent = agent.with_google_search(search);
                }
                agent.execute(prompt.as_str().into()).await?
            }
            "claude_api" => {
                let mut agent = ClaudeApiAgent::try_from_env().await?;
                if let Some(model) = model_name {
                    agent = agent.with_model(model);
                }
                agent.execute(prompt.as_str().into()).await?
            }
            "open_ai_api" => {
                let mut agent = OpenAIApiAgent::try_from_env().await?;
                if let Some(model) = model_name {
                    agent = agent.with_model(model);
                }
                agent.execute(prompt.as_str().into()).await?
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported backend: {}. Supported: gemini_api, claude_api, open_ai_api",
                    backend
                ));
            }
        };

        // Check cancellation after LLM execution
        if let Some(ref flag) = cancel_flag {
            if flag.load(Ordering::SeqCst) {
                return Err(anyhow::anyhow!("Operation cancelled"));
            }
        }

        Ok(markdown)
    }

    /// Generate summary from conversation thread
    ///
    /// # Arguments
    ///
    /// * `thread_content` - The conversation thread to summarize
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Generated summary in markdown format
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let summary = service.generate_summary(
    ///     "[2024-01-01 10:00:00] User (user):\nHelp me implement a feature...\n---\n..."
    /// ).await?;
    /// ```
    pub async fn generate_summary(&self, thread_content: &str) -> Result<String> {
        use llm_toolkit::prompt::ToPrompt;

        // Create typed request with Jinja2 template
        let request = SummaryGenerationRequest {
            conversation: thread_content.to_string(),
            output_schema: SummaryResponse::prompt_schema(),
        };

        // Generate prompt using ToPrompt derive
        let prompt = request.to_prompt();

        let response: SummaryResponse = self.summary_agent.execute(prompt.as_str().into()).await?;

        // Format response as markdown
        let mut markdown = format!("# 📝 Conversation Summary\n\n{}\n", response.summary);

        if let Some(key_points) = response.key_points {
            markdown.push_str("\n## Key Points\n\n");
            for point in key_points {
                markdown.push_str(&format!("- {}\n", point));
            }
        }

        if let Some(next_actions) = response.next_actions {
            markdown.push_str("\n## Suggested Next Actions\n\n");
            for (idx, action) in next_actions.iter().enumerate() {
                markdown.push_str(&format!("{}. {}\n", idx + 1, action));
            }
        }

        Ok(markdown)
    }

    /// Generate action plan from conversation thread
    ///
    /// # Arguments
    ///
    /// * `thread_content` - The conversation thread to create action plan from
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Generated action plan in markdown format
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let action_plan = service.generate_action_plan(
    ///     "[2024-01-01 10:00:00] User (user):\nI need to add a dark mode toggle...\n---\n..."
    /// ).await?;
    /// ```
    pub async fn generate_action_plan(&self, thread_content: &str) -> Result<String> {
        use llm_toolkit::prompt::ToPrompt;

        // Create typed request with Jinja2 template
        let request = ActionPlanGenerationRequest {
            conversation: thread_content.to_string(),
            output_schema: ActionPlanResponse::prompt_schema(),
        };

        // Generate prompt using ToPrompt derive
        let prompt = request.to_prompt();

        // Agent returns Markdown directly (output = "String")
        let markdown: String = self
            .action_plan_agent
            .execute(prompt.as_str().into())
            .await?;

        Ok(markdown)
    }

    /// Generate expertise from conversation thread
    ///
    /// # Arguments
    ///
    /// * `thread_content` - The conversation thread to extract expertise from
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Generated expertise in markdown format
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let expertise = service.generate_expertise(
    ///     "[2024-01-01 10:00:00] User (user):\nHelp me debug API errors...\n---\n..."
    /// ).await?;
    /// ```
    pub async fn generate_expertise(&self, thread_content: &str) -> Result<String> {
        use llm_toolkit::prompt::ToPrompt;

        // Create typed request with Jinja2 template
        let request = ExpertiseGenerationRequest {
            conversation: thread_content.to_string(),
        };

        // Generate prompt using ToPrompt derive
        let prompt = request.to_prompt();

        // Agent returns Markdown directly (output = "String")
        let markdown: String = self.expertise_agent.execute(prompt.as_str().into()).await?;

        Ok(markdown)
    }

    /// Generate comprehensive Concept/Design Issue from conversation thread
    ///
    /// # Arguments
    ///
    /// * `thread_content` - The conversation thread to create concept/design issue from
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Generated concept/design issue document in markdown format
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let concept_issue = service.generate_concept_issue(
    ///     "[2024-01-01 10:00:00] User (user):\nLet's design a new feature...\n---\n..."
    /// ).await?;
    /// ```
    pub async fn generate_concept_issue(&self, thread_content: &str) -> Result<String> {
        use llm_toolkit::prompt::ToPrompt;

        // Create typed request with Jinja2 template
        let request = ConceptIssueGenerationRequest {
            conversation: thread_content.to_string(),
        };

        // Generate prompt using ToPrompt derive
        let prompt = request.to_prompt();

        // Agent returns Markdown directly (output = "String")
        let markdown: String = self
            .concept_issue_agent
            .execute(prompt.as_str().into())
            .await?;

        Ok(markdown)
    }
}

impl Default for SessionSupportAgentService {
    fn default() -> Self {
        Self::new()
    }
}
