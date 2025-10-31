#[tokio::main]
async fn main() {
  match pact_mock_server_cli::handle_command_args().await {
    Ok(_) => (),
    Err(err) => std::process::exit(err)
  }
}
