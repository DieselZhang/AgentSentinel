use serde::Deserialize;
use std::fmt;
use std::sync::LazyLock;

#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
    Allow,
    AskUser,
    Deny { reason: String },
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Permission::Allow => write!(f, "allow"),
            Permission::AskUser => write!(f, "ask_user"),
            Permission::Deny { reason } => write!(f, "deny: {}", reason),
        }
    }
}

pub trait PermissionPolicy: Send + Sync {
    fn check(&self, tool_name: &str, arguments: &serde_json::Value) -> Permission;
}

pub struct AllowAll;

impl PermissionPolicy for AllowAll {
    fn check(&self, _tool_name: &str, _arguments: &serde_json::Value) -> Permission {
        Permission::Allow
    }
}

/// The shared danger-pattern data file. Single source of truth for the Rust
/// runtime policy, the Rust eval-server scorer, and the Python audit SDK —
/// edit the JSON, not the code, so the three never drift apart.
#[derive(Deserialize)]
struct PatternsFile {
    dangerous_command_patterns: Vec<String>,
    sensitive_paths: Vec<String>,
}

/// Load `patterns.json` from the repo root (one level above `agent-runtime/`)
/// at compile time, so the shipped binary stays dependency-free at runtime.
fn load_patterns() -> PatternsFile {
    serde_json::from_str(include_str!("../../patterns.json"))
        .expect("patterns.json should be valid JSON")
}

/// Dangerous command substrings shared between the runtime policy and the
/// eval-server scoring so the two never drift apart.
pub static DANGEROUS_COMMAND_PATTERNS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    load_patterns()
        .dangerous_command_patterns
        .into_iter()
        .map(|s| &*Box::leak(s.into_boxed_str()))
        .collect()
});

/// Protected file paths that should never be read or written.
pub static SENSITIVE_PATHS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    load_patterns()
        .sensitive_paths
        .into_iter()
        .map(|s| &*Box::leak(s.into_boxed_str()))
        .collect()
});

/// A security policy that blocks dangerous bash commands and writes
/// to protected file paths.
pub struct DenyDangerous {
    blocked_commands: &'static [&'static str],
    blocked_paths: &'static [&'static str],
}

impl Default for DenyDangerous {
    fn default() -> Self {
        DenyDangerous {
            blocked_commands: &DANGEROUS_COMMAND_PATTERNS[..],
            blocked_paths: &SENSITIVE_PATHS[..],
        }
    }
}

impl PermissionPolicy for DenyDangerous {
    fn check(&self, tool_name: &str, arguments: &serde_json::Value) -> Permission {
        match tool_name {
            "bash" => {
                if let Some(command) = arguments["command"].as_str() {
                    for blocked in self.blocked_commands.iter() {
                        if command.contains(*blocked) {
                            return Permission::Deny {
                                reason: format!("blocked dangerous command pattern: {}", blocked),
                            };
                        }
                    }
                }
            }
            "write_file" => {
                if let Some(file_path) = arguments["file_path"].as_str() {
                    for blocked in self.blocked_paths.iter() {
                        if file_path.starts_with(*blocked) {
                            return Permission::Deny {
                                reason: format!("blocked protected path: {}", blocked),
                            };
                        }
                    }
                }
            }
            _ => {}
        }
        Permission::Allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allow_all() {
        let policy = AllowAll;
        let args = serde_json::json!({"command": "rm -rf /"});
        assert_eq!(policy.check("bash", &args), Permission::Allow);
    }

    #[test]
    fn test_deny_rm_rf() {
        let policy = DenyDangerous::default();
        let args = serde_json::json!({"command": "rm -rf / --no-preserve-root"});
        match policy.check("bash", &args) {
            Permission::Deny { reason } => assert!(reason.contains("rm -rf")),
            _ => panic!("expected Deny"),
        }
    }

    #[test]
    fn test_deny_protected_path() {
        let policy = DenyDangerous::default();
        let args = serde_json::json!({"file_path": "/etc/passwd", "content": "hacked"});
        match policy.check("write_file", &args) {
            Permission::Deny { reason } => assert!(reason.contains("/etc/passwd")),
            _ => panic!("expected Deny"),
        }
    }

    #[test]
    fn test_allow_safe_command() {
        let policy = DenyDangerous::default();
        let args = serde_json::json!({"command": "ls -la"});
        assert_eq!(policy.check("bash", &args), Permission::Allow);
    }
}
