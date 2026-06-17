//! User-configurable notification appearance.

use std::collections::BTreeMap;

/// Configuration for notification appearance.
#[derive(Debug, Clone)]
pub struct NotificationConfig {
    /// Whether notifications are enabled
    pub enabled: bool,
    /// Icon for the attention state (e.g., "🚨")
    pub attention_icon: String,
    /// Icon for the working state (e.g., "⏳")
    pub working_icon: String,
    /// Icon for the done state (e.g., "✅")
    pub done_icon: String,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            attention_icon: "🚨".to_string(),
            working_icon: "⏳".to_string(),
            done_icon: "✅".to_string(),
        }
    }
}

impl NotificationConfig {
    /// Parse configuration from Zellij layout configuration.
    ///
    /// Accepts flat key-value pairs:
    /// - `enabled`: "true" enables, anything else disables
    /// - `attention_icon` / `working_icon` / `done_icon`: icon strings (warn if > 4 chars)
    ///
    /// Missing keys fall back to defaults.
    pub fn from_configuration(config: &BTreeMap<String, String>) -> Self {
        let mut result = Self::default();

        if let Some(enabled) = config.get("enabled") {
            result.enabled = enabled == "true";
        }

        result.attention_icon = parse_icon(config, "attention_icon", result.attention_icon);
        result.working_icon = parse_icon(config, "working_icon", result.working_icon);
        result.done_icon = parse_icon(config, "done_icon", result.done_icon);

        result
    }
}

/// Read an icon override for `key`, warning if it is unusually wide; keeps `default` if absent.
fn parse_icon(config: &BTreeMap<String, String>, key: &str, default: String) -> String {
    match config.get(key) {
        Some(icon) => {
            if icon.chars().count() > 4 {
                eprintln!(
                    "zellij-attention: Warning: {} '{}' is longer than 4 chars, may not display well",
                    key, icon
                );
            }
            icon.clone()
        }
        None => default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NotificationConfig::default();
        assert!(config.enabled);
        assert_eq!(config.attention_icon, "🚨");
        assert_eq!(config.working_icon, "⏳");
        assert_eq!(config.done_icon, "✅");
    }

    #[test]
    fn test_from_configuration_empty() {
        let config_map = BTreeMap::new();
        let config = NotificationConfig::from_configuration(&config_map);
        // Should use defaults
        assert!(config.enabled);
        assert_eq!(config.attention_icon, "🚨");
        assert_eq!(config.working_icon, "⏳");
        assert_eq!(config.done_icon, "✅");
    }

    #[test]
    fn test_from_configuration_custom() {
        let mut config_map = BTreeMap::new();
        config_map.insert("enabled".to_string(), "true".to_string());
        config_map.insert("attention_icon".to_string(), "A".to_string());
        config_map.insert("working_icon".to_string(), "W".to_string());
        config_map.insert("done_icon".to_string(), "D".to_string());

        let config = NotificationConfig::from_configuration(&config_map);
        assert!(config.enabled);
        assert_eq!(config.attention_icon, "A");
        assert_eq!(config.working_icon, "W");
        assert_eq!(config.done_icon, "D");
    }

    #[test]
    fn test_from_configuration_disabled() {
        let mut config_map = BTreeMap::new();
        config_map.insert("enabled".to_string(), "false".to_string());

        let config = NotificationConfig::from_configuration(&config_map);
        assert!(!config.enabled);
    }
}
