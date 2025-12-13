pub mod claude_api_agent;
pub mod gemini_api_agent;
pub mod local_agents;
pub mod openai_api_agent;

// Re-export API agents for external use
pub use crate::claude_api_agent::ClaudeApiAgent;
pub use crate::gemini_api_agent::GeminiApiAgent;
pub use crate::openai_api_agent::OpenAIApiAgent;
use llm_toolkit::agent::dialogue::{
    Dialogue, DialogueTurn, ExecutionModel, ReactionStrategy, Speaker, TalkStyle,
};
use llm_toolkit::agent::impls::{ClaudeCodeAgent, CodexAgent, GeminiAgent};
use llm_toolkit::agent::persona::Persona as LlmPersona;
use llm_toolkit::agent::{Agent, AgentError, Payload};
use llm_toolkit::attachment::Attachment;
use orcs_core::agent::build_enhanced_path;
use orcs_core::config::EnvSettings;
use orcs_core::persona::{Persona as PersonaDomain, PersonaBackend};
use orcs_core::repository::PersonaRepository;
use orcs_core::session::{
    AppMode, AutoChatConfig, ContextMode, ConversationMessage, ConversationMode, ErrorSeverity,
    MessageMetadata, MessageRole, Plan, Session, SystemEventType,
};
use orcs_core::user::UserService;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Converts a Persona domain model to llm-toolkit Persona.
///
/// Automatically injects runtime capabilities based on the backend type
/// into the communication_style to help the AI understand what it can and cannot do.
fn domain_to_llm_persona(persona: &PersonaDomain) -> LlmPersona {
    use llm_toolkit::agent::persona::VisualIdentity;

    // Inject runtime capabilities into communication style
    let enhanced_communication_style = format!(
        "{}\n\n{}",
        persona.communication_style,
        persona.backend.capabilities_markdown()
    );

    // Create visual identity if icon is present
    let visual_identity = persona.icon.as_ref().map(|icon| {
        let mut identity = VisualIdentity::new(icon.clone());
        // Add base_color if present
        if let Some(ref color) = persona.base_color {
            identity = identity.with_color(color.clone());
        }
        identity
    });

    // Get capabilities from backend
    let capabilities = Some(persona.backend.capabilities());

    LlmPersona {
        name: persona.name.clone(),
        role: persona.role.clone(),
        background: persona.background.clone(),
        communication_style: enhanced_communication_style,
        visual_identity,
        capabilities,
    }
}

/// A single streaming dialogue turn event for frontend consumption.
///
/// This structure represents a unit of communication from the backend to frontend
/// during a dialogue interaction. It uses an enum-based design for type safety and
/// clear semantics.
///
/// # Design
///
/// Uses `#[serde(flatten)]` to flatten the `kind` field into the parent JSON structure,
/// avoiding nested "kind" fields. The `StreamingDialogueTurnKind` enum uses `#[serde(tag = "type")]`
/// to generate a "type" discriminator field.
///
/// # JSON Representation
///
/// ```json
/// // Chunk
/// { "type": "Chunk", "session_id": "...", "timestamp": "...", "author": "...", "content": "..." }
///
/// // Final
/// { "type": "Final", "session_id": "...", "timestamp": "..." }
///
/// // Error
/// { "type": "Error", "session_id": "...", "timestamp": "...", "message": "..." }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingDialogueTurn {
    /// The session ID this turn belongs to (for multi-tab support)
    pub session_id: String,
    /// Timestamp when this turn was created
    pub timestamp: String,
    /// The kind of turn (Chunk, Final, or Error)
    #[serde(flatten)]
    pub kind: StreamingDialogueTurnKind,
}

/// The specific kind of streaming dialogue turn.
///
/// Uses `#[serde(tag = "type")]` to generate a "type" field in JSON for discriminated unions.
/// This enables type-safe handling of different turn types in both Rust and TypeScript.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamingDialogueTurnKind {
    /// A streaming data chunk from an agent
    Chunk {
        /// The author of this chunk (agent name or "USER")
        author: String,
        /// The content of this chunk
        content: String,
    },
    /// Stream completion marker (no more chunks)
    Final,
    /// Error occurred during streaming
    Error {
        /// Error message to display
        message: String,
    },
    /// AutoChat iteration progress update
    AutoChatProgress {
        /// Current iteration number (1-indexed)
        current_iteration: i32,
        /// Maximum iterations configured
        max_iterations: i32,
    },
    /// AutoChat completion notification
    AutoChatComplete {
        /// Total iterations completed
        total_iterations: i32,
    },
}

/// Agent wrapper that delegates to the configured backend.
#[derive(Clone, Debug)]
struct PersonaBackendAgent {
    backend: PersonaBackend,
    model_name: Option<String>,
    gemini_options: Option<orcs_core::persona::GeminiOptions>,
    workspace_root: Arc<RwLock<Option<PathBuf>>>,
    env_settings: Arc<RwLock<EnvSettings>>,
}

impl PersonaBackendAgent {
    fn new(
        backend: PersonaBackend,
        model_name: Option<String>,
        gemini_options: Option<orcs_core::persona::GeminiOptions>,
        workspace_root: Arc<RwLock<Option<PathBuf>>>,
        env_settings: Arc<RwLock<EnvSettings>>,
    ) -> Self {
        Self {
            backend,
            model_name,
            gemini_options,
            workspace_root,
            env_settings,
        }
    }

    /// Executes the agent with optional workspace context.
    ///
    /// # Arguments
    ///
    /// * `payload` - The input payload for the agent
    /// * `workspace_root` - Optional workspace root path (logged but not used for directory changes)
    ///
    /// # Returns
    ///
    /// Returns the agent's output string.
    ///
    /// # Errors
    ///
    /// Returns an error if the agent execution fails
    async fn execute_with_workspace(
        &self,
        payload: Payload,
        workspace_root: Option<PathBuf>,
    ) -> Result<String, AgentError> {
        // Log the intention but do not change the directory
        tracing::info!(
            "[PersonaBackendAgent] Executing with workspace context: {:?} for backend: {:?}",
            workspace_root,
            self.backend
        );

        match self.backend {
            PersonaBackend::ClaudeCli => {
                let mut agent = ClaudeCodeAgent::new()
                    // Pre-approve Edit and Write tools to avoid constant approval prompts
                    .with_args(vec![
                        "--allowed-tools".to_string(),
                        "Edit,Write".to_string(),
                    ]);

                // Set workspace root and enhanced PATH if provided
                if let Some(workspace) = workspace_root {
                    let env_settings = self.env_settings.read().await;
                    let enhanced_path = build_enhanced_path(&workspace, Some(&*env_settings));
                    agent = agent.with_cwd(workspace).with_env("PATH", enhanced_path);
                }
                // Apply model if specified
                if let Some(ref model_str) = self.model_name {
                    tracing::info!("[PersonaBackendAgent] Using Claude model: {}", model_str);
                    agent = agent.with_model_str(model_str);
                }
                agent.execute(payload).await
            }
            PersonaBackend::ClaudeApi => {
                let mut agent = ClaudeApiAgent::try_from_env().await?;
                // Override model if specified
                if let Some(ref model_str) = self.model_name {
                    tracing::info!("[PersonaBackendAgent] Using Claude model: {}", model_str);
                    agent = agent.with_model(model_str);
                }
                agent.execute(payload).await
            }
            PersonaBackend::GeminiCli => {
                let mut agent = GeminiAgent::new();
                // Set workspace root and enhanced PATH if provided
                if let Some(workspace) = workspace_root {
                    let env_settings = self.env_settings.read().await;
                    let enhanced_path = build_enhanced_path(&workspace, Some(&*env_settings));
                    agent = agent.with_cwd(workspace).with_env("PATH", enhanced_path);
                }
                // Apply model if specified
                if let Some(ref model_str) = self.model_name {
                    tracing::info!("[PersonaBackendAgent] Using Gemini model: {}", model_str);
                    agent = agent.with_model_str(model_str);
                }
                agent.execute(payload).await
            }
            PersonaBackend::GeminiApi => {
                let mut agent = GeminiApiAgent::try_from_env().await?;
                // Override model if specified
                if let Some(ref model_str) = self.model_name {
                    tracing::info!("[PersonaBackendAgent] Using Gemini model: {}", model_str);
                    agent = agent.with_model(model_str);
                }
                // Apply Gemini options if specified
                if let Some(ref options) = self.gemini_options {
                    if let Some(ref thinking_level) = options.thinking_level {
                        tracing::info!(
                            "[PersonaBackendAgent] Setting Gemini thinking level: {}",
                            thinking_level
                        );
                        agent = agent.with_thinking_level(thinking_level);
                    }
                    if let Some(google_search) = options.google_search {
                        tracing::info!(
                            "[PersonaBackendAgent] Setting Gemini Google Search: {}",
                            google_search
                        );
                        agent = agent.with_google_search(google_search);
                    }
                }
                agent.execute(payload).await
            }
            PersonaBackend::OpenAiApi => {
                let mut agent = OpenAIApiAgent::try_from_env().await?;
                // Override model if specified
                if let Some(ref model_str) = self.model_name {
                    tracing::info!("[PersonaBackendAgent] Using OpenAI model: {}", model_str);
                    agent = agent.with_model(model_str);
                }
                agent.execute(payload).await
            }
            PersonaBackend::CodexCli => {
                let mut agent = CodexAgent::new();
                // Set workspace root and enhanced PATH if provided
                if let Some(workspace) = workspace_root {
                    let env_settings = self.env_settings.read().await;
                    let enhanced_path = build_enhanced_path(&workspace, Some(&*env_settings));
                    agent = agent.with_cwd(workspace).with_env("PATH", enhanced_path);
                }
                // Apply model if specified
                if let Some(ref model_str) = self.model_name {
                    tracing::info!("[PersonaBackendAgent] Using Codex model: {}", model_str);
                    agent = agent.with_model_str(model_str);
                }
                agent.execute(payload).await
            }
        }
    }
}

#[async_trait::async_trait]
impl Agent for PersonaBackendAgent {
    type Output = String;

    fn expertise(&self) -> &str {
        match self.backend {
            PersonaBackend::ClaudeCli => "Claude CLI persona agent",
            PersonaBackend::ClaudeApi => "Claude API persona agent",
            PersonaBackend::GeminiCli => "Gemini CLI persona agent",
            PersonaBackend::GeminiApi => "Gemini API persona agent",
            PersonaBackend::OpenAiApi => "OpenAI API persona agent",
            PersonaBackend::CodexCli => "Codex CLI persona agent",
        }
    }

    async fn execute(&self, payload: Payload) -> Result<Self::Output, AgentError> {
        // Read workspace_root from shared state
        let workspace_root = self.workspace_root.read().await.clone();
        tracing::info!(
            "[PersonaBackendAgent::execute] Read workspace_root from Arc: {:?}",
            workspace_root
        );
        self.execute_with_workspace(payload, workspace_root).await
    }
}

fn agent_for_persona(
    persona: &PersonaDomain,
    workspace_root: Arc<RwLock<Option<PathBuf>>>,
    env_settings: Arc<RwLock<EnvSettings>>,
) -> Box<dyn Agent<Output = String>> {
    use llm_toolkit::agent::chat::Chat;
    use llm_toolkit::agent::persona::ContextConfig;

    let backend_agent = PersonaBackendAgent::new(
        persona.backend.clone(),
        persona.model_name.clone(),
        persona.gemini_options.clone(),
        workspace_root,
        env_settings,
    );

    let llm_persona = domain_to_llm_persona(persona);
    let mut chat = Chat::new(backend_agent).with_persona(llm_persona);

    // ClaudeCode backend の場合のみ ContextConfig を適用
    if matches!(persona.backend, PersonaBackend::ClaudeCli) {
        let config = ContextConfig {
            recent_messages_count: 20,
            participants_after_context: true, // Participants を Context の後に配置
            include_trailing_prompt: true,
            ..Default::default()
        };
        chat = chat.with_context_config(config);
    }

    chat.with_history(true).build()
}

/// Represents a single message in a dialogue conversation.
///
/// Each message has an author (participant name) and the content of the message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DialogueMessage {
    /// The session ID this message belongs to (for multi-tab support).
    pub session_id: String,
    /// The name of the participant who authored this message.
    pub author: String,
    /// The content of the message.
    pub content: String,
}

/// Result of handling user input in a stateful conversation.
///
/// This enum represents the different outcomes that can occur when processing
/// user input based on the current application mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InteractionResult {
    /// No state change occurred.
    NoOp,
    /// The application mode should be updated to the specified mode.
    ModeChanged(AppMode),
    /// A plan was confirmed and tasks should be dispatched.
    TasksToDispatch {
        /// The list of tasks to be dispatched.
        tasks: Vec<String>,
    },
    /// A new message to be displayed to the user.
    NewMessage(String),
    /// New dialogue messages from multiple participants.
    NewDialogueMessages(Vec<DialogueMessage>),
}

/// Manages user interaction and conversation for a session.
///
/// The `InteractionManager` handles:
/// - Dialogue-based conversations with multiple AI participants
/// - Session state serialization/deserialization
pub struct InteractionManager {
    /// Session ID for this manager instance
    session_id: String,
    /// Session title (user-editable)
    title: Arc<RwLock<String>>,
    /// Session creation timestamp
    created_at: String,
    /// Optional workspace ID for filtering sessions by workspace
    workspace_id: Arc<RwLock<Option<String>>>,
    /// Shared workspace root path for agents (updated when workspace switches)
    agent_workspace_root: Arc<RwLock<Option<PathBuf>>>,
    /// Lazily-initialized dialogue instance
    dialogue: Arc<Mutex<Option<Dialogue>>>,
    /// Raw conversation history per persona (for persistence)
    persona_histories: Arc<RwLock<HashMap<String, Vec<ConversationMessage>>>>,
    /// Repository for persona configurations
    persona_repository: Arc<dyn PersonaRepository>,
    /// Service for retrieving user information
    user_service: Arc<dyn UserService>,
    /// Environment settings for PATH configuration (CLI tools)
    env_settings: Arc<RwLock<EnvSettings>>,
    /// Execution strategy for dialogue
    execution_strategy: Arc<RwLock<ExecutionModel>>,
    /// Active participant persona IDs (restored from session or populated dynamically)
    restored_participant_ids: Arc<RwLock<Option<Vec<String>>>>,
    /// System messages (join/leave notifications, etc.)
    system_messages: Arc<RwLock<Vec<ConversationMessage>>>,
    /// Conversation mode (controls verbosity and style)
    conversation_mode: Arc<RwLock<ConversationMode>>,
    /// Talk style for dialogue context (Brainstorm, Debate, etc.)
    talk_style: Arc<RwLock<Option<TalkStyle>>>,
    /// AutoChat configuration (None means AutoChat is disabled)
    auto_chat_config: Arc<RwLock<Option<AutoChatConfig>>>,
    /// Current iteration in AutoChat mode (None when not running)
    auto_chat_iteration: Arc<RwLock<Option<i32>>>,
    /// Optional prompt extension appended to system prompt
    prompt_extension: Arc<RwLock<Option<String>>>,
    /// Whether this session is muted (AI won't respond to messages)
    is_muted: Arc<RwLock<bool>>,
    /// Context mode for AI interactions (Rich = full context, Clean = expertise only)
    context_mode: Arc<RwLock<ContextMode>>,
    /// Sandbox state for git worktree-based isolated development
    sandbox_state: Arc<RwLock<Option<orcs_core::session::SandboxState>>>,
}

impl InteractionManager {
    /// Creates a new session with empty conversation history.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for this session
    /// * `persona_repository` - Repository for accessing persona configurations
    /// * `user_service` - Service for retrieving user information
    /// * `env_settings` - Environment settings for PATH configuration
    pub fn new_session(
        session_id: String,
        persona_repository: Arc<dyn PersonaRepository>,
        user_service: Arc<dyn UserService>,
        env_settings: EnvSettings,
    ) -> Self {
        let mut persona_histories_map = HashMap::new();

        // Initialize with user's history
        persona_histories_map.insert(user_service.get_user_name(), Vec::new());

        // Initialize with default personas from repository
        // Note: This is a sync context, so we cannot await here.
        // Default persona initialization will be handled later if needed.

        let now = chrono::Utc::now().to_rfc3339();
        let default_title = format!("Session {}", &session_id[..8]);

        Self {
            session_id,
            title: Arc::new(RwLock::new(default_title)),
            created_at: now,
            workspace_id: Arc::new(RwLock::new(None)), // Will be set by the caller if needed
            agent_workspace_root: Arc::new(RwLock::new(None)), // Will be set when workspace is assigned
            dialogue: Arc::new(Mutex::new(None)),
            persona_histories: Arc::new(RwLock::new(persona_histories_map)),
            persona_repository,
            user_service,
            env_settings: Arc::new(RwLock::new(env_settings)),
            execution_strategy: Arc::new(RwLock::new(ExecutionModel::Broadcast)),
            restored_participant_ids: Arc::new(RwLock::new(None)),
            system_messages: Arc::new(RwLock::new(Vec::new())),
            conversation_mode: Arc::new(RwLock::new(ConversationMode::default())),
            talk_style: Arc::new(RwLock::new(None)),
            auto_chat_config: Arc::new(RwLock::new(None)),
            auto_chat_iteration: Arc::new(RwLock::new(None)),
            prompt_extension: Arc::new(RwLock::new(None)),
            is_muted: Arc::new(RwLock::new(false)),
            context_mode: Arc::new(RwLock::new(ContextMode::default())),
            sandbox_state: Arc::new(RwLock::new(None)),
        }
    }

    /// Restores a session from persisted data.
    ///
    /// # Arguments
    ///
    /// * `data` - The session data to restore
    /// * `persona_repository` - Repository for accessing persona configurations
    /// * `user_service` - Service for retrieving user information
    /// * `env_settings` - Environment settings for PATH configuration
    ///
    /// # Note
    ///
    /// This creates new Agent instances. History is stored separately
    /// in persona_histories and included in prompts manually.
    pub fn from_session(
        data: Session,
        persona_repository: Arc<dyn PersonaRepository>,
        user_service: Arc<dyn UserService>,
        env_settings: EnvSettings,
    ) -> Self {
        let restored_ids = if data.active_participant_ids.is_empty() {
            None
        } else {
            Some(data.active_participant_ids)
        };

        Self {
            session_id: data.id,
            title: Arc::new(RwLock::new(data.title)),
            created_at: data.created_at,
            workspace_id: Arc::new(RwLock::new(Some(data.workspace_id))),
            agent_workspace_root: Arc::new(RwLock::new(None)), // Will be resolved and set by the caller
            dialogue: Arc::new(Mutex::new(None)),
            persona_histories: Arc::new(RwLock::new(data.persona_histories)),
            persona_repository,
            user_service,
            env_settings: Arc::new(RwLock::new(env_settings)),
            execution_strategy: Arc::new(RwLock::new(data.execution_strategy)),
            restored_participant_ids: Arc::new(RwLock::new(restored_ids)),
            system_messages: Arc::new(RwLock::new(data.system_messages)),
            conversation_mode: Arc::new(RwLock::new(data.conversation_mode)),
            talk_style: Arc::new(RwLock::new(data.talk_style)),
            auto_chat_config: Arc::new(RwLock::new(data.auto_chat_config)),
            auto_chat_iteration: Arc::new(RwLock::new(None)), // Never running when restored from disk
            prompt_extension: Arc::new(RwLock::new(None)),
            is_muted: Arc::new(RwLock::new(data.is_muted)),
            context_mode: Arc::new(RwLock::new(data.context_mode)),
            sandbox_state: Arc::new(RwLock::new(data.sandbox_state)),
        }
    }

    /// Resolves a persona name to its UUID.
    ///
    /// This is used to convert speaker names to persona IDs.
    async fn get_persona_id_by_name(&self, name: &str) -> Option<String> {
        let personas = self.persona_repository.get_all().await.ok()?;
        personas
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.id.clone())
    }

    /// Rebuilds dialogue history from persona_histories and system_messages for restoration.
    ///
    /// This method converts the stored conversation messages into DialogueTurn format,
    /// sorted by timestamp, to restore the conversation context when recreating a Dialogue.
    ///
    /// # Returns
    ///
    /// A vector of DialogueTurn representing the full conversation history.
    async fn rebuild_dialogue_history(&self) -> Vec<DialogueTurn> {
        let histories = self.persona_histories.read().await;

        // Flatten all messages with (persona_id, timestamp, message)
        let mut all_messages: Vec<(String, String, ConversationMessage)> = Vec::new();

        // Add messages from persona_histories
        for (persona_id, messages) in histories.iter() {
            for msg in messages {
                all_messages.push((persona_id.clone(), msg.timestamp.clone(), msg.clone()));
            }
        }

        // Add system_messages that should be included in dialogue
        let system_msgs = self.system_messages.read().await;
        for msg in system_msgs.iter() {
            if msg.metadata.include_in_dialogue {
                all_messages.push((
                    "system".to_string(), // Use "system" as pseudo persona_id for system messages
                    msg.timestamp.clone(),
                    msg.clone(),
                ));
            }
        }

        // Sort by timestamp to maintain chronological order
        all_messages.sort_by(|a, b| a.1.cmp(&b.1));

        // Convert to DialogueTurn with explicit Speaker attribution
        all_messages
            .iter()
            .filter_map(|(persona_id, _, msg)| {
                match msg.role {
                    MessageRole::User => {
                        // User input with explicit User speaker
                        let user_name = self.user_service.get_user_name();
                        Some(DialogueTurn {
                            speaker: Speaker::user(user_name, "User"),
                            content: msg.content.clone(),
                        })
                    }
                    MessageRole::Assistant => {
                        // Assistant response - convert persona_id to Agent speaker
                        // Note: We cannot await inside filter_map, so we'll use a fallback
                        // This is acceptable because we're just displaying the history
                        Some(DialogueTurn {
                            speaker: Speaker::agent(persona_id, "Agent"),
                            content: msg.content.clone(),
                        })
                    }
                    MessageRole::System => {
                        // System/Error messages included in dialogue history
                        Some(DialogueTurn {
                            speaker: Speaker::System,
                            content: msg.content.clone(),
                        })
                    }
                }
            })
            .collect()
    }

    /// Ensures the dialogue is initialized. If not, creates it from a blueprint.
    ///
    /// # Errors
    ///
    /// Returns an error if dialogue creation fails.
    async fn ensure_dialogue_initialized(&self) -> Result<(), String> {
        let mut dialogue_guard = self.dialogue.lock().await;
        if dialogue_guard.is_some() {
            return Ok(());
        }

        let strategy_model = self.execution_strategy.read().await.clone();

        // Rebuild dialogue history from persona_histories
        let history_turns = self.rebuild_dialogue_history().await;

        // Read current talk style (only in Rich mode)
        let context_mode = self.context_mode.read().await.clone();
        let talk_style = if matches!(context_mode, ContextMode::Rich) {
            self.talk_style.read().await.clone()
        } else {
            None // Clean mode: no talk style
        };

        // Create dialogue with restored history and context
        let mut dialogue = match strategy_model {
            ExecutionModel::Sequential => Dialogue::sequential(),
            ExecutionModel::Broadcast => Dialogue::broadcast(),
            ExecutionModel::Mentioned { .. } => Dialogue::mentioned(),
            // New variants - map to closest existing strategy
            ExecutionModel::OrderedSequential(_) => Dialogue::sequential(),
            ExecutionModel::OrderedBroadcast(_) => Dialogue::broadcast(),
            ExecutionModel::Moderator => Dialogue::broadcast(),
        };

        // Apply context settings
        let mut additional_context = "【協調ガイドライン】\n\
                 - 複数の AI ペルソナが協力してユーザーをサポートします\n\
                 - 他の参加者の意見を尊重し、重複を避けて新しい視点を提供してください\n\
                 - ユーザーのワークスペース環境で実行されています\n\
                 - 建設的で協調的なコミュニケーションを心がけてください"
            .to_string();

        if let Some(extension) = self.prompt_extension.read().await.clone() {
            if !extension.trim().is_empty() {
                additional_context.push_str("\n\n");
                additional_context.push_str(&extension);
            }
        }

        dialogue
            .with_environment("ORCS (Orchestrated Reasoning & Collaboration System) マルチエージェント対話アプリケーション")
            .with_additional_context(additional_context)
            .with_reaction_strategy(ReactionStrategy::ExceptContextInfo);

        // Apply talk style if set
        if let Some(style) = talk_style {
            dialogue.with_talk_style(style);
        }

        tracing::info!(
            "[InteractionManager] Restored dialogue with {} history turns",
            history_turns.len()
        );

        let mut dialogue = dialogue.with_history_as_system_prompt(history_turns);

        // Check if we have restored participant IDs from session
        let restored_ids_opt = self.restored_participant_ids.read().await.clone();

        let personas_to_add: Vec<PersonaDomain> = if let Some(restored_ids) = restored_ids_opt {
            // Restore specific participants from session
            let all_personas = self
                .persona_repository
                .get_all()
                .await
                .map_err(|e| e.to_string())?;
            all_personas
                .into_iter()
                .filter(|p| restored_ids.contains(&p.id))
                .collect()
        } else {
            // Use default participants
            self.persona_repository
                .get_all()
                .await
                .map_err(|e| e.to_string())?
                .into_iter()
                .filter(|p| p.default_participant)
                .collect()
        };

        for persona in personas_to_add {
            let llm_persona = domain_to_llm_persona(&persona);
            let agent = agent_for_persona(
                &persona,
                self.agent_workspace_root.clone(),
                self.env_settings.clone(),
            );
            dialogue.add_agent(llm_persona, agent);
        }

        // Keep restored_participant_ids for future dialogue recreations
        // Do NOT clear them - they should persist to maintain participant list
        // across dialogue invalidations (e.g., when execution strategy changes)

        *dialogue_guard = Some(dialogue);
        Ok(())
    }

    /// Converts the current state to Session for persistence.
    ///
    /// # Arguments
    ///
    /// * `app_mode` - The current application mode
    /// * `workspace_id` - Workspace ID to associate with this session
    pub async fn to_session(&self, app_mode: AppMode, workspace_id: String) -> Session {
        let persona_histories = self.persona_histories.read().await.clone();
        let title = self.title.read().await.clone();
        let execution_strategy = self.execution_strategy.read().await.clone();
        let system_messages = self.system_messages.read().await.clone();

        // Use the first default participant as current_persona_id
        let current_persona_id = self
            .persona_repository
            .get_all()
            .await
            .ok()
            .and_then(|personas| {
                personas
                    .iter()
                    .find(|p| p.default_participant)
                    .map(|p| p.id.clone())
            })
            .unwrap_or_else(|| "unknown".to_string());

        // Use internal workspace_id if set, otherwise use provided workspace_id as fallback
        let internal_workspace_id = self.workspace_id.read().await.clone();
        let final_workspace_id = internal_workspace_id.unwrap_or(workspace_id);

        // Get active participants if dialogue is initialized
        let active_participant_ids = self.get_active_participants().await.unwrap_or_default();

        // Build participants map: persona ID -> name
        let mut participants = HashMap::new();
        // Build participant_icons map: persona ID -> icon
        let mut participant_icons = HashMap::new();
        // Build participant_colors map: persona ID -> base_color
        let mut participant_colors = HashMap::new();
        // Build participant_backends map: persona ID -> backend (e.g., "claude_api")
        let mut participant_backends = HashMap::new();
        // Build participant_models map: persona ID -> model name
        let mut participant_models = HashMap::new();

        // Always add user name first (user is always a participant)
        let user_name = self.user_service.get_user_name();
        participants.insert(user_name.clone(), user_name.clone());
        // User has no icon/color/backend/model for now

        // Add all personas from persona_histories (AI participants)
        if let Ok(all_personas) = self.persona_repository.get_all().await {
            for persona_id in persona_histories.keys() {
                // Skip user's history key if it exists
                if persona_id == &user_name {
                    continue;
                }

                if let Some(persona) = all_personas.iter().find(|p| &p.id == persona_id) {
                    participants.insert(persona_id.clone(), persona.name.clone());
                    // Add icon if persona has one
                    if let Some(icon) = &persona.icon {
                        participant_icons.insert(persona_id.clone(), icon.clone());
                    }
                    // Add base_color if persona has one
                    if let Some(color) = &persona.base_color {
                        participant_colors.insert(persona_id.clone(), color.clone());
                    }
                    // Add backend (serialize backend enum to string)
                    let backend_str = serde_json::to_string(&persona.backend)
                        .unwrap_or_else(|_| "\"claude_cli\"".to_string())
                        .trim_matches('"')
                        .to_string();
                    participant_backends.insert(persona_id.clone(), backend_str);
                    // Add model_name if persona has one
                    participant_models.insert(persona_id.clone(), persona.model_name.clone());
                }
            }
        }

        let conversation_mode = self.conversation_mode.read().await.clone();
        let talk_style = self.talk_style.read().await.clone();
        let auto_chat_config = self.auto_chat_config.read().await.clone();
        let is_muted = self.is_muted.read().await.clone();

        Session {
            id: self.session_id.clone(),
            title,
            created_at: self.created_at.clone(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            current_persona_id,
            persona_histories,
            app_mode,
            workspace_id: final_workspace_id,
            active_participant_ids,
            execution_strategy,
            system_messages,
            participants,
            participant_icons,
            participant_colors,
            participant_backends,
            participant_models,
            conversation_mode,
            talk_style,
            is_favorite: false,
            is_archived: false,
            sort_order: None,
            auto_chat_config,
            is_muted,
            context_mode: self.context_mode.read().await.clone(),
            sandbox_state: self.sandbox_state.read().await.clone(),
        }
    }

    /// Returns the session ID.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Updates the workspace ID for this session.
    pub async fn set_workspace_id(
        &self,
        workspace_id: Option<String>,
        workspace_root: Option<PathBuf>,
    ) {
        tracing::info!(
            "[InteractionManager::set_workspace_id] Called with workspace_id={:?}, workspace_root={:?}",
            workspace_id,
            workspace_root
        );

        let mut ws_id = self.workspace_id.write().await;
        *ws_id = workspace_id.clone();

        let mut ws_root = self.agent_workspace_root.write().await;
        *ws_root = workspace_root.clone();

        tracing::info!(
            "[InteractionManager::set_workspace_id] Updated agent_workspace_root to: {:?}",
            workspace_root
        );
    }

    /// Sets the agent workspace root (used for Sandbox mode to change CWD).
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - The workspace root path for agents to use as CWD
    ///
    /// # Note
    ///
    /// This will invalidate the current dialogue instance, which will be recreated
    /// with the new workspace root on the next interaction.
    pub async fn set_agent_workspace_root(&self, workspace_root: Option<PathBuf>) {
        tracing::info!(
            "[InteractionManager::set_agent_workspace_root] Setting to: {:?}",
            workspace_root
        );

        *self.agent_workspace_root.write().await = workspace_root;

        // Invalidate dialogue so agents are recreated with new workspace root
        self.invalidate_dialogue().await;
    }

    /// Gets the current agent workspace root.
    pub async fn get_agent_workspace_root(&self) -> Option<PathBuf> {
        self.agent_workspace_root.read().await.clone()
    }

    /// Returns a list of available persona IDs.
    pub async fn available_personas(&self) -> Vec<String> {
        self.persona_repository
            .get_all()
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|p| p.id)
            .collect()
    }

    /// Adds a participant to the dialogue.
    ///
    /// # Arguments
    ///
    /// * `persona_id` - The ID of the persona to add (e.g., "mai", "yui")
    ///
    /// # Errors
    ///
    /// Returns an error if the persona is not found or dialogue initialization fails.
    pub async fn add_participant(&self, persona_id: &str) -> Result<(), String> {
        // Ensure dialogue is initialized
        self.ensure_dialogue_initialized().await?;

        // Find the persona
        let persona_config = self
            .persona_repository
            .get_all()
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|p| p.id == persona_id)
            .ok_or_else(|| format!("Persona with id '{}' not found", persona_id))?;
        let persona = domain_to_llm_persona(&persona_config);

        // Record system message
        let system_msg = ConversationMessage {
            role: MessageRole::System,
            content: format!("{} が会話に参加しました", persona_config.name),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: MessageMetadata {
                system_event_type: Some(SystemEventType::ParticipantJoined),
                error_severity: None,
                system_message_type: None,
                include_in_dialogue: true,
                llm_debug_info: None,
            },
            attachments: vec![],
        };
        self.system_messages.write().await.push(system_msg);

        // Lock the dialogue and add participant
        let mut dialogue_guard = self.dialogue.lock().await;
        let dialogue = match dialogue_guard.as_mut() {
            Some(d) => d,
            None => {
                return Err(
                    "Dialogue was invalidated during initialization (possible race condition)"
                        .to_string(),
                )
            }
        };
        let agent = agent_for_persona(
            &persona_config,
            self.agent_workspace_root.clone(),
            self.env_settings.clone(),
        );
        dialogue.add_agent(persona, agent);

        // Update restored_participant_ids to persist across dialogue recreations
        // Get current active participants and add the new one
        let all_personas = self.persona_repository.get_all().await.ok();
        let current_ids = dialogue
            .participants()
            .iter()
            .filter_map(|p| {
                all_personas
                    .as_ref()
                    .and_then(|all| all.iter().find(|persona| persona.name == p.name))
                    .map(|persona| persona.id.clone())
            })
            .collect::<Vec<_>>();

        *self.restored_participant_ids.write().await = Some(current_ids);

        Ok(())
    }

    /// Removes a participant from the dialogue.
    ///
    /// # Arguments
    ///
    /// * `persona_id` - The ID of the persona to remove (e.g., "mai", "yui")
    ///
    /// # Errors
    ///
    /// Returns an error if the persona is not found, dialogue initialization fails,
    /// or the participant cannot be removed.
    pub async fn remove_participant(&self, persona_id: &str) -> Result<(), String> {
        // Ensure dialogue is initialized
        self.ensure_dialogue_initialized().await?;

        // Find the persona to get its full name
        let persona_config = self
            .persona_repository
            .get_all()
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|p| p.id == persona_id)
            .ok_or_else(|| format!("Persona with id '{}' not found", persona_id))?;
        let persona = domain_to_llm_persona(&persona_config);

        // Record system message
        let system_msg = ConversationMessage {
            role: MessageRole::System,
            content: format!("{} が会話から退出しました", persona_config.name),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: MessageMetadata {
                system_event_type: Some(SystemEventType::ParticipantLeft),
                error_severity: None,
                system_message_type: None,
                include_in_dialogue: true,
                llm_debug_info: None,
            },
            attachments: vec![],
        };
        self.system_messages.write().await.push(system_msg);

        // Lock the dialogue and remove participant
        let mut dialogue_guard = self.dialogue.lock().await;
        let dialogue = match dialogue_guard.as_mut() {
            Some(d) => d,
            None => {
                return Err(
                    "Dialogue was invalidated during initialization (possible race condition)"
                        .to_string(),
                )
            }
        };
        dialogue
            .remove_participant(&persona.name)
            .map_err(|e| e.to_string())?;

        // Update restored_participant_ids to persist across dialogue recreations
        let all_personas = self.persona_repository.get_all().await.ok();
        let current_ids = dialogue
            .participants()
            .iter()
            .filter_map(|p| {
                all_personas
                    .as_ref()
                    .and_then(|all| all.iter().find(|persona| persona.name == p.name))
                    .map(|persona| persona.id.clone())
            })
            .collect::<Vec<_>>();

        // Always set Some(...) to distinguish between:
        // - None: initial state (use default_participant)
        // - Some(vec![]): user explicitly removed all participants (add nobody)
        *self.restored_participant_ids.write().await = Some(current_ids);

        Ok(())
    }

    /// Records a system-level conversation message so it persists with the session.
    pub async fn add_system_conversation_message(
        &self,
        content: String,
        message_type: Option<String>,
        error_severity: Option<ErrorSeverity>,
    ) {
        let is_context_info = matches!(
            message_type.as_deref(),
            Some("context_info" | "shell_output")
        );
        let message = ConversationMessage {
            role: MessageRole::System,
            content,
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: MessageMetadata {
                system_event_type: Some(SystemEventType::Notification),
                error_severity,
                system_message_type: message_type,
                include_in_dialogue: true,
                llm_debug_info: None,
            },
            attachments: vec![],
        };

        self.system_messages.write().await.push(message);

        if is_context_info {
            // Context info (shell output, etc.) must be visible before the next agent turn.
            // We intentionally invalidate the dialogue on every context info write so that
            // shell results injected via append_system_messages are folded into the prompt
            // on the very next ensure_dialogue_initialized() call.  This code path has caused
            // regressions multiple times; resist the urge to “optimize” it away.
            self.invalidate_dialogue().await;
        }
    }

    /// Returns a list of active participant IDs.
    ///
    /// # Errors
    ///
    /// Returns an error if dialogue initialization fails.
    pub async fn get_active_participants(&self) -> Result<Vec<String>, String> {
        self.ensure_dialogue_initialized().await?;

        let dialogue_guard = self.dialogue.lock().await;
        let dialogue = match dialogue_guard.as_ref() {
            Some(d) => d,
            None => {
                return Err(
                    "Dialogue was invalidated during initialization (possible race condition)"
                        .to_string(),
                )
            }
        };

        // Convert participant names to persona UUIDs
        let mut participant_ids = Vec::new();
        for persona in dialogue.participants().iter() {
            if let Some(id) = self.get_persona_id_by_name(&persona.name).await {
                participant_ids.push(id);
            }
        }

        Ok(participant_ids)
    }

    /// Sets the execution strategy for the dialogue.
    ///
    /// # Arguments
    ///
    /// * `strategy` - The execution strategy to use
    ///
    /// # Note
    ///
    /// This will invalidate the current dialogue instance, which will be recreated
    /// with the new strategy on the next interaction.
    pub async fn set_execution_strategy(&self, strategy: ExecutionModel) {
        // Record system message for context visibility to agents
        let strategy_name = match strategy {
            ExecutionModel::Broadcast => "Broadcast",
            ExecutionModel::Sequential => "Sequential",
            ExecutionModel::Mentioned { .. } => "Mentioned",
            // New variants
            ExecutionModel::OrderedSequential(_) => "Ordered Sequential",
            ExecutionModel::OrderedBroadcast(_) => "Ordered Broadcast",
            ExecutionModel::Moderator => "Moderator",
        };
        let system_msg = ConversationMessage {
            role: MessageRole::System,
            content: format!("実行戦略を {} に変更しました", strategy_name),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: MessageMetadata {
                system_event_type: Some(SystemEventType::ExecutionStrategyChanged),
                error_severity: None,
                system_message_type: None,
                include_in_dialogue: true,
                llm_debug_info: None,
            },
            attachments: vec![],
        };
        self.system_messages.write().await.push(system_msg);

        *self.execution_strategy.write().await = strategy;
        // Clear the dialogue to force recreation with new strategy
        *self.dialogue.lock().await = None;
    }

    /// Gets the current execution strategy.
    pub async fn get_execution_strategy(&self) -> ExecutionModel {
        self.execution_strategy.read().await.clone()
    }

    /// Sets the conversation mode for controlling dialogue verbosity.
    ///
    /// # Arguments
    ///
    /// * `mode` - The conversation mode to use (Normal/Concise/Brief/Discussion)
    ///
    /// # Note
    ///
    /// This affects how AI agents respond to prevent response escalation.
    /// The mode's system instruction will be injected on the next interaction.
    pub async fn set_conversation_mode(&self, mode: ConversationMode) {
        // Record system message for mode change
        let mode_str = match mode {
            ConversationMode::Detailed => "詳細",
            ConversationMode::Normal => "通常",
            ConversationMode::Concise => "簡潔 (300文字)",
            ConversationMode::Brief => "極簡潔 (150文字)",
            ConversationMode::Discussion => "議論",
        };
        let system_msg = ConversationMessage {
            role: MessageRole::System,
            content: format!("会話モードを {} に変更しました", mode_str),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: MessageMetadata {
                system_event_type: Some(SystemEventType::ModeChanged),
                error_severity: None,
                system_message_type: None,
                include_in_dialogue: true,
                llm_debug_info: None,
            },
            attachments: vec![],
        };
        self.system_messages.write().await.push(system_msg);

        *self.conversation_mode.write().await = mode;
    }

    /// Gets the current conversation mode.
    pub async fn get_conversation_mode(&self) -> ConversationMode {
        self.conversation_mode.read().await.clone()
    }

    /// Sets the talk style for dialogue context.
    ///
    /// # Arguments
    ///
    /// * `style` - The talk style to use (Brainstorm/Debate/ProblemSolving/etc.)
    ///
    /// # Note
    ///
    /// This affects the dialogue context and conversation tone.
    /// The style will be applied on the next dialogue creation.
    pub async fn set_talk_style(&self, style: Option<TalkStyle>) {
        // Record system message for talk style change
        if let Some(s) = &style {
            let style_str = match s {
                TalkStyle::Brainstorm => "ブレインストーミング",
                TalkStyle::Casual => "カジュアル",
                TalkStyle::DecisionMaking => "意思決定",
                TalkStyle::Debate => "議論",
                TalkStyle::ProblemSolving => "問題解決",
                TalkStyle::Review => "レビュー",
                TalkStyle::Planning => "計画",
            };
            let system_msg = ConversationMessage {
                role: MessageRole::System,
                content: format!("会話スタイルを {} に変更しました", style_str),
                timestamp: chrono::Utc::now().to_rfc3339(),
                metadata: MessageMetadata {
                    system_event_type: Some(SystemEventType::ModeChanged),
                    error_severity: None,
                    system_message_type: None,
                    include_in_dialogue: true,
                    llm_debug_info: None,
                },
                attachments: vec![],
            };
            self.system_messages.write().await.push(system_msg);
        }

        *self.talk_style.write().await = style;

        // Invalidate dialogue to apply new style
        self.invalidate_dialogue().await;
    }

    /// Gets the current talk style.
    pub async fn get_talk_style(&self) -> Option<TalkStyle> {
        self.talk_style.read().await.clone()
    }

    /// Sets an additional prompt extension that will be appended to the system prompt.
    pub async fn set_prompt_extension(&self, extension: Option<String>) {
        *self.prompt_extension.write().await = extension;
        self.invalidate_dialogue().await;
    }

    /// Sets the AutoChat configuration.
    pub async fn set_auto_chat_config(&self, config: Option<AutoChatConfig>) {
        *self.auto_chat_config.write().await = config;
    }

    /// Gets the current AutoChat configuration.
    pub async fn get_auto_chat_config(&self) -> Option<AutoChatConfig> {
        self.auto_chat_config.read().await.clone()
    }

    /// Gets the current AutoChat iteration (None if not running).
    pub async fn get_auto_chat_iteration(&self) -> Option<i32> {
        *self.auto_chat_iteration.read().await
    }

    /// Sets the current AutoChat iteration.
    pub async fn set_auto_chat_iteration(&self, iteration: Option<i32>) {
        *self.auto_chat_iteration.write().await = iteration;
    }

    /// Invalidates the current dialogue, forcing it to be recreated with latest persona settings.
    ///
    /// This should be called when:
    /// - Persona configurations are updated
    /// - Persona settings (role, background, etc.) are changed
    ///
    /// The dialogue will be recreated with the latest settings on the next interaction.
    pub async fn invalidate_dialogue(&self) {
        *self.dialogue.lock().await = None;
    }

    /// Toggles mute status and returns the new value.
    pub async fn toggle_mute(&self) -> bool {
        let mut is_muted = self.is_muted.write().await;
        *is_muted = !*is_muted;
        *is_muted
    }

    /// Gets the current mute status.
    pub async fn is_muted(&self) -> bool {
        *self.is_muted.read().await
    }

    /// Sets the mute status.
    pub async fn set_mute(&self, muted: bool) {
        *self.is_muted.write().await = muted;
    }

    /// Gets the current context mode.
    pub async fn get_context_mode(&self) -> ContextMode {
        *self.context_mode.read().await
    }

    /// Sets the context mode.
    pub async fn set_context_mode(&self, mode: ContextMode) {
        *self.context_mode.write().await = mode;
    }

    /// Sets the sandbox state for git worktree-based isolated development.
    pub async fn set_sandbox_state(&self, state: Option<orcs_core::session::SandboxState>) {
        *self.sandbox_state.write().await = state;
    }

    /// Gets the current sandbox state.
    pub async fn get_sandbox_state(&self) -> Option<orcs_core::session::SandboxState> {
        self.sandbox_state.read().await.clone()
    }

    /// Handles user input based on the current application mode.
    ///
    /// # Arguments
    ///
    /// * `mode` - The current application mode
    /// * `input` - The user's input string
    pub async fn handle_input(&self, mode: &AppMode, input: &str) -> InteractionResult {
        match mode {
            AppMode::Idle => {
                self.handle_idle_mode(input, None, None::<fn(&DialogueMessage)>, true)
                    .await
            }
            AppMode::AwaitingConfirmation { plan } => {
                self.handle_awaiting_confirmation(input, plan)
            }
        }
    }

    /// Handles user input with streaming support via callback.
    ///
    /// # Arguments
    ///
    /// * `mode` - The current application mode
    /// * `input` - The user's input string
    /// * `file_paths` - Optional list of file paths to attach
    /// * `on_turn` - Callback function called for each dialogue turn as it becomes available
    ///
    /// # Returns
    ///
    /// Returns an `InteractionResult` indicating the outcome of handling the input.
    pub async fn handle_input_with_streaming<F>(
        &self,
        mode: &AppMode,
        input: &str,
        file_paths: Option<Vec<String>>,
        on_turn: F,
    ) -> InteractionResult
    where
        F: Fn(&DialogueMessage),
    {
        match mode {
            AppMode::Idle => {
                self.handle_idle_mode(input, file_paths, Some(on_turn), true)
                    .await
            }
            AppMode::AwaitingConfirmation { plan } => {
                self.handle_awaiting_confirmation(input, plan)
            }
        }
    }

    /// Handles a system message that triggers dialogue continuation.
    ///
    /// # Arguments
    ///
    /// * `message` - The system message content
    /// * `on_turn` - Optional callback for streaming turns
    async fn handle_system_message<F>(&self, message: &str, on_turn: Option<F>) -> InteractionResult
    where
        F: Fn(&DialogueMessage),
    {
        // Ensure dialogue is initialized
        if let Err(e) = self.ensure_dialogue_initialized().await {
            return InteractionResult::NewMessage(format!("Error initializing dialogue: {}", e));
        }

        // Add system message to history for persistence
        self.add_system_conversation_message(message.to_string(), Some("system".to_string()), None)
            .await;

        // Send system message to UI via callback
        if let Some(ref callback) = on_turn {
            let system_msg = DialogueMessage {
                session_id: self.session_id.clone(),
                author: "System".to_string(),
                content: message.to_string(),
            };
            callback(&system_msg);
        }

        // Run the dialogue with system speaker
        let mut dialogue_guard = self.dialogue.lock().await;
        let dialogue = match dialogue_guard.as_mut() {
            Some(d) => d,
            None => {
                drop(dialogue_guard);
                return InteractionResult::NewMessage(
                    "Error: Dialogue was invalidated during initialization (possible race condition)"
                        .to_string(),
                );
            }
        };

        let speaker = Speaker::System;
        let mut payload = Payload::new().with_message(speaker, message);

        // Prepend conversation mode system instruction if available (Rich mode only)
        let context_mode = self.context_mode.read().await.clone();
        if matches!(context_mode, ContextMode::Rich) {
            let conversation_mode = self.conversation_mode.read().await;
            if let Some(instruction) = conversation_mode.system_instruction() {
                payload = payload.prepend_system(instruction);
            }
            drop(conversation_mode);
        }

        // Create a partial session for incremental turn processing
        let mut session = dialogue.partial_session(payload);
        let mut messages = Vec::new();

        // Process each turn as it becomes available
        while let Some(result) = session.next_turn().await {
            match result {
                Ok(turn) => {
                    let speaker_name = turn.speaker.name();
                    let preview: String = turn.content.chars().take(50).collect();
                    tracing::debug!(
                        "[DIALOGUE] Turn received: {} - {}...",
                        speaker_name,
                        preview
                    );

                    // Convert speaker name to persona_id (UUID)
                    let persona_id = self
                        .get_persona_id_by_name(speaker_name)
                        .await
                        .unwrap_or_else(|| speaker_name.to_string());

                    // Add each response to history using persona_id
                    self.add_to_history(&persona_id, MessageRole::Assistant, &turn.content, None)
                        .await;

                    // Create DialogueMessage for UI display
                    let message = DialogueMessage {
                        session_id: self.session_id.clone(),
                        author: speaker_name.to_string(),
                        content: turn.content.clone(),
                    };

                    // Call the streaming callback if provided
                    if let Some(ref callback) = on_turn {
                        callback(&message);
                    }

                    messages.push(message);
                }
                Err(e) => {
                    tracing::error!("[DIALOGUE] Agent execution failed: {}", e);

                    let error_msg = format!("{}\n\nPlease check the logs for more details.", e);

                    // Emit error as a system message via callback if provided
                    if let Some(ref callback) = on_turn {
                        let error_turn = DialogueMessage {
                            session_id: self.session_id.clone(),
                            author: String::new(),
                            content: error_msg.clone(),
                        };
                        callback(&error_turn);
                    }

                    // Add error to history for persistence with metadata
                    let error_history = ConversationMessage {
                        role: MessageRole::System,
                        content: error_msg.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        metadata: MessageMetadata {
                            system_event_type: None,
                            error_severity: Some(ErrorSeverity::Critical),
                            system_message_type: None,
                            include_in_dialogue: true,
                            llm_debug_info: None,
                        },
                        attachments: vec![],
                    };
                    self.persona_histories
                        .write()
                        .await
                        .entry("Error".to_string())
                        .or_insert_with(Vec::new)
                        .push(error_history);

                    return InteractionResult::NewDialogueMessages(Vec::new());
                }
            }
        }

        InteractionResult::NewDialogueMessages(messages)
    }

    /// Handles input when in Idle mode.
    ///
    /// # Arguments
    ///
    /// * `input` - The input text to process
    /// * `file_paths` - Optional file attachments
    /// * `on_turn` - Optional callback for streaming turns
    /// * `add_to_history` - Whether to add the input to user history (default: true)
    async fn handle_idle_mode<F>(
        &self,
        input: &str,
        file_paths: Option<Vec<String>>,
        on_turn: Option<F>,
        add_to_history: bool,
    ) -> InteractionResult
    where
        F: Fn(&DialogueMessage),
    {
        let trimmed: &str = input.trim();
        if trimmed.is_empty() {
            return InteractionResult::NoOp;
        }

        // Check if session is muted - if so, only add to history but don't run AI
        let is_muted = self.is_muted().await;

        // Add user input to history BEFORE checking mute (so user's message is saved)
        let user_name = self.user_service.get_user_name();
        if add_to_history {
            self.add_to_history(&user_name, MessageRole::User, input, file_paths.clone())
                .await;
        }

        // If muted, return early without running dialogue
        if is_muted {
            tracing::info!("[InteractionManager] Session is muted, skipping AI response");
            return InteractionResult::NoOp;
        }

        // Ensure dialogue is initialized
        if let Err(e) = self.ensure_dialogue_initialized().await {
            return InteractionResult::NewMessage(format!("Error initializing dialogue: {}", e));
        }
        let user_name_str = if user_name.to_lowercase() == "you" {
            tracing::warn!(
                "[InteractionManager] Detected user name 'You', which may cause speaker attribution issues."
            );
            "User"
        } else {
            &user_name
        };
        let speaker = Speaker::user(user_name_str, "User");

        // Run the dialogue with the user's input using partial_session for streaming
        let mut dialogue_guard = self.dialogue.lock().await;
        let dialogue = match dialogue_guard.as_mut() {
            Some(d) => d,
            None => {
                drop(dialogue_guard);
                return InteractionResult::NewMessage(
                    "Error: Dialogue was invalidated during initialization (possible race condition)"
                        .to_string(),
                );
            }
        };

        // Note: Dialogue/Persona agents handle speaker attribution internally
        let mut payload = Payload::new().with_message(speaker, input);

        // Prepend conversation mode system instruction if available (Rich mode only)
        let context_mode = self.context_mode.read().await.clone();
        if matches!(context_mode, ContextMode::Rich) {
            let conversation_mode = self.conversation_mode.read().await;
            if let Some(instruction) = conversation_mode.system_instruction() {
                payload = payload.prepend_system(instruction);
            }
            drop(conversation_mode);
        }

        // Add file attachments if provided
        if let Some(paths) = file_paths {
            for path in paths {
                tracing::info!("[InteractionManager] Attaching file: {}", path);
                payload = payload.with_attachment(Attachment::local(path));
            }
        }

        // Debug: Log payload content before partial_session
        tracing::debug!(
            "[InteractionManager] Payload before partial_session: user_input='{}', payload={:?}",
            input.chars().take(100).collect::<String>(),
            payload.clone()
        );

        // Create a partial session for incremental turn processing
        // partial_session now accepts impl Into<Payload>, so both String and Payload work
        let mut session = dialogue.partial_session(payload);
        let mut messages = Vec::new();

        // Process each turn as it becomes available
        while let Some(result) = session.next_turn().await {
            match result {
                Ok(turn) => {
                    // Log the turn for debugging sequential execution with timestamp
                    let speaker_name = turn.speaker.name();
                    let preview: String = turn.content.chars().take(50).collect();
                    tracing::debug!(
                        "[DIALOGUE] Turn received: {} - {}...",
                        speaker_name,
                        preview
                    );

                    // Convert speaker name to persona_id (UUID)
                    let persona_id = self
                        .get_persona_id_by_name(speaker_name)
                        .await
                        .unwrap_or_else(|| speaker_name.to_string());

                    // Add each response to history using persona_id
                    self.add_to_history(&persona_id, MessageRole::Assistant, &turn.content, None)
                        .await;

                    // Create DialogueMessage for UI display
                    let message = DialogueMessage {
                        session_id: self.session_id.clone(),
                        author: speaker_name.to_string(),
                        content: turn.content.clone(),
                    };

                    // Call the streaming callback if provided
                    if let Some(ref callback) = on_turn {
                        callback(&message);
                    }

                    messages.push(message);
                }
                Err(e) => {
                    // Log the error for debugging
                    tracing::error!("[DIALOGUE] Agent execution failed: {}", e);

                    // Create a user-friendly error message
                    let error_msg = format!("{}\n\nPlease check the logs for more details.", e);

                    // Emit error as a system message via callback if provided
                    if let Some(ref callback) = on_turn {
                        let error_turn = DialogueMessage {
                            session_id: self.session_id.clone(),
                            author: String::new(), // Empty author for error messages
                            content: error_msg.clone(),
                        };
                        callback(&error_turn);
                    }

                    // Add error to history for persistence with metadata
                    let error_history = ConversationMessage {
                        role: MessageRole::System,
                        content: error_msg.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        metadata: MessageMetadata {
                            system_event_type: None,
                            error_severity: Some(ErrorSeverity::Critical),
                            system_message_type: None,
                            include_in_dialogue: true,
                            llm_debug_info: None,
                        },
                        attachments: vec![],
                    };
                    self.persona_histories
                        .write()
                        .await
                        .entry("Error".to_string())
                        .or_insert_with(Vec::new)
                        .push(error_history);

                    // Return empty dialogue messages (error already streamed via callback)
                    return InteractionResult::NewDialogueMessages(Vec::new());
                }
            }
        }

        InteractionResult::NewDialogueMessages(messages)
    }

    /// Executes AutoChat mode: runs multiple dialogue iterations automatically.
    ///
    /// # Arguments
    ///
    /// * `initial_input` - The user's initial input to start the auto-chat
    /// * `file_paths` - Optional list of file paths to attach (only for initial input)
    /// * `on_turn` - Callback function called for each dialogue turn as it becomes available
    /// * `cancel_flag` - Optional atomic flag to check for cancellation
    ///
    /// # Returns
    ///
    /// Returns an `InteractionResult` indicating the outcome.
    ///
    /// # AutoChat Behavior
    ///
    /// - Iteration 1: Uses `initial_input` from the user
    /// - Iteration 2+: Uses empty string (agents continue discussion based on context)
    /// - Stops when: max_iterations reached OR user calls stop (via set_auto_chat_iteration(None)) OR cancel_flag is set
    pub async fn execute_auto_chat<F>(
        &self,
        initial_input: &str,
        file_paths: Option<Vec<String>>,
        on_turn: F,
        cancel_flag: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    ) -> InteractionResult
    where
        F: Fn(&DialogueMessage),
    {
        // Get AutoChat configuration
        let config = match self.get_auto_chat_config().await {
            Some(cfg) => cfg,
            None => {
                tracing::warn!("[AutoChat] No AutoChat config found, aborting");
                return InteractionResult::NewMessage(
                    "AutoChat is not configured for this session".to_string(),
                );
            }
        };

        tracing::info!(
            "[AutoChat] Starting with max_iterations={}, stop_condition={:?}",
            config.max_iterations,
            config.stop_condition
        );

        // Set initial iteration
        self.set_auto_chat_iteration(Some(0)).await;

        let mut current_iteration = 0;
        let mut last_result = InteractionResult::NoOp;

        while current_iteration < config.max_iterations {
            // Check cancellation flag
            if let Some(ref flag) = cancel_flag {
                if flag.load(std::sync::atomic::Ordering::SeqCst) {
                    tracing::info!("[AutoChat] Cancelled by user");
                    break;
                }
            }

            // Check if user manually stopped (set_auto_chat_iteration(None))
            if self.get_auto_chat_iteration().await.is_none() {
                tracing::info!("[AutoChat] Manually stopped by user");
                break;
            }

            // Update iteration counter
            current_iteration += 1;
            self.set_auto_chat_iteration(Some(current_iteration)).await;

            tracing::info!(
                "[AutoChat] Iteration {}/{}",
                current_iteration,
                config.max_iterations
            );

            // Execute dialogue iteration
            if current_iteration == 1 {
                // First iteration: use user's actual input
                last_result = self
                    .handle_idle_mode(
                        initial_input,
                        file_paths.clone(),
                        Some(&on_turn),
                        true, // Add to history
                    )
                    .await;
            } else {
                // Iteration 2+: Send system message to continue the discussion
                let continuation_content = "🔄 AutoMode: Discussion を続けましょう".to_string();
                last_result = self
                    .handle_system_message(&continuation_content, Some(&on_turn))
                    .await;
            }

            // Optional: Add delay between iterations to avoid overwhelming the UI
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            // For user_interrupt mode, check if iteration counter was cleared
            if matches!(
                config.stop_condition,
                orcs_core::session::StopCondition::UserInterrupt
            ) {
                if self.get_auto_chat_iteration().await.is_none() {
                    tracing::info!("[AutoChat] User interrupt detected");
                    break;
                }
            }
        }

        // Clear iteration counter when done
        self.set_auto_chat_iteration(None).await;

        tracing::info!(
            "[AutoChat] Completed after {} iterations",
            current_iteration
        );

        // Persist AutoChat completion message to session history
        let completion_content = format!(
            "✅ AutoChat completed after {} iterations.",
            current_iteration
        );
        self.add_system_conversation_message(
            completion_content,
            Some("auto_chat_completion".to_string()),
            None,
        )
        .await;

        last_result
    }

    /// Handles input when awaiting plan confirmation.
    fn handle_awaiting_confirmation(&self, input: &str, plan: &Plan) -> InteractionResult {
        let trimmed = input.trim().to_lowercase();

        match trimmed.as_str() {
            "yes" | "y" => InteractionResult::TasksToDispatch {
                tasks: plan.steps.clone(),
            },
            "no" | "n" => InteractionResult::ModeChanged(AppMode::Idle),
            _ => InteractionResult::NoOp,
        }
    }

    /// Adds a message to the conversation history.
    async fn add_to_history(
        &self,
        persona_id: &str,
        role: MessageRole,
        content: &str,
        attachments: Option<Vec<String>>,
    ) {
        let mut histories = self.persona_histories.write().await;
        let history = histories
            .entry(persona_id.to_string())
            .or_insert_with(Vec::new);

        history.push(ConversationMessage {
            role,
            content: content.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: MessageMetadata::default(), // User/Assistant messages with default metadata
            attachments: attachments.unwrap_or_default(),
        });
    }
}

impl Default for InteractionManager {
    fn default() -> Self {
        panic!("InteractionManager cannot be created with default. Use new_session.")
    }
}

// Implement trait required by SessionManager
impl orcs_core::session::InteractionManagerTrait for InteractionManager {
    fn session_id(&self) -> &str {
        &self.session_id
    }

    async fn to_session(&self, app_mode: AppMode, workspace_id: String) -> Session {
        self.to_session(app_mode, workspace_id).await
    }

    async fn set_workspace_id(
        &self,
        workspace_id: Option<String>,
        workspace_root: Option<PathBuf>,
    ) {
        self.set_workspace_id(workspace_id, workspace_root).await
    }
}
