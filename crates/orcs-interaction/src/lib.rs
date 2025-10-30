pub mod gemini_api_agent;
pub mod local_agents;
pub mod persona_agent;

use crate::gemini_api_agent::GeminiApiAgent;
use crate::local_agents::claude_code::ClaudeCodeAgent;
use crate::local_agents::gemini::GeminiAgent;
use llm_toolkit::agent::dialogue::{Dialogue, ExecutionModel};
use llm_toolkit::agent::persona::Persona as LlmPersona;
use llm_toolkit::agent::{Agent, AgentError, Payload};
use llm_toolkit::attachment::Attachment;
use orcs_core::persona::{Persona as PersonaDomain, PersonaBackend};
use orcs_core::repository::PersonaRepository;
use orcs_core::session::{AppMode, ConversationMessage, MessageRole, Plan, Session};
use orcs_core::user::UserService;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Converts a Persona domain model to llm-toolkit Persona.
fn domain_to_llm_persona(persona: &PersonaDomain) -> LlmPersona {
    LlmPersona {
        name: persona.name.clone(),
        role: persona.role.clone(),
        background: persona.background.clone(),
        communication_style: persona.communication_style.clone(),
    }
}

/// Converts a string strategy name to ExecutionModel enum.
fn string_to_execution_model(s: &str) -> ExecutionModel {
    match s {
        "sequential" => ExecutionModel::Sequential,
        _ => ExecutionModel::Broadcast,
    }
}

/// Agent wrapper that delegates to the configured backend.
#[derive(Clone, Debug)]
struct PersonaBackendAgent {
    backend: PersonaBackend,
    model_name: Option<String>,
    workspace_root: Arc<RwLock<Option<PathBuf>>>,
}

impl PersonaBackendAgent {
    fn new(
        backend: PersonaBackend,
        model_name: Option<String>,
        workspace_root: Arc<RwLock<Option<PathBuf>>>,
    ) -> Self {
        Self {
            backend,
            model_name,
            workspace_root,
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
            workspace_root, self.backend
        );

        match self.backend {
            PersonaBackend::ClaudeCli => {
                let mut agent = ClaudeCodeAgent::new(workspace_root);
                // Apply model if specified
                if let Some(ref model_str) = self.model_name {
                    tracing::info!("[PersonaBackendAgent] Using Claude model: {}", model_str);
                    agent = agent.with_model_str(model_str);
                }
                agent.execute(payload).await
            }
            PersonaBackend::GeminiCli => {
                // TODO: Add model support for Gemini CLI when available
                GeminiAgent::new(workspace_root).execute(payload).await
            }
            PersonaBackend::GeminiApi => {
                let mut agent = GeminiApiAgent::try_from_env()?;
                // Override model if specified
                if let Some(ref model_str) = self.model_name {
                    tracing::info!("[PersonaBackendAgent] Using Gemini model: {}", model_str);
                    agent = agent.with_model(model_str);
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
            PersonaBackend::GeminiCli => "Gemini CLI persona agent",
            PersonaBackend::GeminiApi => "Gemini API persona agent",
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
) -> PersonaBackendAgent {
    PersonaBackendAgent::new(
        persona.backend.clone(),
        persona.model_name.clone(),
        workspace_root,
    )
}

/// Represents a single message in a dialogue conversation.
///
/// Each message has an author (participant name) and the content of the message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DialogueMessage {
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
    /// Execution strategy for dialogue (e.g., "broadcast", "sequential")
    execution_strategy: Arc<RwLock<String>>,
}

impl InteractionManager {
    /// Creates a new session with empty conversation history.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for this session
    /// * `persona_repository` - Repository for accessing persona configurations
    /// * `user_service` - Service for retrieving user information
    pub fn new_session(
        session_id: String,
        persona_repository: Arc<dyn PersonaRepository>,
        user_service: Arc<dyn UserService>,
    ) -> Self {
        let mut persona_histories_map = HashMap::new();

        // Initialize with user's history
        persona_histories_map.insert(user_service.get_user_name(), Vec::new());

        // Initialize with default personas from repository
        if let Ok(personas) = persona_repository.get_all() {
            for persona in personas {
                if persona.default_participant {
                    persona_histories_map.insert(persona.id, Vec::new());
                }
            }
        }

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
            execution_strategy: Arc::new(RwLock::new("broadcast".to_string())),
        }
    }

    /// Restores a session from persisted data.
    ///
    /// # Arguments
    ///
    /// * `data` - The session data to restore
    /// * `persona_repository` - Repository for accessing persona configurations
    /// * `user_service` - Service for retrieving user information
    ///
    /// # Note
    ///
    /// This creates new Agent instances. History is stored separately
    /// in persona_histories and included in prompts manually.
    pub fn from_session(
        data: Session,
        persona_repository: Arc<dyn PersonaRepository>,
        user_service: Arc<dyn UserService>,
    ) -> Self {
        Self {
            session_id: data.id,
            title: Arc::new(RwLock::new(data.title)),
            created_at: data.created_at,
            workspace_id: Arc::new(RwLock::new(data.workspace_id)),
            agent_workspace_root: Arc::new(RwLock::new(None)), // Will be resolved and set by the caller
            dialogue: Arc::new(Mutex::new(None)),
            persona_histories: Arc::new(RwLock::new(data.persona_histories)),
            persona_repository,
            user_service,
            execution_strategy: Arc::new(RwLock::new("broadcast".to_string())),
        }
    }

    /// Resolves a persona name to its UUID.
    ///
    /// This is used to convert DialogueTurn participant names to persona IDs.
    fn get_persona_id_by_name(&self, name: &str) -> Option<String> {
        let personas = self.persona_repository.get_all().ok()?;
        personas
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.id.clone())
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

        let strategy_str = self.execution_strategy.read().await.clone();
        let strategy_model = string_to_execution_model(&strategy_str);
        let mut dialogue = match strategy_model {
            ExecutionModel::Sequential => Dialogue::sequential(),
            ExecutionModel::Broadcast => Dialogue::broadcast(),
        };

        let default_personas: Vec<PersonaDomain> = self
            .persona_repository
            .get_all()?
            .into_iter()
            .filter(|p| p.default_participant)
            .collect();

        for persona in default_personas {
            let llm_persona = domain_to_llm_persona(&persona);
            dialogue.add_participant(
                llm_persona,
                agent_for_persona(&persona, self.agent_workspace_root.clone()),
            );
        }

        *dialogue_guard = Some(dialogue);
        Ok(())
    }

    /// Converts the current state to Session for persistence.
    ///
    /// # Arguments
    ///
    /// * `app_mode` - The current application mode
    /// * `workspace_id` - Optional workspace ID to associate with this session (overrides instance workspace_id if provided)
    pub async fn to_session(&self, app_mode: AppMode, workspace_id: Option<String>) -> Session {
        let persona_histories = self.persona_histories.read().await.clone();
        let title = self.title.read().await.clone();
        let instance_workspace_id = self.workspace_id.read().await.clone();

        // Use the first default participant as current_persona_id
        let current_persona_id = self
            .persona_repository
            .get_all()
            .ok()
            .and_then(|personas| {
                personas
                    .iter()
                    .find(|p| p.default_participant)
                    .map(|p| p.id.clone())
            })
            .unwrap_or_else(|| "unknown".to_string());

        // Use provided workspace_id, fallback to instance workspace_id
        let final_workspace_id = workspace_id.or(instance_workspace_id);

        Session {
            id: self.session_id.clone(),
            title,
            created_at: self.created_at.clone(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            current_persona_id,
            persona_histories,
            app_mode,
            workspace_id: final_workspace_id,
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
            workspace_id, workspace_root
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

    /// Returns a list of available persona IDs.
    pub fn available_personas(&self) -> Vec<String> {
        self.persona_repository
            .get_all()
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
            .get_all()?
            .into_iter()
            .find(|p| p.id == persona_id)
            .ok_or_else(|| format!("Persona with id '{}' not found", persona_id))?;
        let persona = domain_to_llm_persona(&persona_config);

        // Lock the dialogue and add participant
        let mut dialogue_guard = self.dialogue.lock().await;
        let dialogue = dialogue_guard.as_mut().expect("Dialogue not initialized");
        dialogue.add_participant(
            persona,
            agent_for_persona(&persona_config, self.agent_workspace_root.clone()),
        );

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
            .get_all()?
            .into_iter()
            .find(|p| p.id == persona_id)
            .ok_or_else(|| format!("Persona with id '{}' not found", persona_id))?;
        let persona = domain_to_llm_persona(&persona_config);

        // Lock the dialogue and remove participant
        let mut dialogue_guard = self.dialogue.lock().await;
        let dialogue = dialogue_guard.as_mut().expect("Dialogue not initialized");
        dialogue
            .remove_participant(&persona.name)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Returns a list of active participant IDs.
    ///
    /// # Errors
    ///
    /// Returns an error if dialogue initialization fails.
    pub async fn get_active_participants(&self) -> Result<Vec<String>, String> {
        self.ensure_dialogue_initialized().await?;

        let dialogue_guard = self.dialogue.lock().await;
        let dialogue = dialogue_guard.as_ref().expect("Dialogue not initialized");

        // Convert participant names to persona UUIDs
        let participant_ids = dialogue
            .participants()
            .iter()
            .filter_map(|persona| self.get_persona_id_by_name(&persona.name))
            .collect();

        Ok(participant_ids)
    }

    /// Sets the execution strategy for the dialogue.
    ///
    /// # Arguments
    ///
    /// * `strategy` - The execution strategy to use (e.g., "broadcast", "sequential")
    ///
    /// # Note
    ///
    /// This will invalidate the current dialogue instance, which will be recreated
    /// with the new strategy on the next interaction.
    pub async fn set_execution_strategy(&self, strategy: String) {
        *self.execution_strategy.write().await = strategy;
        // Clear the dialogue to force recreation with new strategy
        *self.dialogue.lock().await = None;
    }

    /// Gets the current execution strategy.
    pub async fn get_execution_strategy(&self) -> String {
        self.execution_strategy.read().await.clone()
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
                self.handle_idle_mode(input, None, None::<fn(&DialogueMessage)>)
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
            AppMode::Idle => self.handle_idle_mode(input, file_paths, Some(on_turn)).await,
            AppMode::AwaitingConfirmation { plan } => {
                self.handle_awaiting_confirmation(input, plan)
            }
        }
    }

    /// Handles input when in Idle mode.
    async fn handle_idle_mode<F>(
        &self,
        input: &str,
        file_paths: Option<Vec<String>>,
        on_turn: Option<F>,
    ) -> InteractionResult
    where
        F: Fn(&DialogueMessage),
    {
        let trimmed = input.trim();

        if trimmed == "/plan" {
            // Create a sample plan
            let plan = Plan {
                steps: vec![
                    "Step 1: Refactor module A".to_string(),
                    "Step 2: Add tests for B".to_string(),
                ],
            };
            return InteractionResult::ModeChanged(AppMode::AwaitingConfirmation { plan });
        }

        // Ensure dialogue is initialized
        if let Err(e) = self.ensure_dialogue_initialized().await {
            return InteractionResult::NewMessage(format!("Error initializing dialogue: {}", e));
        }

        // Add user input to history BEFORE running dialogue (so timestamp is correct)
        let user_name = self.user_service.get_user_name();
        self.add_to_history(&user_name, MessageRole::User, input)
            .await;

        // Run the dialogue with the user's input using partial_session for streaming
        let mut dialogue_guard = self.dialogue.lock().await;
        let dialogue = dialogue_guard.as_mut().expect("Dialogue not initialized");

        // Create a payload with text and optional file attachments
        let mut payload = Payload::text(input.to_string());
        if let Some(paths) = file_paths {
            for path in paths {
                tracing::info!("[InteractionManager] Attaching file: {}", path);
                payload = payload.with_attachment(Attachment::local(path));
            }
        }

        // Create a partial session for incremental turn processing
        // partial_session now accepts impl Into<Payload>, so both String and Payload work
        let mut session = dialogue.partial_session(payload);
        let mut messages = Vec::new();

        // Process each turn as it becomes available
        while let Some(result) = session.next_turn().await {
            match result {
                Ok(turn) => {
                    // Log the turn for debugging sequential execution with timestamp
                    let preview: String = turn.content.chars().take(50).collect();
                    tracing::debug!(
                        "[DIALOGUE] Turn received: {} - {}...",
                        turn.participant_name,
                        preview
                    );

                    // Convert participant_name (display name) to persona_id (UUID)
                    let persona_id = self
                        .get_persona_id_by_name(&turn.participant_name)
                        .unwrap_or_else(|| turn.participant_name.clone());

                    // Add each response to history using persona_id
                    self.add_to_history(&persona_id, MessageRole::Assistant, &turn.content)
                        .await;

                    // Create DialogueMessage (still using name for UI display)
                    let message = DialogueMessage {
                        author: turn.participant_name.clone(),
                        content: turn.content.clone(),
                    };

                    // Call the streaming callback if provided
                    if let Some(ref callback) = on_turn {
                        callback(&message);
                    }

                    messages.push(message);
                }
                Err(e) => {
                    return InteractionResult::NewMessage(format!("Error: {}", e));
                }
            }
        }

        InteractionResult::NewDialogueMessages(messages)
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
    async fn add_to_history(&self, persona_id: &str, role: MessageRole, content: &str) {
        let mut histories = self.persona_histories.write().await;
        let history = histories
            .entry(persona_id.to_string())
            .or_insert_with(Vec::new);

        history.push(ConversationMessage {
            role,
            content: content.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
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

    async fn to_session(&self, app_mode: AppMode, workspace_id: Option<String>) -> Session {
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
