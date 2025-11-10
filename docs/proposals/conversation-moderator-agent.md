# Conversation Moderator Agent Design

## Goal

Create an “Inner Moderator” agent that lives entirely on the backend (Tauri/Rust) and continuously observes dialogue turns to keep sessions organized. Rather than one-off utilities like automatic title generation, this moderator focuses on:  

- Detecting conversational signals (long idle stretches, TODO statements, conflicts).  
- Triggering `SessionEvent`s to change conversation mode, add/remove personas, or enqueue follow-up tasks.  
- Emitting backend-owned `SystemMessage`s so every client sees the same moderation decisions.

## Responsibilities

| Scope | Description |
| --- | --- |
| Observation | Subscribe to dialogue turn stream (`DialogueMessage`s) and to persisted `SystemMessage`s. |
| Reasoning | Apply rule-based or LLM-driven heuristics (e.g., TODO detection, sentiment, progress tracking). |
| Actions | Publish structured events via `SessionEvent::ModeratorAction`, such as `SetConversationMode`, `AddParticipant`, `TriggerSlashCommand`. |
| Transparency | Emit moderator system logs (e.g., “🛎 Moderator switched mode to review because …”) through backend push channels. |

## Event & Data Model

1. **SessionEvent Enum (backend)**  
   ```rust
   enum SessionEvent {
       UserInput { content: String, attachments: Vec<String> },
       SystemEvent { content: String, metadata: MessageMetadata },
       CommandResult { command: String, output: String, deliver_to_agent: bool },
       ModeratorAction(ModeratorAction),
   }
   ```

2. **ModeratorAction Enum**  
   ```rust
   enum ModeratorAction {
       SetConversationMode(ConversationMode),
       SetTalkStyle(Option<TalkStyle>),
       AddParticipant { persona_id: String },
       RemoveParticipant { persona_id: String },
       ScheduleSlashCommand { command: String, args: String, persist_result: bool },
       AppendSystemMessage { content: String, message_type: String },
   }
   ```

3. **Event Flow**  
   - UI posts only `SessionEvent::UserInput` or `CommandResult`.  
   - Moderator listens to dialogue broadcasts and issues `SessionEvent::ModeratorAction`.  
   - `InteractionManager` applies actions atomically (e.g., change mode, run slash command) and records system messages centrally.

## Architecture

```
┌──────────────┐     SessionEvent      ┌────────────────────┐
│ React / UI   │ ───────────────────► │ Tauri Command Bus  │
└──────────────┘                      └─────────┬──────────┘
                                                │
                                      publish_session_event()
                                                │
                                     ┌──────────▼─────────┐
                                     │ InteractionManager │
                                     │  + Moderator Hook  │
                                     └────────┬───────────┘
                                              │
                         dialogue_turn stream │ moderator subscribes
                                              ▼
                                   ┌─────────────────┐
                                   │ ModeratorAgent  │
                                   └─────────────────┘
```

### ModeratorAgent in Tauri

- Lives inside the Tauri backend, instantiated alongside `AppState`.  
- Receives a `tokio::broadcast::Receiver<DialogueMessage>` and `tokio::broadcast::Receiver<SystemEvent>`.  
- Runs an async loop:
  1. Wait for new turn.  
  2. Evaluate heuristics / LLM call.  
  3. Emit `SessionEvent::ModeratorAction` via the same command bus (or directly call `InteractionManager` APIs).  
  4. Inject moderator system logs with `add_system_conversation_message`.

### Reaction Hook

To avoid duplicating logic, `llm_toolkit::agent::dialogue::Dialogue` gets a `with_reaction_handler(handler)` API, where `handler: Fn(&DialogueTurn) -> Vec<ModeratorAction>`. Initial version can be implemented in `InteractionManager` (Rust) by:

```rust
dialogue.set_reaction_handler(|turn| moderator.evaluate(turn));
```

This keeps the moderator close to the dialogue execution loop and ensures actions happen synchronously relative to new turns.

## Implementation Plan

1. **SessionEvent Infrastructure**
   - Define enum + Tauri command `publish_session_event`.
   - Update frontend to send user messages through this path (compat shim for existing `handle_input`).

2. **Broadcast Channels**
   - `InteractionManager` publishes dialogue turns + system events via `tokio::broadcast`.  
   - Tauri command that previously pushed via callback now subscribes to these channels.

3. **ModeratorAgent MVP**
   - Implement a Rust service spawned in `app/bootstrap.rs` after `AppState` creation.
   - Subscribe to turn broadcast; add simple heuristics:
     - Auto title update using first user message.
     - Mode switch to `Review` when assistant output contains “TODO” or “review”.
     - Emit a system note when conversation goes idle for X minutes.

4. **Action Execution**
   - Extend `InteractionManager` with methods invoked by moderator (set mode, add participant, run slash command).  
   - All actions emit backend-owned system messages so clients remain in sync.

5. **LLM-Based Moderator (optional)**
   - Introduce a lightweight persona (e.g., GPT/Claude) dedicated to moderation.  
   - Feed it turn summaries + metadata, and let it output `ModeratorAction` JSON that the runtime parses.

## Open Questions

- How to prevent moderator loops (e.g., action triggers another evaluation). Need debounce/cooldown.
- Access control: ensure moderator actions are distinguishable from user commands for auditing.
- Performance: when multiple sessions run, the moderator must handle per-session state; consider multiplexing via session IDs.

By grounding moderation in backend events and Reaction hooks, we can build a truly autonomous multi-persona experience and set the stage for future async APIs.
