use colored::Colorize;

/// Prints a server or command response to stdout with formatting.
///
/// - Empty responses are shown in yellow as `"(empty response)"`.
/// - Valid JSON responses are pretty-printed in cyan.
/// - Plain text responses are printed in cyan.
///
/// # Arguments
/// * `response` - The raw response string from a command or API call
pub fn print_response(response: &str) {
    if response.is_empty() {
        println!("{}", "(empty response)".yellow());
        return;
    }

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(response) {
        match serde_json::to_string_pretty(&json) {
            Ok(pretty) => println!("{}", pretty.cyan()),
            Err(_) => println!("{}", response.cyan()),
        }
    } else {
        println!("{}", response.cyan());
    }
}

/// Prints an error message to stderr in red.
///
/// # Arguments
/// * `msg` - The error message to display
pub fn print_error(msg: &str) {
    eprintln!("{}", msg.red());
}
