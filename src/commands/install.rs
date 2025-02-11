use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

use crate::args::{GlobalArgs, InstallArgs};
use crate::config::{read_config, Task};

pub fn install(_args: InstallArgs, global_args: GlobalArgs) -> Result<()> {
    let cfg = read_config(&global_args.config_path)?;

    let pb = create_progress_bar(cfg.tasks.len() as u64);

    for (i, task) in cfg.tasks.iter().enumerate() {
        let task_name = task
            .name
            .as_deref()
            .unwrap_or_else(|| task.script.to_str().unwrap());
        pb.set_message(format!("{:>8} {}", "Running".yellow().bold(), task_name));

        execute_task(task).with_context(|| format!("Failed to execute task: {}", task_name))?;

        pb.inc(1);
        pb.println(format!(
            "{} {:>8} {} ({}/{})",
            "✓".green(),
            "Finished".green().bold(),
            task_name,
            i + 1,
            cfg.tasks.len()
        ));
    }

    pb.finish_and_clear();
    println!("{}", "\nAll tasks completed successfully 🎉".green().bold());
    Ok(())
}

fn create_progress_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg} ({pos}/{len})")
            .unwrap()
            .progress_chars("#>-")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_prefix("Running".yellow().bold().to_string());
    pb
}

fn execute_task(task: &Task) -> Result<()> {
    let output = Command::new(&task.shell)
        .arg(task.script.to_str().context("Invalid script path")?)
        .current_dir(&task.workdir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output()
        .context("Failed to execute task")?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Task failed: {}", stderr)
    }
}
