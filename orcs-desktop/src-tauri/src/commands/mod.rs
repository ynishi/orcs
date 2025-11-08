pub mod files;
pub mod git;
pub mod paths;
pub mod personas;
pub mod session;
pub mod tasks;
pub mod user;
pub mod workspaces;

pub use files::*;
pub use git::*;
pub use paths::*;
pub use personas::*;
pub use session::*;
pub use tasks::*;
pub use user::*;
pub use workspaces::*;

pub fn handlers() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        session::create_session,
        session::create_config_session,
        session::list_sessions,
        tasks::list_tasks,
        tasks::delete_task,
        personas::create_adhoc_persona,
        personas::save_adhoc_persona,
        session::switch_session,
        session::delete_session,
        session::rename_session,
        session::toggle_session_favorite,
        session::toggle_session_archive,
        session::update_session_sort_order,
        session::save_current_session,
        session::append_system_messages,
        session::get_active_session,
        personas::get_personas,
        personas::save_persona_configs,
        personas::get_persona_backend_options,
        user::get_user_nickname,
        user::get_user_profile,
        session::execute_message_as_task,
        session::add_participant,
        session::remove_participant,
        session::get_active_participants,
        session::set_execution_strategy,
        session::get_execution_strategy,
        session::set_conversation_mode,
        session::get_conversation_mode,
        session::set_talk_style,
        session::get_talk_style,
        paths::get_config_path,
        paths::get_sessions_directory,
        paths::get_workspaces_directory,
        paths::get_workspaces_repository_directory,
        paths::get_personas_directory,
        paths::get_slash_commands_directory,
        tasks::get_tasks_directory,
        paths::get_root_pathectory,
        paths::get_logs_directory,
        paths::get_secret_path,
        paths::get_default_workspace_path,
        git::get_git_info,
        workspaces::get_current_workspace,
        workspaces::create_workspace,
        workspaces::create_workspace_with_session,
        workspaces::list_workspaces,
        workspaces::switch_workspace,
        workspaces::toggle_favorite_workspace,
        workspaces::delete_workspace,
        workspaces::list_workspace_files,
        workspaces::upload_file_to_workspace,
        workspaces::upload_file_from_bytes,
        workspaces::delete_file_from_workspace,
        workspaces::rename_file_in_workspace,
        files::read_workspace_file,
        files::save_code_snippet,
        files::open_terminal,
        session::handle_input,
        crate::slash_commands::list_slash_commands,
        crate::slash_commands::get_slash_command,
        crate::slash_commands::save_slash_command,
        crate::slash_commands::remove_slash_command,
        crate::slash_commands::expand_command_template,
        crate::slash_commands::execute_shell_command,
    ]
}

