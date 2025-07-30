use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn initialization_test() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("webThing")?;

    cmd.env("DATABASE_FILE", "data.db");

    // cmd.arg("foobar").arg("test/file/doesnt/exist");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("panicked"));

    // cmd.kill();

    Ok(())
}
