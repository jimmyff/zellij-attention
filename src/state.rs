/// Per-pane notification state, ordered by display priority.
///
/// A tab showing several Claude panes renders the highest-priority state
/// (see [`NotificationType::priority`]).
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum NotificationType {
    /// Needs the user (highest): `Notification` / `PermissionRequest` hooks.
    Attention,
    /// Claude is actively working: `UserPromptSubmit` / `PreToolUse` hooks.
    Working,
    /// Claude finished — a completion flag that focus clears: `Stop` hook.
    Done,
}

impl NotificationType {
    /// Higher wins when a tab has several Claude panes: attention > working > done.
    pub fn priority(&self) -> u8 {
        match self {
            NotificationType::Attention => 3,
            NotificationType::Working => 2,
            NotificationType::Done => 1,
        }
    }
}
