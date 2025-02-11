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
    let overrides = overrides(config.exclude)?;

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

fn overrides(excludes: Vec<String>) -> Result<Override> {
    let mut ov = OverrideBuilder::new(".");
    ov.add("!.git/")?;
    for exclude in excludes {
        ov.add(&format!("!{}", exclude))?;
    }
    ov.build().context("Failed to build ignore overrides")
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

    fn setup_config(
        config_path: &PathBuf,
        answer_path: &Path,
        template_suffix: &str,
        excludes: Vec<String>,
    ) {
        let content = format!(
            r#"
            template_suffix = "{}"
            answer_file = "{}"
            exclude = {:?}
            "#,
            template_suffix,
            answer_path.display(),
            excludes,
        );
        println!("{}", content);
        write_file(config_path, &content);
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

        setup_config(&config_path, &answer_path, "tera", vec![]);
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

    #[test]
    fn test_has_template_suffix() {
        let path = Path::new("file.txt.tera");
        assert!(super::has_template_suffix(path, "tera"));
        assert!(!super::has_template_suffix(path, "txt"));

        let path = Path::new("file.tera");
        assert!(super::has_template_suffix(path, "tera"));

        let path = Path::new("file");
        assert!(!super::has_template_suffix(path, "tera"));

        let path = Path::new(".hidden.tera");
        assert!(super::has_template_suffix(path, "tera"));
    }

    #[test]
    fn test_ignore_git_directory() {
        let tmp_dir = create_temp_dir();

        let git_dir = tmp_dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();
        let git_template = git_dir.join("test.tera");
        create_template(&git_template, "content");

        let config_path = tmp_dir.path().join("config.toml");
        let answer_path = tmp_dir.path().join("answers.toml");
        setup_config(&config_path, &answer_path, "tera", vec![]);
        setup_answers(&answer_path, r#"key = "value""#);

        let args = RenderArgs {
            path: Some(tmp_dir.path().to_path_buf()),
        };
        let global_args = GlobalArgs {
            config_path: config_path.clone(),
        };

        render(args, global_args).expect("Render failed");

        let output_path = git_dir.join("test");
        assert!(!output_path.exists());
    }

    #[test]
    fn test_process_hidden_template() {
        let tmp_dir = create_temp_dir();

        let hidden_template = tmp_dir.path().join(".env.tera");
        create_template(&hidden_template, "key={{ key }}");

        let config_path = tmp_dir.path().join("config.toml");
        let answer_path = tmp_dir.path().join("answers.toml");
        setup_config(&config_path, &answer_path, "tera", vec![]);
        setup_answers(&answer_path, r#"key = "value""#);

        let args = RenderArgs {
            path: Some(tmp_dir.path().to_path_buf()),
        };
        let global_args = GlobalArgs { config_path };

        render(args, global_args).unwrap();

        let output_path = tmp_dir.path().join(".env");
        assert!(output_path.exists());
        assert_eq!(read_file_contents(&output_path), "key=value");
    }

    #[test]
    fn test_missing_answer_file() {
        let tmp_dir = create_temp_dir();

        let config_path = tmp_dir.path().join("config.toml");
        let answer_path = tmp_dir.path().join("nonexistent.toml");
        setup_config(&config_path, &answer_path, "tera", vec![]);

        let args = RenderArgs {
            path: Some(tmp_dir.path().to_path_buf()),
        };
        let global_args = GlobalArgs { config_path };

        let result = render(args, global_args);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_toml_in_answers() {
        let tmp_dir = create_temp_dir();

        let answer_path = tmp_dir.path().join("answers.toml");
        setup_answers(&answer_path, "invalid = toml here");

        let config_path = tmp_dir.path().join("config.toml");
        setup_config(&config_path, &answer_path, "tera", vec![]);

        let args = RenderArgs {
            path: Some(tmp_dir.path().to_path_buf()),
        };
        let global_args = GlobalArgs { config_path };

        let result = render(args, global_args);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_template_syntax() {
        let tmp_dir = create_temp_dir();

        let template_path = tmp_dir.path().join("invalid.tera");
        create_template(&template_path, "{{ invalid syntax }}");

        let config_path = tmp_dir.path().join("config.toml");
        let answer_path = tmp_dir.path().join("answers.toml");
        setup_config(&config_path, &answer_path, "tera", vec![]);
        setup_answers(&answer_path, r#"key = "value""#);

        let args = RenderArgs {
            path: Some(tmp_dir.path().to_path_buf()),
        };
        let global_args = GlobalArgs { config_path };

        let result = render(args, global_args);
        assert!(result.is_err());
    }

    #[test]
    fn test_exclude_directory() {
        let tmp_dir = create_temp_dir();

        // Create an excluded directory with a template file inside
        let excluded_dir = tmp_dir.path().join("excluded_dir");
        fs::create_dir(&excluded_dir).expect("Failed to create excluded directory");
        let template_in_excluded = excluded_dir.join("file.tera");
        create_template(&template_in_excluded, "key={{ key }}");

        let config_path = tmp_dir.path().join("config.toml");
        let answer_path = tmp_dir.path().join("answers.toml");
        // Pass the exclude pattern for the directory
        setup_config(
            &config_path,
            &answer_path,
            "tera",
            vec!["excluded_dir/".to_string()],
        );
        setup_answers(&answer_path, r#"key = "value""#);

        let args = RenderArgs {
            path: Some(tmp_dir.path().to_path_buf()),
        };
        let global_args = GlobalArgs { config_path };

        render(args, global_args).unwrap();

        let output_path = excluded_dir.join("file");
        // The output file should not exist since the directory is excluded
        assert!(!output_path.exists());
    }
}
