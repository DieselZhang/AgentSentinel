use std::fmt;

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

/// Dangerous command substrings shared between the runtime policy and the
/// eval-server scoring so the two never drift apart.
pub const DANGEROUS_COMMAND_PATTERNS: &[&str] = &[
    "rm -rf",
    "rm -r",
    "mkfs",
    "dd if=",
    ":(){ :|:& };:",
    "chmod -R 777",
    "chmod 777",
    "> /dev/",
    "sudo",
    "curl",
    "| sh",
    "wget",
    "-o-",
    "git push --force",
    "git push -f",
];

/// Protected file paths that should never be read or written.
pub const SENSITIVE_PATHS: &[&str] = &[
    "/etc/passwd",
    "/etc/shadow",
    "/etc/sudoers",
    "~/.ssh",
    "/root/",
];

/// A security policy that blocks dangerous bash commands and writes
/// to protected file paths.
pub struct DenyDangerous {
    blocked_commands: &'static [&'static str],
    blocked_paths: &'static [&'static str],
}

impl Default for DenyDangerous {
    fn default() -> Self {
        DenyDangerous {
            blocked_commands: DANGEROUS_COMMAND_PATTERNS,
            blocked_paths: SENSITIVE_PATHS,
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
