use std::borrow::Cow::{self, Borrowed, Owned};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use colored::Colorize;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use rustyline::Editor;
use tokio::sync::mpsc;
use tokio::time::timeout;

use orcs_core::TaskManager;
use orcs_core::user_service::{UserService, DefaultUserService};
use orcs_execution::TaskExecutor;
use orcs_infrastructure::repository::TomlPersonaRepository;
use orcs_interaction::{DialogueMessage, InteractionManager, InteractionResult};
use orcs_types::{AppMode, TaskContext};

/// CLI helper for rustyline that provides completion, highlighting, and hints.
#[derive(Clone)]
struct CliHelper {
    commands: Vec<String>,
}

impl CliHelper {
    fn new() -> Self {
        Self {
            commands: vec!["/plan".to_string(), "/tasks".to_string()],
        }
    }
}

impl Helper for CliHelper {}

impl Completer for CliHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let line = &line[..pos];

        if line.starts_with('/') {
            let candidates: Vec<Pair> = self
                .commands
                .iter()
                .filter(|cmd| cmd.starts_with(line))
                .map(|cmd| Pair {
                    display: cmd.clone(),
                    replacement: cmd.clone(),
                })
                .collect();
            Ok((0, candidates))
        } else {
            Ok((0, vec![]))
        }
    }
}

impl Highlighter for CliHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        if line.starts_with('/') {
            Owned(line.bright_cyan().to_string())
        } else {
            Borrowed(line)
        }
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: bool) -> bool {
        true
    }
}

impl Hinter for CliHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<String> {
        let line = &line[..pos];

        if line.starts_with('/') && !line.contains(' ') {
            self.commands
                .iter()
                .find(|cmd| cmd.starts_with(line) && cmd.len() > line.len())
                .map(|cmd| cmd[line.len()..].to_string())
        } else {
            None
        }
    }
}

impl Validator for CliHelper {}

/// The main entry point for the Orcs readline REPL application.
///
/// This async function sets up a rustyline-based REPL that:
/// 1. Initializes async backend components (TaskManager, TaskExecutor, InteractionManager)
/// 2. Sets up mpsc channels for task and response communication
/// 3. Provides command completion for /plan and /tasks
/// 4. Handles user input asynchronously without blocking
/// 5. Displays colored output for user, AI, and system messages
#[tokio::main]
async fn main() -> Result<()> {
    // ===== Backend Initialization =====
    // Initialize task management components
    let _task_manager = TaskManager::new();
    let task_executor = TaskExecutor::new();
    let persona_repository = Arc::new(TomlPersonaRepository);
    let user_service: Arc<dyn UserService> = Arc::new(DefaultUserService::default());
    let interaction_manager = Arc::new(InteractionManager::new_session("cli-session".to_string(), persona_repository, user_service));

    // Create a channel for sending tasks to the background executor
    let (task_tx, mut task_rx) = mpsc::channel::<TaskContext>(100);

    // Create a channel for receiving AI responses from background tasks
    let (response_tx, mut response_rx) = mpsc::channel::<InteractionResult>(32);

    // Spawn background task executor
    tokio::spawn(async move {
        while let Some(task) = task_rx.recv().await {
            if let Err(e) = task_executor.execute(&task).await {
                eprintln!("{}", format!("Task execution failed: {:?}", e).red());
            }
        }
    });

    // Spawn response handler task
    let response_handler_task_tx = task_tx.clone();
    let response_handler = tokio::spawn(async move {
        let mut app_mode = AppMode::Idle;
        let mut task_manager_handler = TaskManager::new();

        while let Some(result) = response_rx.recv().await {
            match result {
                InteractionResult::ModeChanged(new_mode) => {
                    // Handle mode change (e.g., to AwaitingConfirmation)
                    if let AppMode::AwaitingConfirmation { ref plan } = new_mode {
                        println!("{}", "A plan has been proposed:".bright_yellow());
                        for step in &plan.steps {
                            println!("  {}", format!("- {}", step).yellow());
                        }
                        println!("{}", "Please confirm (yes/no)".bright_yellow());
                    }
                    // Transition to the new mode
                    app_mode = new_mode;
                }
                InteractionResult::TasksToDispatch { tasks } => {
                    // Iterate over the tasks and create real tasks
                    for task_description in tasks {
                        // Create task from description
                        let task = task_manager_handler.create_task(&task_description).clone();

                        // Display task creation
                        println!("{}", format!("Task created: {}", task_description).green());

                        // Send to background executor
                        if let Err(e) = response_handler_task_tx.send(task).await {
                            eprintln!("{}", format!("Failed to send task: {:?}", e).red());
                        }
                    }

                    // Change mode back to Idle
                    app_mode = AppMode::Idle;
                }
                InteractionResult::NewMessage(message) => {
                    // Display the message to the user with AI color
                    for line in message.lines() {
                        println!("{}", line.bright_blue());
                    }
                }
                InteractionResult::NewDialogueMessages(messages) => {
                    // Display dialogue messages with author names
                    for DialogueMessage { author, content } in messages {
                        println!("{}", format!("[{}]", author).bright_magenta());
                        for line in content.lines() {
                            println!("{}", line.bright_blue());
                        }
                        println!(); // Add spacing between messages
                    }
                }
                InteractionResult::NoOp => {
                    // Add a simple acknowledgment for unknown commands
                    if matches!(app_mode, AppMode::Idle) {
                        println!("{}", "Unknown command".bright_black());
                    }
                }
            }
        }
    });

    // ===== REPL Setup =====
    let helper = CliHelper::new();
    let mut rl = Editor::new()?;
    rl.set_helper(Some(helper));

    println!("{}", "=== Orcs REPL ===".bright_magenta().bold());
    println!("{}", "Type '/plan' to create a plan, '/tasks' to view tasks, or 'quit' to exit.".bright_black());
    println!();

    // Track the current mode in the REPL
    let app_mode = AppMode::Idle;

    // ===== Main REPL Loop =====
    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                let trimmed = line.trim();

                // Handle quit command
                if trimmed == "quit" || trimmed == "exit" {
                    println!("{}", "Goodbye!".bright_green());
                    break;
                }

                // Skip empty lines
                if trimmed.is_empty() {
                    continue;
                }

                // Add to history
                let _ = rl.add_history_entry(&line);

                // Display user input in green
                println!("{}", format!("> {}", trimmed).green());

                // Clone necessary variables for background task
                let tx = response_tx.clone();
                let mode = app_mode.clone();
                let input = trimmed.to_string();
                let manager = Arc::clone(&interaction_manager);

                // Spawn background task for AI interaction
                tokio::spawn(async move {
                    // Wrap the handle_input call in a timeout
                    let agent_response = timeout(
                        Duration::from_secs(30),
                        manager.handle_input(&mode, &input)
                    ).await;

                    // Check if timeout occurred
                    let result = match agent_response {
                        Ok(result) => result,
                        Err(_) => InteractionResult::NewMessage("Error: Request timed out.".to_string()),
                    };

                    // Send result back through channel
                    let _ = tx.send(result).await;
                });

                // Update app_mode if we receive a mode change
                // Note: In a real implementation, we'd need to handle this more carefully
                // For now, we'll rely on the response handler to manage mode transitions
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("{}", "CTRL-C detected. Type 'quit' to exit.".yellow());
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("{}", "CTRL-D detected. Exiting...".bright_green());
                break;
            }
            Err(err) => {
                eprintln!("{}", format!("Error: {:?}", err).red());
                break;
            }
        }
    }

    // Drop channels to signal shutdown
    drop(response_tx);
    drop(task_tx);

    // Wait for response handler to finish
    let _ = response_handler.await;

    Ok(())
}
