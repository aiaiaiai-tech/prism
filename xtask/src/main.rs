use std::{error::Error, fs, path::Path};

use prism_protocol::{RequestEnvelope, ResponseEnvelope};
use schemars::{JsonSchema, schema_for};

const REQUEST_SCHEMA: &str = "contracts/prism-execution.v1.request.schema.json";
const RESPONSE_SCHEMA: &str = "contracts/prism-execution.v1.response.schema.json";

fn main() -> Result<(), Box<dyn Error>> {
    let command = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "check".to_owned());
    let generated = [
        (REQUEST_SCHEMA, render_schema::<RequestEnvelope>()?),
        (RESPONSE_SCHEMA, render_schema::<ResponseEnvelope>()?),
    ];

    match command.as_str() {
        "write" => {
            for (path, content) in generated {
                fs::write(path, content)?;
                println!("wrote {path}");
            }
            Ok(())
        }
        "check" => {
            let mut stale = Vec::new();
            for (path, expected) in generated {
                match fs::read_to_string(path) {
                    Ok(actual) if actual == expected => {}
                    _ => stale.push(path),
                }
            }
            if stale.is_empty() {
                Ok(())
            } else {
                Err(format!(
                    "generated contracts are missing or stale: {}; run `cargo run -p xtask -- write`",
                    stale.join(", ")
                )
                .into())
            }
        }
        _ => Err(format!("unknown xtask command `{command}`; use `check` or `write`").into()),
    }
}

fn render_schema<T: JsonSchema>() -> Result<String, serde_json::Error> {
    let mut rendered = serde_json::to_string_pretty(&schema_for!(T))?;
    rendered.push('\n');
    Ok(rendered)
}

#[allow(dead_code)]
fn _assert_paths_are_relative() {
    debug_assert!(!Path::new(REQUEST_SCHEMA).is_absolute());
    debug_assert!(!Path::new(RESPONSE_SCHEMA).is_absolute());
}
