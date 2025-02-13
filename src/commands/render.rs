use anyhow::{Context, Result};
use colored::Colorize;

use crate::args::{GlobalArgs, RenderArgs};
use crate::config::Config;
use crate::context::read_context;
use crate::template::render_template;
use crate::walk::WalkOptions;

pub fn render(args: RenderArgs, global: GlobalArgs) -> Result<()> {
    let cfg = Config::from_file(&global.config_path)?;
    let ctx = read_context(&cfg.answer_file)?;

    WalkOptions::from_config(&cfg)
        .walk(&args.path)?
        .skip(1) // Skip the root directory
        .filter_map(|entry| entry.context("Failed to read directory entry").ok())
        .try_for_each(|entry| {
            println!(
                "{} {} -> {}",
                "Rendering".green().bold(),
                entry.path().display(),
                entry.path().with_extension("").display()
            );
            render_template(entry.path(), &entry.path().with_extension(""), &ctx)
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;

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
            path: tmp_path.to_path_buf(),
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
            path: tmp_path.to_path_buf(),
        };
        let global_args = GlobalArgs {
            config_path: config_path.clone(),
        };
        let result = render(args, global_args);
        assert!(result.is_err(), "Should error on missing answer file");

        // Test invalid TOML
        setup_answers(&answer_path, "invalid = toml here");
        let args = RenderArgs {
            path: tmp_path.to_path_buf(),
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
            path: tmp_path.to_path_buf(),
        };
        let global_args = GlobalArgs { config_path };
        let result = render(args, global_args);
        assert!(result.is_err(), "Should error on invalid template syntax");
    }
}
