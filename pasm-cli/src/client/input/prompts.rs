use std::io::{self, Write};
use std::process::Command;

use pasm_core::types::detail::Details;

/// Guard that restores terminal echo when dropped.
///
/// On construction, disables echo via `stty -echo`.
/// On drop (including during panic unwinding), re-enables echo via `stty echo`.
/// This prevents the terminal from being left with echo disabled if the program
/// exits unexpectedly while reading hidden input.
struct EchoGuard;

impl EchoGuard {
    /// Creates a new guard and immediately disables terminal echo.
    fn new() -> Self {
        Command::new("stty").args(["-echo"]).status().ok();
        EchoGuard
    }
}

impl Drop for EchoGuard {
    /// Restores terminal echo. Called automatically on scope exit or panic unwind.
    fn drop(&mut self) {
        Command::new("stty").args(["echo"]).status().ok();
    }
}

/// Collects entry details from the user via interactive prompts.
///
/// Prompts for name, site, username, password (hidden input), and note.
/// Each field is trimmed of leading/trailing whitespace.
///
/// # Returns
/// A `Details` struct with all five fields populated from user input.
pub fn collect_details() -> Details {
    let name = prompt("Entry name: ");
    let site = prompt("Site: ");
    let uname = prompt("Username: ");
    let pword = prompt_hidden("Password: ");
    let note = prompt("Note: ");
    Details {
        name,
        site,
        uname,
        pword,
        note,
    }
}

/// Reads a line of visible (echoed) text from stdin.
///
/// # Arguments
/// * `msg` - The prompt message shown to the user
///
/// # Returns
/// The trimmed input string, or an empty string on read failure.
fn prompt(msg: &str) -> String {
    print!("{msg}");
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    input.trim().to_string()
}

/// Reads a line of hidden (no-echo) text from stdin.
///
/// Terminal echo is disabled via `stty -echo` while reading and restored
/// via `EchoGuard` on function exit or panic. The input is not visible
/// on screen, making it suitable for passwords.
///
/// # Arguments
/// * `msg` - The prompt message shown to the user
///
/// # Returns
/// The trimmed input string, or an empty string on read failure.
///
/// # Caveats
/// - Requires a Unix-like terminal with `stty` available.
/// - Does not protect against keyloggers or terminal recording.
pub fn prompt_hidden(msg: &str) -> String {
    let _guard = EchoGuard::new();
    print!("{msg}");
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    println!();
    input.trim().to_string()
}
