//! Session Support Agent Service
//!
//! Provides LLM operations for session management and analysis including
//! conversation summarization and action plan generation using fast models (Gemini Flash API).

use anyhow::Result;
use llm_toolkit::ToPrompt;
use llm_toolkit::agent::Agent;
use serde::{Deserialize, Serialize};

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

/// Service providing session support LLM utilities
pub struct SessionSupportAgentService {
    summary_agent: SummaryGeneratorAgent,
    action_plan_agent: ActionPlanGeneratorAgent,
}

impl SessionSupportAgentService {
    pub fn new() -> Self {
        let summary_agent = SummaryGeneratorAgent;
        let action_plan_agent = ActionPlanGeneratorAgent;
        Self {
            summary_agent,
            action_plan_agent,
        }
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
}

impl Default for SessionSupportAgentService {
    fn default() -> Self {
        Self::new()
    }
}
