// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: Apache-2.0

use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};

use prism_protocol::{RequestEnvelope, ResponseEnvelope};
use schemars::{JsonSchema, schema_for};

const REQUEST_SCHEMA: &str = "contracts/prism-execution.v1.request.schema.json";
const RESPONSE_SCHEMA: &str = "contracts/prism-execution.v1.response.schema.json";
const COPYRIGHT: &str = "© 2026 aiaiaiai · aiaiaiai.org";
const SPDX: &str = "SPDX-License-Identifier: Apache-2.0";
const MARKDOWN_NOTICE: &str = "<!-- © 2026 aiaiaiai · aiaiaiai.org -->";

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
            let mut failures = Vec::new();
            for (path, expected) in generated {
                match fs::read_to_string(path) {
                    Ok(actual) if actual == expected => {}
                    _ => failures.push(format!(
                        "generated contract is missing or stale: {path}; run `cargo run -p xtask -- write`"
                    )),
                }
            }
            failures.extend(copyright_failures(Path::new("."))?);
            finish_check(failures)
        }
        "check-copyright" => finish_check(copyright_failures(Path::new("."))?),
        _ => Err(format!(
            "unknown xtask command `{command}`; use `check`, `check-copyright`, or `write`"
        )
        .into()),
    }
}

fn finish_check(failures: Vec<String>) -> Result<(), Box<dyn Error>> {
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n").into())
    }
}

fn copyright_failures(root: &Path) -> Result<Vec<String>, io::Error> {
    let mut files = Vec::new();
    collect_authored_files(root, &mut files)?;
    files.sort();

    let mut failures = Vec::new();
    for path in files {
        let content = fs::read_to_string(&path)?;
        let relative = path.strip_prefix(root).unwrap_or(&path).display();
        match path.extension().and_then(|extension| extension.to_str()) {
            Some("rs") => validate_header(
                &content,
                &format!("// {COPYRIGHT}\n// {SPDX}"),
                &relative.to_string(),
                &mut failures,
            ),
            Some("toml" | "yaml" | "yml") => validate_header(
                &content,
                &format!("# {COPYRIGHT}\n# {SPDX}"),
                &relative.to_string(),
                &mut failures,
            ),
            Some("md") if !content.trim_end().ends_with(MARKDOWN_NOTICE) => {
                failures.push(format!(
                    "{relative}: missing canonical copyright footer `{MARKDOWN_NOTICE}`"
                ));
            }
            _ => {}
        }
    }

    validate_repository_notices(root, &mut failures)?;
    Ok(failures)
}

fn collect_authored_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), io::Error> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            let name = entry.file_name();
            if name != ".git" && name != "target" && name != "contracts" {
                collect_authored_files(&path, files)?;
            }
        } else if file_type.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn validate_header(content: &str, expected: &str, path: &str, failures: &mut Vec<String>) {
    if !content.starts_with(expected) {
        failures.push(format!(
            "{path}: expected canonical copyright and Apache-2.0 SPDX header"
        ));
    }
}

fn validate_repository_notices(root: &Path, failures: &mut Vec<String>) -> Result<(), io::Error> {
    let notice = fs::read_to_string(root.join("NOTICE"))?;
    if !notice.lines().any(|line| line == COPYRIGHT) {
        failures.push("NOTICE: missing canonical aiaiaiai signature".to_owned());
    }
    if !notice.contains("Apache-2.0") {
        failures.push("NOTICE: missing repository license identifier Apache-2.0".to_owned());
    }

    let manifest = fs::read_to_string(root.join("Cargo.toml"))?;
    if !manifest.contains("license = \"Apache-2.0\"") {
        failures.push("Cargo.toml: workspace license must remain Apache-2.0".to_owned());
    }
    Ok(())
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
