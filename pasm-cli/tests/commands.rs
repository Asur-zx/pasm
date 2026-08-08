use pasm_cli::client::cli::commands::CliCommand;

#[test]
fn from_args_login() {
    let args = vec!["login".to_string()];
    let (cmd, addr, conf) = CliCommand::from_args(&args).unwrap();
    assert!(matches!(cmd, CliCommand::Login));
    assert!(addr.is_none());
    assert!(conf.is_none());
}

#[test]
fn from_args_logout() {
    let args = vec!["logout".to_string()];
    let (cmd, _, _) = CliCommand::from_args(&args).unwrap();
    assert!(matches!(cmd, CliCommand::Logout));
}

#[test]
fn from_args_create() {
    let args = vec!["create".to_string()];
    let (cmd, _, _) = CliCommand::from_args(&args).unwrap();
    assert!(matches!(cmd, CliCommand::Create));
}

#[test]
fn from_args_list() {
    let args = vec!["list".to_string()];
    let (cmd, _, _) = CliCommand::from_args(&args).unwrap();
    assert!(matches!(cmd, CliCommand::List));
}

#[test]
fn from_args_help() {
    for arg in &["help", "-h", "--help"] {
        let args = vec![arg.to_string()];
        let (cmd, _, _) = CliCommand::from_args(&args).unwrap();
        assert!(matches!(cmd, CliCommand::Help), "failed for {arg}");
    }
}

#[test]
fn from_args_find_with_name() {
    let args = vec!["find".to_string(), "github".to_string()];
    let (cmd, _, _) = CliCommand::from_args(&args).unwrap();
    assert!(matches!(cmd, CliCommand::Find { .. }));
    if let CliCommand::Find { name } = cmd {
        assert_eq!(name, "github");
    }
}

#[test]
fn from_args_find_missing_name() {
    let args = vec!["find".to_string()];
    let err = CliCommand::from_args(&args).unwrap_err();
    assert_eq!(err, "find requires a name");
}

#[test]
fn from_args_delete_with_name() {
    let args = vec!["delete".to_string(), "test_entry".to_string()];
    let (cmd, _, _) = CliCommand::from_args(&args).unwrap();
    assert!(matches!(cmd, CliCommand::Delete { .. }));
    if let CliCommand::Delete { name } = cmd {
        assert_eq!(name, "test_entry");
    }
}

#[test]
fn from_args_delete_missing_name() {
    let args = vec!["delete".to_string()];
    let err = CliCommand::from_args(&args).unwrap_err();
    assert_eq!(err, "delete requires a name");
}

#[test]
fn from_args_amend() {
    let args = vec!["amend".to_string()];
    let (cmd, _, _) = CliCommand::from_args(&args).unwrap();
    assert!(matches!(cmd, CliCommand::Amend));
}

#[test]
fn from_args_register() {
    let args = vec!["register".to_string()];
    let (cmd, _, _) = CliCommand::from_args(&args).unwrap();
    assert!(matches!(cmd, CliCommand::Register));
}

#[test]
fn from_args_update_auth() {
    let args = vec!["update-auth".to_string(), "new_key_value".to_string()];
    let (cmd, _, _) = CliCommand::from_args(&args).unwrap();
    assert!(matches!(cmd, CliCommand::UpdateAuth { .. }));
    if let CliCommand::UpdateAuth { new_key } = cmd {
        assert_eq!(new_key, "new_key_value");
    }
}

#[test]
fn from_args_update_auth_missing_key() {
    let args = vec!["update-auth".to_string()];
    let err = CliCommand::from_args(&args).unwrap_err();
    assert_eq!(err, "update-auth requires a new key");
}

#[test]
fn from_args_remove_auth() {
    let args = vec!["remove-auth".to_string()];
    let (cmd, _, _) = CliCommand::from_args(&args).unwrap();
    assert!(matches!(cmd, CliCommand::RemoveAuth));
}

#[test]
fn from_args_list_users() {
    let args = vec!["list-users".to_string()];
    let (cmd, _, _) = CliCommand::from_args(&args).unwrap();
    assert!(matches!(cmd, CliCommand::ListUsers));
}

#[test]
fn from_args_backup() {
    let args = vec!["backup".to_string()];
    let (cmd, _, _) = CliCommand::from_args(&args).unwrap();
    assert!(matches!(cmd, CliCommand::Backup));
}

#[test]
fn from_args_set_admin() {
    let args = vec!["set-admin".to_string(), "target_key".to_string()];
    let (cmd, _, _) = CliCommand::from_args(&args).unwrap();
    assert!(matches!(cmd, CliCommand::SetAdmin { .. }));
    if let CliCommand::SetAdmin { key } = cmd {
        assert_eq!(key, "target_key");
    }
}

#[test]
fn from_args_set_admin_missing_key() {
    let args = vec!["set-admin".to_string()];
    let err = CliCommand::from_args(&args).unwrap_err();
    assert_eq!(err, "set-admin requires an auth key");
}

#[test]
fn from_args_unknown_command() {
    let args = vec!["nonexistent".to_string()];
    let err = CliCommand::from_args(&args).unwrap_err();
    assert_eq!(err, "unknown command: nonexistent");
}

#[test]
fn from_args_missing_command() {
    let args: Vec<String> = vec![];
    let err = CliCommand::from_args(&args).unwrap_err();
    assert_eq!(err, "missing command");
}

#[test]
fn from_args_with_global_addr_flag() {
    let args = vec![
        "--addr".to_string(),
        "10.0.0.1:4000".to_string(),
        "list".to_string(),
    ];
    let (cmd, addr, conf) = CliCommand::from_args(&args).unwrap();
    assert!(matches!(cmd, CliCommand::List));
    assert_eq!(addr, Some("10.0.0.1:4000".to_string()));
    assert!(conf.is_none());
}

#[test]
fn from_args_with_global_config_flag() {
    let args = vec![
        "--config".to_string(),
        "/custom/path.toml".to_string(),
        "login".to_string(),
    ];
    let (cmd, addr, conf) = CliCommand::from_args(&args).unwrap();
    assert!(matches!(cmd, CliCommand::Login));
    assert!(addr.is_none());
    assert_eq!(conf, Some("/custom/path.toml".to_string()));
}

#[test]
fn from_args_with_both_global_flags() {
    let args = vec![
        "--addr".to_string(),
        "0.0.0.0:3000".to_string(),
        "--config".to_string(),
        "my.toml".to_string(),
        "find".to_string(),
        "entry1".to_string(),
    ];
    let (cmd, addr, conf) = CliCommand::from_args(&args).unwrap();
    assert!(matches!(cmd, CliCommand::Find { .. }));
    assert_eq!(addr, Some("0.0.0.0:3000".to_string()));
    assert_eq!(conf, Some("my.toml".to_string()));
    if let CliCommand::Find { name } = cmd {
        assert_eq!(name, "entry1");
    }
}

#[test]
fn from_args_addr_flag_without_value() {
    let args = vec!["--addr".to_string()];
    let err = CliCommand::from_args(&args).unwrap_err();
    assert_eq!(err, "--addr requires a value");
}

#[test]
fn from_args_config_flag_without_value() {
    let args = vec!["--config".to_string()];
    let err = CliCommand::from_args(&args).unwrap_err();
    assert_eq!(err, "--config requires a value");
}

#[test]
fn usage_contains_all_commands() {
    let usage = CliCommand::usage();
    assert!(usage.contains("login"));
    assert!(usage.contains("logout"));
    assert!(usage.contains("create"));
    assert!(usage.contains("find"));
    assert!(usage.contains("list"));
    assert!(usage.contains("delete"));
    assert!(usage.contains("amend"));
    assert!(usage.contains("backup"));
    assert!(usage.contains("set-admin"));
    assert!(usage.contains("register"));
    assert!(usage.contains("update-auth"));
    assert!(usage.contains("remove-auth"));
    assert!(usage.contains("list-users"));
    assert!(usage.contains("--config"));
    assert!(usage.contains("--addr"));
}
