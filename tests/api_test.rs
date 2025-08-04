use hurl::runner;
use hurl::runner::{RunnerOptionsBuilder, Value, VariableSet};
use hurl::util::logger::LoggerOptionsBuilder;
use hurl_core::input::Input;

#[test]
fn api_test() -> Result<(), Box<dyn std::error::Error>> {
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_webThing"))
        .env("DATABASE_FILE", "data.db")
        .spawn()?;

    // Read hurl file
    let content = std::fs::read_to_string("./tests/api.hurl")?;

    // Set the baseurl variable
    let mut variables = VariableSet::new();
    variables.insert(
        "baseurl".to_string(),
        Value::String("http://localhost:8080/".to_string()),
    );

    // Run it
    let runner_opts = RunnerOptionsBuilder::new().follow_location(true).build();
    let logger_opts = LoggerOptionsBuilder::new().build();
    let result = runner::run(
        &content,
        Some(Input::new("api.hurl")).as_ref(),
        &runner_opts,
        &variables,
        &logger_opts,
    )?;

    child.kill()?;

    assert!(result.success);

    Ok(())
}
