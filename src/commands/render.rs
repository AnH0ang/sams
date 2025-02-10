use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use ignore::overrides::{Override, OverrideBuilder};
use ignore::WalkBuilder;
use tera::{Context as TeraContext, Tera};
use toml::Table;

use crate::args::{GlobalArgs, RenderArgs};
use crate::config::read_config;

pub fn render(args: RenderArgs, global_args: GlobalArgs) -> Result<()> {
    let config = read_config(&global_args.config_path)?;
    let answers = read_answers(&config.answer_file)?;
    let context = TeraContext::from_serialize(answers)?;
    let overrides = default_overrides()?;

    let root = args.path.as_deref().unwrap_or_else(|| Path::new("."));

    WalkBuilder::new(root)
        .overrides(overrides)
        .standard_filters(true)
        .hidden(false)
        .build()
        .filter_map(|entry| entry.context("Failed to read directory entry").ok())
        .filter(|entry| entry.file_type().is_some_and(|ft| ft.is_file()))
        .filter(|entry| has_template_suffix(entry.path(), &config.template_suffix))
        .try_for_each(|entry| process_template(entry.path(), &context))?;

    Ok(())
}

fn read_answers<P: AsRef<Path>>(answer_file: P) -> Result<Table> {
    let path = answer_file.as_ref();

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read answer file: {}", path.display()))?;

    toml::from_str(&content)
        .with_context(|| format!("Failed to parse TOML from {}", path.display()))
}

fn has_template_suffix(path: &Path, suffix: &str) -> bool {
    path.extension().and_then(OsStr::to_str) == Some(suffix)
}

fn default_overrides() -> Result<Override> {
    OverrideBuilder::new(".")
        .add("!.git/")?
        .build()
        .context("Failed to build ignore overrides")
}

fn process_template(path: &Path, context: &TeraContext) -> Result<()> {
    let template_str = fs::read_to_string(path)
        .with_context(|| format!("Failed to read template: {}", path.display()))?;

    let rendered = Tera::one_off(&template_str, context, true)
        .with_context(|| format!("Failed to render template: {}", path.display()))?;

    let output_path = path.with_extension("");
    fs::write(&output_path, rendered)
        .with_context(|| format!("Failed to write output: {}", output_path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;

    use tempfile::TempDir;

    use super::*;

    fn tmpdir() -> TempDir {
        TempDir::new().unwrap()
    }

    fn wfile(path: PathBuf, contents: &str) {
        let mut file = File::create(path).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }

    fn setup_config(config_file: PathBuf, answer_file: PathBuf, template_suffix: &str) {
        wfile(
            config_file,
            &format!(
                r#"
                template_suffix = "{}"
                answer_file = "{}"
                "#,
                template_suffix,
                answer_file.to_str().unwrap(),
            ),
        );
    }

    fn setup_answers(answer_file: PathBuf, answers: &str) {
        wfile(answer_file, answers);
    }

    fn create_template(file: PathBuf, content: &str) {
        wfile(file, content);
    }

    fn read_file(file: PathBuf) -> String {
        fs::read_to_string(file).unwrap()
    }

    #[test]
    fn renders_template_with_suffix() {
        let td = tmpdir();

        setup_config(
            td.path().join("sams.toml"),
            td.path().join(".answers.toml"),
            "tera",
        );
        setup_answers(td.path().join(".answers.toml"), r#"key = "testvalue""#);
        create_template(td.path().join("test1.txt.tera"), "key={{ key }}");
        create_template(td.path().join("test2.txt"), "key");

        let args = super::RenderArgs {
            path: Some(td.path().to_path_buf()),
        };
        let global_args = super::GlobalArgs {
            config_path: td.path().join("sams.toml"),
        };

        render(args, global_args).unwrap();

        assert_eq!(read_file(td.path().join("test1.txt")), "key=testvalue");
        assert_eq!(read_file(td.path().join("test2.txt")), "key");
    }
}
