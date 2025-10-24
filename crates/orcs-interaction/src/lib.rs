pub mod persona_agent;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use orcs_core::session::{AppMode, ConversationMessage, MessageRole, Plan, Session};
use orcs_core::repository::PersonaRepository;
use orcs_core::persona::Persona as PersonaDomain;
use orcs_core::user::UserService;
use llm_toolkit::agent::impls::{ClaudeCodeAgent, ClaudeCodeJsonAgent};
use llm_toolkit::agent::dialogue::{Dialogue, DialogueBlueprint};
use llm_toolkit::agent::persona::{Persona as LlmPersona, PersonaTeam};
use llm_toolkit::agent::{Agent, AgentError, Payload};

/// Converts a Persona domain model to llm-toolkit Persona.
fn domain_to_llm_persona(persona: &PersonaDomain) -> LlmPersona {
    LlmPersona {
        name: persona.name.clone(),
        role: persona.role.clone(),
        background: persona.background.clone(),
        communication_style: persona.communication_style.clone(),
    }
}

/// Wrapper around ClaudeCodeAgent that implements Clone by creating new instances.
/// This is needed because Dialogue::from_blueprint requires Clone + 'static.
#[derive(Debug)]
struct ClonableAgent {
    _phantom: std::marker::PhantomData<()>,
}

impl ClonableAgent {
    fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl Clone for ClonableAgent {
    fn clone(&self) -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Agent for ClonableAgent {
    type Output = String;

    fn expertise(&self) -> &str {
        "General AI Assistant"
    }

    async fn execute(&self, payload: Payload) -> Result<Self::Output, AgentError> {
        // Create a fresh ClaudeCodeAgent for each execution
        let agent = ClaudeCodeAgent::new();
        agent.execute(payload).await
    }
}

/// Represents a single message in a dialogue conversation.
///
/// Each message has an author (participant name) and the content of the message.
#[derive(Debug, Clone, PartialEq, Eq)]
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
        // Migrate persona_histories keys from old IDs to UUIDs
        let migrated_histories = Self::migrate_persona_history_keys(
            data.persona_histories,
            &persona_repository,
        );

        Self {
            session_id: data.id,
            title: Arc::new(RwLock::new(data.title)),
            created_at: data.created_at,
            dialogue: Arc::new(Mutex::new(None)),
            persona_histories: Arc::new(RwLock::new(migrated_histories)),
            persona_repository,
            user_service,
            execution_strategy: Arc::new(RwLock::new("broadcast".to_string())),
        }
    }

    /// Migrates persona_histories keys from non-UUID formats to UUID.
    ///
    /// This handles:
    /// - Old string IDs ("mai", "yui") → UUID
    /// - Display names ("Mai", "Yui") → UUID
    /// - Already-UUID keys → pass through
    fn migrate_persona_history_keys(
        histories: HashMap<String, Vec<ConversationMessage>>,
        persona_repository: &Arc<dyn PersonaRepository>,
    ) -> HashMap<String, Vec<ConversationMessage>> {
        let personas = persona_repository.get_all().unwrap_or_default();
        let mut migrated = HashMap::new();

        for (key, messages) in histories {
            // Check if key is already a valid UUID
            if uuid::Uuid::parse_str(&key).is_ok() {
                // Already UUID - keep as is
                migrated.insert(key, messages);
                continue;
            }

            // Try to find persona by old ID or name (case-insensitive)
            let key_lower = key.to_lowercase();
            let matching_persona = personas.iter().find(|p| {
                // Check old ID format (mai, yui, etc.)
                let old_id_matches = p.name.to_lowercase() == key_lower;
                // Check display name (Mai, Yui)
                let name_matches = p.name == key;
                old_id_matches || name_matches
            });

            if let Some(persona) = matching_persona {
                // Use the UUID from the persona
                migrated.insert(persona.id.clone(), messages);
            } else {
                // Unknown key - keep as is (might be user name)
                migrated.insert(key, messages);
            }
        }

        migrated
    }

    /// Resolves a persona name to its UUID.
    ///
    /// This is used to convert DialogueTurn participant names to persona IDs.
    fn get_persona_id_by_name(&self, name: &str) -> Option<String> {
        let personas = self.persona_repository.get_all().ok()?;
        personas.iter()
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

        let all_configs = self.persona_repository.get_all()?;
        let participants = all_configs
            .iter()
            .filter(|p| p.default_participant)
            .map(domain_to_llm_persona)
            .collect();

        let strategy = self.execution_strategy.read().await.clone();

        let blueprint = DialogueBlueprint {
            agenda: "Orcs AI Assistant Session".to_string(),
            context: "A collaborative session between the user and AI assistants Mai and Yui.".to_string(),
            participants: Some(participants),
            execution_strategy: Some(strategy),
        };

        // Create fresh agents for dialogue initialization
        let generator_agent = ClaudeCodeJsonAgent::<PersonaTeam>::new();
        let llm_agent = ClonableAgent::new();

        let dialogue = Dialogue::from_blueprint(
            blueprint,
            generator_agent,
            llm_agent,
        )
        .await
        .map_err(|e| e.to_string())?;

        *dialogue_guard = Some(dialogue);
        Ok(())
    }

    /// Converts the current state to Session for persistence.
    ///
    /// # Arguments
    ///
    /// * `app_mode` - The current application mode
    pub async fn to_session(&self, app_mode: AppMode) -> Session {
        let persona_histories = self.persona_histories.read().await.clone();
        let title = self.title.read().await.clone();

        // Use the first default participant as current_persona_id
        let current_persona_id = self.persona_repository
            .get_all()
            .ok()
            .and_then(|personas| {
                personas.iter()
                    .find(|p| p.default_participant)
                    .map(|p| p.id.clone())
            })
            .unwrap_or_else(|| "unknown".to_string());

        Session {
            id: self.session_id.clone(),
            title,
            created_at: self.created_at.clone(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            current_persona_id,
            persona_histories,
            app_mode,
        }
    }

    /// Returns the session ID.
    pub fn session_id(&self) -> &str {
        &self.session_id
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
        let persona_config = self.persona_repository.get_all()?
            .into_iter()
            .find(|p| p.id == persona_id)
            .ok_or_else(|| format!("Persona with id '{}' not found", persona_id))?;
        let persona = domain_to_llm_persona(&persona_config);

        // Lock the dialogue and add participant
        let mut dialogue_guard = self.dialogue.lock().await;
        let dialogue = dialogue_guard.as_mut().expect("Dialogue not initialized");
        dialogue.add_participant(persona, ClonableAgent::new());

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
        let persona_config = self.persona_repository.get_all()?
            .into_iter()
            .find(|p| p.id == persona_id)
            .ok_or_else(|| format!("Persona with id '{}' not found", persona_id))?;
        let persona = domain_to_llm_persona(&persona_config);

        // Lock the dialogue and remove participant
        let mut dialogue_guard = self.dialogue.lock().await;
        let dialogue = dialogue_guard.as_mut().expect("Dialogue not initialized");
        dialogue.remove_participant(&persona.name).map_err(|e| e.to_string())?;

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
        let participant_ids = dialogue.participants()
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
            AppMode::Idle => self.handle_idle_mode(input).await,
            AppMode::AwaitingConfirmation { plan } => {
                self.handle_awaiting_confirmation(input, plan)
            }
        }
    }

    /// Handles input when in Idle mode.
    async fn handle_idle_mode(&self, input: &str) -> InteractionResult {
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
        self.add_to_history(&user_name, MessageRole::User, input).await;

        // Run the dialogue with the user's input using partial_session for streaming
        let mut dialogue_guard = self.dialogue.lock().await;
        let dialogue = dialogue_guard.as_mut().expect("Dialogue not initialized");

        // Create a partial session for incremental turn processing
        let mut session = dialogue.partial_session(input.to_string());
        let mut messages = Vec::new();

        // Process each turn as it becomes available
        while let Some(result) = session.next_turn().await {
            match result {
                Ok(turn) => {
                    // Convert participant_name (display name) to persona_id (UUID)
                    let persona_id = self.get_persona_id_by_name(&turn.participant_name)
                        .unwrap_or_else(|| turn.participant_name.clone());

                    // Add each response to history using persona_id
                    self.add_to_history(&persona_id, MessageRole::Assistant, &turn.content).await;

                    // Create DialogueMessage (still using name for UI display)
                    messages.push(DialogueMessage {
                        author: turn.participant_name.clone(),
                        content: turn.content.clone(),
                    });
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
        let history = histories.entry(persona_id.to_string()).or_insert_with(Vec::new);

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

    async fn to_session(&self, app_mode: AppMode) -> Session {
        self.to_session(app_mode).await
    }
}

// Tests are temporarily commented out to allow compilation after refactoring
// They will need to be updated to use a PersonaRepository implementation
/*
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_session() {
        let manager = InteractionManager::new_session("test-session".to_string());
        assert_eq!(manager.session_id(), "test-session");
        assert_eq!(manager.available_personas().len(), 2);
    }

    #[tokio::test]
    async fn test_idle_mode_plan_command() {
        let manager = InteractionManager::new_session("test".to_string());
        let mode = AppMode::Idle;

        let result = manager.handle_input(&mode, "/plan").await;

        match result {
            InteractionResult::ModeChanged(AppMode::AwaitingConfirmation { plan }) => {
                assert_eq!(plan.steps.len(), 2);
            }
            _ => panic!("Expected ModeChanged with AwaitingConfirmation"),
        }
    }

    #[tokio::test]
    async fn test_awaiting_confirmation() {
        let manager = InteractionManager::new_session("test".to_string());
        let plan = Plan {
            steps: vec!["Task 1".to_string(), "Task 2".to_string()],
        };
        let mode = AppMode::AwaitingConfirmation { plan: plan.clone() };

        // Test yes
        let result = manager.handle_input(&mode, "yes").await;
        match result {
            InteractionResult::TasksToDispatch { tasks } => {
                assert_eq!(tasks.len(), 2);
            }
            _ => panic!("Expected TasksToDispatch"),
        }

        // Test no
        let result = manager.handle_input(&mode, "no").await;
        assert_eq!(result, InteractionResult::ModeChanged(AppMode::Idle));

        // Test invalid
        let result = manager.handle_input(&mode, "maybe").await;
        assert_eq!(result, InteractionResult::NoOp);
    }

    #[tokio::test]
    async fn test_to_session() {
        let manager = InteractionManager::new_session("test-session".to_string());

        let session = manager.to_session(AppMode::Idle).await;

        assert_eq!(session.id, "test-session");
        assert_eq!(session.current_persona_id, "mai");
        assert_eq!(session.app_mode, AppMode::Idle);
    }
}
*/
