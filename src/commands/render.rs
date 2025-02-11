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

    let root_path = args.path.as_deref().unwrap_or(Path::new("."));

    WalkBuilder::new(root_path)
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
    let template_content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read template: {}", path.display()))?;

    let rendered_output = Tera::one_off(&template_content, context, true)
        .with_context(|| format!("Failed to render template: {}", path.display()))?;

    let output_path = path.with_extension("");
    fs::write(&output_path, rendered_output)
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

    fn create_temp_dir() -> TempDir {
        TempDir::new().expect("Failed to create temp directory")
    }

    fn write_file(path: &PathBuf, contents: &str) {
        let mut file = File::create(path).expect("Failed to create file");
        file.write_all(contents.as_bytes())
            .expect("Failed to write file");
    }

    fn setup_config(config_path: &PathBuf, answer_path: &Path, template_suffix: &str) {
        write_file(
            config_path,
            &format!(
                r#"
                template_suffix = "{}"
                answer_file = "{}"
                "#,
                template_suffix,
                answer_path.display(),
            ),
        );
    }

    fn setup_answers(answer_path: &PathBuf, answers: &str) {
        write_file(answer_path, answers);
    }

    fn create_template(template_path: &PathBuf, content: &str) {
        write_file(template_path, content);
    }

    fn read_file_contents(file_path: &PathBuf) -> String {
        fs::read_to_string(file_path).expect("Failed to read file")
    }

    #[test]
    fn test_render_template_with_suffix() {
        let tmp_dir = create_temp_dir();

        let config_path = tmp_dir.path().join("config.toml");
        let answer_path = tmp_dir.path().join("answers.toml");
        let template1_path = tmp_dir.path().join("test1.txt.tera");
        let template2_path = tmp_dir.path().join("test2.txt");

        setup_config(&config_path, &answer_path, "tera");
        setup_answers(&answer_path, r#"key = "testvalue""#);
        create_template(&template1_path, "key={{ key }}");
        create_template(&template2_path, "key");

        let args = RenderArgs {
            path: Some(tmp_dir.path().to_path_buf()),
        };
        let global_args = GlobalArgs {
            config_path: config_path.clone(),
        };

        render(args, global_args).expect("Render function failed");

        assert_eq!(
            read_file_contents(&tmp_dir.path().join("test1.txt")),
            "key=testvalue"
        );
        assert_eq!(read_file_contents(&tmp_dir.path().join("test2.txt")), "key");
    }
}
