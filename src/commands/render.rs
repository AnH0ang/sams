use std::ffi::OsStr;
use std::path::Path;

use anyhow::{Context, Result};
use ignore::overrides::{Override, OverrideBuilder};
use ignore::WalkBuilder;

use crate::args::{GlobalArgs, RenderArgs};
use crate::config::read_config;
use crate::context::read_context;
use crate::template::render_template;

pub fn render(args: RenderArgs, global: GlobalArgs) -> Result<()> {
    let cfg = read_config(&global.config_path)?;
    let ctx = read_context(cfg.answer_file)?;

    let overrides = build_overrides(cfg.exclude)?;
    let root = args.path.as_deref().unwrap_or_else(|| Path::new("."));

    WalkBuilder::new(root)
        .overrides(overrides)
        .standard_filters(true)
        .hidden(false)
        .build()
        .filter_map(|entry| entry.context("Failed to read directory entry").ok())
        .filter(|entry| entry.file_type().is_some_and(|ft| ft.is_file()))
        .filter(|entry| has_template_suffix(entry.path(), &cfg.template_suffix))
        .try_for_each(|entry| {
            let output_path = entry.path().with_extension("");
            render_template(entry.path(), &output_path, &ctx)
        })?;

    Ok(())
}

fn has_template_suffix(path: &Path, suffix: &str) -> bool {
    path.extension().and_then(OsStr::to_str) == Some(suffix)
}

fn build_overrides(excludes: Vec<String>) -> Result<Override> {
    let mut builder = OverrideBuilder::new(".");
    builder.add("!.git/")?;
    for exclude in excludes {
        builder.add(&format!("!{}", exclude))?;
    }
    builder.build().context("Failed to build ignore overrides")
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::Write;

    use tempfile::TempDir;

    use super::*;

    fn create_temp_dir() -> TempDir {
        TempDir::new().expect("Failed to create temp directory")
    }

    fn write_file(path: &Path, contents: &str) {
        let mut file = File::create(path).expect("Failed to create file");
        file.write_all(contents.as_bytes())
            .expect("Failed to write file");
    }

    fn setup_config(
        config_path: &Path,
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
        write_file(config_path, &content);
    }

    fn setup_answers(answer_path: &Path, answers: &str) {
        write_file(answer_path, answers);
    }

    fn create_template(template_path: &Path, content: &str) {
        write_file(template_path, content);
    }

    fn read_file_contents(file_path: &Path) -> String {
        fs::read_to_string(file_path).expect("Failed to read file")
    }

    #[test]
    fn test_template_rendering_and_exclusion() {
        let tmp_dir = create_temp_dir();
        let tmp_path = tmp_dir.path();

        // Setup paths
        let config_path = tmp_path.join("config.toml");
        let answer_path = tmp_path.join("answers.toml");
        let template1_path = tmp_path.join("test1.txt.tera");
        let template2_path = tmp_path.join("test2.txt");
        let excluded_dir = tmp_path.join("excluded_dir");
        fs::create_dir(&excluded_dir).expect("Failed to create excluded directory");
        let template_in_excluded = excluded_dir.join("file.tera");

        // Setup config and answers
        setup_config(
            &config_path,
            &answer_path,
            "tera",
            vec!["excluded_dir/".to_string()],
        );
        setup_answers(&answer_path, r#"key = "testvalue""#);

        // Create templates
        create_template(&template1_path, "key={{ key }}");
        create_template(&template2_path, "key");
        create_template(&template_in_excluded, "key={{ key }}");

        // Run render function
        let args = RenderArgs {
            path: Some(tmp_path.to_path_buf()),
        };
        let global_args = GlobalArgs {
            config_path: config_path.clone(),
        };
        render(args, global_args).expect("Render function failed");

        // Assertions
        assert_eq!(
            read_file_contents(&tmp_path.join("test1.txt")),
            "key=testvalue"
        );
        assert_eq!(read_file_contents(&tmp_path.join("test2.txt")), "key");
        assert!(
            !excluded_dir.join("file").exists(),
            "Excluded directory should not be processed"
        );
    }

    #[test]
    fn test_error_handling() {
        let tmp_dir = create_temp_dir();
        let tmp_path = tmp_dir.path();

        // Setup paths
        let config_path = tmp_path.join("config.toml");
        let answer_path = tmp_path.join("answers.toml");
        let invalid_template_path = tmp_path.join("invalid.tera");

        // Test missing answer file
        setup_config(&config_path, &answer_path, "tera", vec![]);
        let args = RenderArgs {
            path: Some(tmp_path.to_path_buf()),
        };
        let global_args = GlobalArgs {
            config_path: config_path.clone(),
        };
        let result = render(args, global_args);
        assert!(result.is_err(), "Should error on missing answer file");

        // Test invalid TOML
        setup_answers(&answer_path, "invalid = toml here");
        let args = RenderArgs {
            path: Some(tmp_path.to_path_buf()),
        };
        let global_args = GlobalArgs {
            config_path: config_path.clone(),
        };
        let result = render(args, global_args);
        assert!(result.is_err(), "Should error on invalid TOML");

        // Test invalid template syntax
        setup_answers(&answer_path, r#"key = "value""#);
        create_template(&invalid_template_path, "{{ invalid syntax }}");
        let args = RenderArgs {
            path: Some(tmp_path.to_path_buf()),
        };
        let global_args = GlobalArgs { config_path };
        let result = render(args, global_args);
        assert!(result.is_err(), "Should error on invalid template syntax");
    }
}
