use pasm_cli::client::auth::master;
use pasm_cli::client::entry::ops;
use pasm_cli::client::{cli::commands::CliCommand, curl::requests, response::display};
use pasm_core::utils::config;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd_args: Vec<String> = args.into_iter().skip(1).collect();

    // Extract --loadfile before passing to command parser
    let mut loadfile: Option<String> = None;
    let mut filtered: Vec<String> = Vec::new();
    let mut i = 0;
    while i < cmd_args.len() {
        if cmd_args[i] == "--loadfile" {
            i += 1;
            loadfile = Some(
                cmd_args
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| "--loadfile requires a path".to_string()),
            );
            i += 1;
        } else {
            filtered.push(cmd_args[i].clone());
            i += 1;
        }
    }

    if filtered.is_empty() && loadfile.is_none() {
        print!("{}", CliCommand::usage());
        return;
    }

    let (command, addr, conf) = match CliCommand::from_args(&filtered) {
        Ok(res) => res,
        Err(e) => {
            display::print_error(&format!("Error: {e}"));
            eprint!("\n{}", CliCommand::usage());
            std::process::exit(1);
        }
    };

    if let Some(a) = &addr {
        config::set_server_url(a);
    }
    if let Some(c) = &conf {
        config::set_config_path(c);
    }

    if matches!(
        command,
        CliCommand::Login | CliCommand::Logout | CliCommand::Help
    ) {
        let result = command.execute("", "");
        if result.starts_with("Error") {
            display::print_error(&result);
        } else {
            display::print_response(&result);
        }
        return;
    }

    let (api_key, encr_key) = match master::get_session_keys() {
        Some((ak, ek)) => (ak, ek),
        None => {
            display::print_error("not logged in. run `pasm_client login` first");
            std::process::exit(1);
        }
    };

    // Handle --loadfile restore before normal command flow
    if let Some(path) = loadfile {
        let result = ops::restore_from_file(&api_key, &path);
        if result.starts_with("Error") {
            display::print_error(&result);
        } else {
            display::print_response(&result);
        }
        return;
    }

    if let Err(msg) = requests::check_health() {
        display::print_error(&msg);
        std::process::exit(1);
    }

    let result = command.execute(&api_key, &encr_key);
    if result.starts_with("Error") {
        display::print_error(&result);
    } else {
        display::print_response(&result);
    }
}
