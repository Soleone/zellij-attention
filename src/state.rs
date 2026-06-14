use serde::{Deserialize, Serialize};

/// Types of notifications a pane can have.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum NotificationType {
    /// Agent is thinking
    Thinking,
    /// Agent is running a shell command
    Bash,
    /// Agent is reading/searching files
    Read,
    /// Agent is editing/writing files
    Edit,
    /// Agent is spawning or waiting on a subagent
    Subagent,
    /// Agent is using web search/fetch
    Web,
    /// Agent is using another tool
    Other,
    /// Agent is waiting for user input
    Waiting,
    /// Agent has completed
    Completed,
    /// Agent is idle
    Idle,
}

impl NotificationType {
    pub fn from_event_type(event_type: &str) -> Option<Self> {
        match event_type.to_lowercase().as_str() {
            "thinking" => Some(Self::Thinking),
            "bash" | "running_bash" | "running-bash" => Some(Self::Bash),
            "read" | "reading" | "search" | "searching" => Some(Self::Read),
            "edit" | "editing" | "write" | "writing" => Some(Self::Edit),
            "subagent" | "spawning_subagent" | "spawning-subagent" => Some(Self::Subagent),
            "web" | "web_search" | "web-search" | "fetch" => Some(Self::Web),
            "other" | "tool" | "other_tool" | "other-tool" => Some(Self::Other),
            "waiting" | "prompt" | "user_prompt" | "user-prompt" => Some(Self::Waiting),
            "completed" | "done" => Some(Self::Completed),
            "idle" => Some(Self::Idle),
            _ => None,
        }
    }
}
