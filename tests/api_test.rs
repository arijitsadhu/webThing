use hurl::runner;
use hurl::runner::{RunnerOptionsBuilder, Value, VariableSet};
use hurl::util::logger::{LoggerOptions, LoggerOptionsBuilder, Verbosity};
use hurl_core::input::Input;

const HTTP_PORT: &str = "8080";
const BASEURL: &str = "http://localhost:8080";

#[test]
fn api_test() -> Result<(), Box<dyn std::error::Error>> {
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_webThing"))
        .env("DATABASE_FILE", "data.db")
        .env("HTTP_PORT", HTTP_PORT)
        .spawn()?;

    // Read hurl file
    let content = std::fs::read_to_string("./tests/api.hurl")?;

    // Set the baseurl variable
    let mut variables = VariableSet::new();
    variables.insert("baseurl".to_string(), Value::String(BASEURL.to_string()));

    // Run it
    let runner_opts = RunnerOptionsBuilder::new()
        .follow_location(true)
        .cookie_input_file(Some("session_cookie.txt".to_string()))
        .build();
    let logger_opts: LoggerOptions = LoggerOptionsBuilder::new()
        .verbosity(Some(Verbosity::VeryVerbose))
        .build();
    let result = runner::run(
        &content,
        Some(Input::new("api.hurl")).as_ref(),
        &runner_opts,
        &variables,
        &logger_opts,
    )?;

    child.kill()?;

    std::fs::remove_file("data.db")?;

    assert!(result.success);

    Ok(())
}
