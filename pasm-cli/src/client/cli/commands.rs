use crate::client::auth::master;
use crate::client::curl::requests;
use crate::client::entry;

/// All CLI subcommands supported by pasm_client.
///
/// Each variant maps to a command string from CLI args. Variants that require
/// additional data carry associated fields (e.g., `Find { name }`).
///
/// Commands that manage credentials (`Create`, `Find`, `List`, `Delete`,
/// `Amend`) require an active session (api_key + encr_key). `Login` and
/// `Logout` work without a session.
#[derive(Debug)]
pub enum CliCommand {
    /// Log in with master password — creates or verifies password, sets up session.
    Login,
    /// Log out — clears the local session file.
    Logout,
    /// Create a new entry via interactive prompts (encrypts before upload).
    Create,
    /// Find an entry by name and display decrypted details.
    Find { name: String },
    /// List all entries with their decrypted details.
    List,
    /// Delete an entry by name.
    Delete { name: String },
    /// Amend (create or overwrite) an entry via interactive prompts.
    Amend,
    /// Register the current api_key with the server (auto-done on first login).
    Register,
    /// Replace the current api_key with a new one (server-side key rotation).
    /// Takes the new key as a CLI argument.
    UpdateAuth { new_key: String },
    /// Remove the current user and all their data from the server.
    RemoveAuth,
    /// List all registered users (auth keys) on the server.
    ListUsers,
    /// Create a backup of all encrypted entries (server writes to /tmp/pasm/backups/).
    Backup,
    /// Promote a user to admin by their auth key (admin only).
    SetAdmin { key: String },
    /// Show this help message.
    Help,
}

impl CliCommand {
    /// Parses CLI arguments into a `CliCommand`, optional `--addr`, and optional `--config`.
    ///
    /// Global flags (`--addr`, `--config`) can appear before the command name:
    /// ```text
    /// pasm_client --config ~/pasm.toml --addr 192.168.1.5:3000 find github
    /// ```
    ///
    /// # Arguments
    /// * `args` - Argument strings excluding the binary name.
    ///
    /// # Returns
    /// `Ok((command, addr, config_path))` — the parsed command and optionally
    /// a server address and/or config path override.
    pub fn from_args(args: &[String]) -> Result<(Self, Option<String>, Option<String>), String> {
        let mut addr: Option<String> = None;
        let mut conf: Option<String> = None;
        let mut remaining = args.to_vec();

        loop {
            match remaining.first().map(|s| s.as_str()) {
                Some("--addr") => {
                    if remaining.len() < 2 {
                        return Err("--addr requires a value".into());
                    }
                    addr = Some(remaining[1].clone());
                    remaining = remaining[2..].to_vec();
                }
                Some("--config") => {
                    if remaining.len() < 2 {
                        return Err("--config requires a value".into());
                    }
                    conf = Some(remaining[1].clone());
                    remaining = remaining[2..].to_vec();
                }
                _ => break,
            }
        }

        let cmd = remaining.first().ok_or("missing command")?;
        let cmd = match cmd.as_str() {
            "login" => CliCommand::Login,
            "logout" => CliCommand::Logout,
            "create" => CliCommand::Create,
            "find" => {
                let name = remaining.get(1).ok_or("find requires a name")?;
                CliCommand::Find { name: name.clone() }
            }
            "list" => CliCommand::List,
            "delete" => {
                let name = remaining.get(1).ok_or("delete requires a name")?;
                CliCommand::Delete { name: name.clone() }
            }
            "amend" => CliCommand::Amend,
            "register" => CliCommand::Register,
            "update-auth" => {
                let new_key = remaining.get(1).ok_or("update-auth requires a new key")?;
                CliCommand::UpdateAuth {
                    new_key: new_key.clone(),
                }
            }
            "remove-auth" => CliCommand::RemoveAuth,
            "backup" => CliCommand::Backup,
            "set-admin" => {
                let key = remaining.get(1).ok_or("set-admin requires an auth key")?;
                CliCommand::SetAdmin { key: key.clone() }
            }
            "list-users" => CliCommand::ListUsers,
            "help" | "-h" | "--help" => CliCommand::Help,
            _ => return Err(format!("unknown command: {cmd}")),
        };
        Ok((cmd, addr, conf))
    }

    /// Executes the command, returning a human-readable result string.
    ///
    /// `Login` and `Logout` ignore the key arguments (typically passed as
    /// empty strings). All other commands use `api_key` for API
    /// authentication and `encr_key` for entry-level encryption/decryption.
    ///
    /// # Arguments
    /// * `api_key` - Bearer token for server API calls
    /// * `encr_key` - AES-256 key for entry encrypt/decrypt operations
    ///
    /// # Returns
    /// A displayable result message. May be an error message on failure.
    pub fn execute(&self, api_key: &str, encr_key: &str) -> String {
        match self {
            CliCommand::Login => master::login(),
            CliCommand::Logout => master::logout(),
            CliCommand::Create => entry::ops::create(api_key, encr_key),
            CliCommand::Find { name } => entry::ops::find(api_key, encr_key, name),
            CliCommand::List => entry::ops::list(api_key, encr_key),
            CliCommand::Delete { name } => entry::ops::delete(api_key, name),
            CliCommand::Amend => entry::ops::amend(api_key, encr_key),
            CliCommand::Register => requests::register_auth(api_key),
            CliCommand::UpdateAuth { new_key } => requests::update_auth(api_key, new_key),
            CliCommand::RemoveAuth => requests::remove_auth(api_key),
            CliCommand::ListUsers => requests::list_users(api_key),
            CliCommand::Backup => entry::ops::backup(api_key),
            CliCommand::SetAdmin { key } => requests::set_admin(api_key, key),
            CliCommand::Help => Self::usage(),
        }
    }

    /// Returns the help text listing all available commands.
    pub fn usage() -> String {
        concat!(
            "Usage: pasm_client [options] <command> [args]\n",
            "       pasm_client -h | --help\n",
            "\n",
            "Global options:\n",
            "  --config <path>     Config file path (default: ~/.config/pasm/config.toml)\n",
            "  --addr <host:port>  Server address  (default: http://localhost:3000)\n",
            "\n",
            "Session management:\n",
            "  login              Login with master password\n",
            "  logout             Log out (clear local session)\n",
            "\n",
            "Entry management (requires login):\n",
            "  create             Create an entry (interactive prompts)\n",
            "  find <name>        Find and display an entry\n",
            "  list               List all entries\n",
            "  delete <name>      Delete an entry\n",
            "  amend              Create or overwrite an entry (interactive)\n",
            "\n",
            "Account management (requires login):\n",
            "  backup             Backup all encrypted entries to a JSON file\n",
            "  set-admin <key>    Promote a user to admin (admin only)\n",
            "  register           Register current auth key with server\n",
            "  update-auth <key>  Replace auth key (key rotation)\n",
            "  remove-auth        Remove user and all data\n",
            "  list-users         List all registered users\n",
        )
        .to_string()
    }
}
