use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

use crate::args::{GlobalArgs, InstallArgs};
use crate::config::{Config, Task};

pub fn install(args: InstallArgs, global: &GlobalArgs) -> Result<()> {
    let cfg = Config::from_file(&global.config_path)?;
    let pb = progress_bar(cfg.tasks.len() as u64);

    for (idx, task) in cfg.tasks.iter().enumerate() {
        let name = task
            .name
            .as_deref()
            .unwrap_or_else(|| task.script.to_str().unwrap());
        pb.set_prefix(format!("{:>8} {}", "Running".yellow().bold(), name.bold()));

        run_task(task, &pb, &args.path)
            .with_context(|| format!("Failed to execute task: {}", name))?;

        pb.set_message("");
        pb.inc(1);
        pb.println(format!(
            "{} {:>8} {} ({}/{})",
            "✓".green().bold(),
            "Finished".green().bold(),
            name.bold(),
            idx + 1,
            cfg.tasks.len()
        ));
    }
    pb.finish_and_clear();
    Ok(())
}

fn progress_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len).with_style(
        ProgressStyle::with_template("{spinner:.cyan.bold} {prefix} ({pos}/{len}) {msg}")
            .unwrap()
            .progress_chars("#>-")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.enable_steady_tick(Duration::from_millis(120));
    pb
}

fn run_task(task: &Task, pb: &ProgressBar, root: &Path) -> Result<()> {
    let mut cmd = Command::new(&task.shell)
        .arg(task.script.as_os_str())
        .current_dir(root.join(&task.workdir))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to start command")?;

    if let Some(stdout) = cmd.stdout.take() {
        BufReader::new(stdout)
            .lines()
            .map_while(Result::ok)
            .for_each(|line| {
                pb.set_message(line);
                pb.tick();
            });
    }

    let status = cmd.wait().context("Failed to wait for command")?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Command failed with status: {}", status))
    }
}
