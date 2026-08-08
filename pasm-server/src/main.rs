use pasm_core::utils::config;
use pasm_server::server;

/// pasm REST API server.
///
/// ```text
/// pasm_server [options]
///
/// Options:
///   --config <path>     Config file path (default: ~/.config/pasm/config.toml)
///   --addr <host:port>  Bind address    (default: 0.0.0.0:3000)
/// ```
#[tokio::main]
pub async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--addr" => {
                i += 1;
                if i < args.len() {
                    std::env::set_var("PASM_SERVER_ADDR", &args[i]);
                }
            }
            "--config" => {
                i += 1;
                if i < args.len() {
                    config::set_config_path(&args[i]);
                }
            }
            _ => {}
        }
        i += 1;
    }

    server::run().await;
}
