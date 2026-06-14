//! User-configurable notification appearance.

use std::collections::BTreeMap;

use crate::state::NotificationType;

/// Configuration for notification appearance.
#[derive(Debug, Clone)]
pub struct NotificationConfig {
    /// Whether notifications are enabled
    pub enabled: bool,
    pub thinking_icon: String,
    pub bash_icon: String,
    pub read_icon: String,
    pub edit_icon: String,
    pub subagent_icon: String,
    pub web_icon: String,
    pub other_icon: String,
    /// Icon for waiting state (e.g., "▶")
    pub waiting_icon: String,
    /// Icon for completed state (e.g., "✓")
    pub completed_icon: String,
    pub idle_icon: String,
    /// Clear notifications for any pane in a tab when that tab becomes active.
    pub clear_on_tab_focus: bool,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            thinking_icon: "●".to_string(),
            bash_icon: "⚡".to_string(),
            read_icon: "◉".to_string(),
            edit_icon: "✎".to_string(),
            subagent_icon: "⊜".to_string(),
            web_icon: "◈".to_string(),
            other_icon: "⚙".to_string(),
            waiting_icon: "▶".to_string(),
            completed_icon: "✓".to_string(),
            idle_icon: "○".to_string(),
            clear_on_tab_focus: true,
        }
    }
}

impl NotificationConfig {
    pub fn icon_for(&self, notification_type: NotificationType) -> &str {
        match notification_type {
            NotificationType::Thinking => &self.thinking_icon,
            NotificationType::Bash => &self.bash_icon,
            NotificationType::Read => &self.read_icon,
            NotificationType::Edit => &self.edit_icon,
            NotificationType::Subagent => &self.subagent_icon,
            NotificationType::Web => &self.web_icon,
            NotificationType::Other => &self.other_icon,
            NotificationType::Waiting => &self.waiting_icon,
            NotificationType::Completed => &self.completed_icon,
            NotificationType::Idle => &self.idle_icon,
        }
    }

    pub fn icons(&self) -> [&str; 12] {
        [
            &self.thinking_icon,
            &self.bash_icon,
            &self.read_icon,
            &self.edit_icon,
            &self.subagent_icon,
            &self.web_icon,
            &self.other_icon,
            &self.waiting_icon,
            &self.completed_icon,
            &self.idle_icon,
            // Legacy defaults, so changing config cleans up old tab suffixes.
            "⏳",
            "✅",
        ]
    }

    /// Parse configuration from Zellij layout configuration.
    pub fn from_configuration(config: &BTreeMap<String, String>) -> Self {
        let mut result = Self::default();

        if let Some(enabled) = config.get("enabled") {
            result.enabled = enabled == "true";
        }

        macro_rules! parse_icon {
            ($key:literal, $field:ident) => {
                if let Some(icon) = config.get($key) {
                    if icon.chars().count() > 4 {
                        eprintln!(
                            "zellij-attention: Warning: {} '{}' is longer than 4 chars, may not display well",
                            $key, icon
                        );
                    }
                    result.$field = icon.clone();
                }
            };
        }

        parse_icon!("thinking_icon", thinking_icon);
        parse_icon!("bash_icon", bash_icon);
        parse_icon!("read_icon", read_icon);
        parse_icon!("edit_icon", edit_icon);
        parse_icon!("subagent_icon", subagent_icon);
        parse_icon!("web_icon", web_icon);
        parse_icon!("other_icon", other_icon);
        parse_icon!("waiting_icon", waiting_icon);
        parse_icon!("completed_icon", completed_icon);
        parse_icon!("idle_icon", idle_icon);

        if let Some(clear_on_tab_focus) = config.get("clear_on_tab_focus") {
            result.clear_on_tab_focus = clear_on_tab_focus == "true";
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NotificationConfig::default();
        assert!(config.enabled);
        assert_eq!(config.thinking_icon, "●");
        assert_eq!(config.bash_icon, "⚡");
        assert_eq!(config.read_icon, "◉");
        assert_eq!(config.edit_icon, "✎");
        assert_eq!(config.subagent_icon, "⊜");
        assert_eq!(config.web_icon, "◈");
        assert_eq!(config.other_icon, "⚙");
        assert_eq!(config.waiting_icon, "▶");
        assert_eq!(config.completed_icon, "✓");
        assert_eq!(config.idle_icon, "○");
        assert!(config.clear_on_tab_focus);
    }

    #[test]
    fn test_from_configuration_custom() {
        let mut config_map = BTreeMap::new();
        config_map.insert("enabled".to_string(), "true".to_string());
        config_map.insert("waiting_icon".to_string(), "W".to_string());
        config_map.insert("completed_icon".to_string(), "D".to_string());
        config_map.insert("clear_on_tab_focus".to_string(), "false".to_string());

        let config = NotificationConfig::from_configuration(&config_map);
        assert!(config.enabled);
        assert_eq!(config.waiting_icon, "W");
        assert_eq!(config.completed_icon, "D");
        assert!(!config.clear_on_tab_focus);
    }
}
