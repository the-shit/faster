//! Faster - Voice-driven deterministic intent processor for Claude Code

mod audio;
mod bridge;
mod config;
mod executor;
mod intent;
mod knowledge;
mod queue;

use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;

use config::Config;
use executor::ClaudeExecutor;
use queue::{TaskQueue, TaskStatus};
use audio::{MacOSSTT, MacOSTTS};
use intent::IntentProcessor;

#[derive(Parser)]
#[command(name = "faster")]
#[command(about = "Voice-driven deterministic intent processor for Claude Code", long_about = None)]
#[command(version)]
struct Cli {
    /// Quick command to execute (e.g., "run tests")
    #[arg(value_name = "COMMAND")]
    quick_command: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,

    /// Use specific Claude model
    #[arg(short, long)]
    model: Option<String>,

    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Start voice mode (default)
    Voice,

    /// Start background daemon
    Daemon,

    /// Show task queue status
    Status {
        /// Show all tasks (including completed)
        #[arg(short, long)]
        all: bool,
    },

    /// Cancel a task
    Cancel {
        /// Task ID to cancel
        task_id: String,
    },

    /// Clear completed tasks
    Clear,

    /// Test installation and components
    Test,

    /// Show or edit configuration
    Config {
        /// Show current configuration
        #[arg(short, long)]
        show: bool,

        /// Edit configuration file
        #[arg(short, long)]
        edit: bool,
    },

    /// Manage knowledge system
    Knowledge {
        #[command(subcommand)]
        action: KnowledgeCommands,
    },

    /// Setup wizard
    Setup,
}

#[derive(Subcommand)]
enum KnowledgeCommands {
    /// Show speech patterns
    Patterns,

    /// Show current goals
    Goals,

    /// Show current context
    Context,

    /// Show recent decisions
    Decisions,

    /// Clear all knowledge (dangerous!)
    Clear,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("faster=info".parse()?),
        )
        .init();

    let cli = Cli::parse();

    // Load config
    let config = load_or_create_config()?;

    // Handle quick command first (e.g., faster "run tests")
    if let Some(cmd) = cli.quick_command {
        queue_command(&cmd, &config, cli.model).await?;
        return Ok(());
    }

    match cli.command {
        Some(Commands::Daemon) => {
            run_daemon(&config).await?;
        }
        Some(Commands::Status { all }) => {
            show_status(&config, all).await?;
        }
        Some(Commands::Cancel { task_id }) => {
            cancel_task(&config, &task_id).await?;
        }
        Some(Commands::Clear) => {
            clear_completed(&config).await?;
        }
        Some(Commands::Voice) | None => {
            // Default: start voice mode
            voice_mode(config, cli.debug).await?;
        }
        Some(Commands::Test) => {
            test_installation()?;
        }
        Some(Commands::Config { show, edit }) => {
            handle_config_command(show, edit)?;
        }
        Some(Commands::Knowledge { action }) => {
            handle_knowledge_command(action)?;
        }
        Some(Commands::Setup) => {
            setup_wizard()?;
        }
    }

    Ok(())
}

async fn voice_mode(config: Config, debug: bool) -> anyhow::Result<()> {
    println!("{}", "🎤 Faster - Voice Mode".bright_green().bold());
    println!();
    println!("{}",  "Press Ctrl+C to exit".dimmed());
    println!();

    if debug {
        println!("{}", "[DEBUG MODE ENABLED]".yellow());
    }

    // Initialize STT and TTS
    let stt = MacOSSTT::new(&config.stt.language);
    let tts = MacOSTTS::new(&config.tts.voice, config.tts.rate);

    // Initialize intent processor
    let processor = IntentProcessor::new(config.intent.confidence_threshold);

    // Check availability
    if !MacOSSTT::is_available() {
        eprintln!("{}", "✗ Speech-to-text not available".red());
        return Ok(());
    }

    if !MacOSTTS::is_available() {
        eprintln!("{}", "✗ Text-to-speech not available".red());
        return Ok(());
    }

    println!("{}", "✓ Voice mode ready".green());
    println!();

    // Voice loop
    loop {
        println!("{}", "Press Enter to speak, or Ctrl+C to exit".dimmed());

        // Wait for Enter
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        // Transcribe
        match stt.transcribe() {
            Ok(transcript) => {
                println!();
                println!("{} {}", "📝 You said:".blue(), transcript.bright_white());

                // Process intent
                match processor.process(&transcript) {
                    Ok(command) => {
                        if debug {
                            println!("{} {:?}", "🎯 Intent:".cyan(), command.intent);
                            println!("{} {}", "📋 Directive:".cyan(), command.directive);
                            println!("{} {:.0}%", "🎲 Confidence:".cyan(), command.confidence * 100.0);
                        }

                        // Queue the processed command
                        if let Err(e) = queue_command(&command.directive, &config, None).await {
                            eprintln!("{} Failed to queue: {}", "✗".red(), e);
                            continue;
                        }

                        // Speak confirmation with intent
                        let response = format!("{:?}", command.intent);
                        tts.speak_async(&response)?;
                    }
                    Err(e) => {
                        eprintln!("{} Failed to process intent: {}", "✗".red(), e);
                        continue;
                    }
                }
            }
            Err(e) => {
                if debug {
                    eprintln!("{} {}", "✗ STT error:".red(), e);
                }
                continue;
            }
        }

        println!();
    }
}

fn test_installation() -> anyhow::Result<()> {
    println!("{}", "Testing Faster installation...".bright_cyan());
    println!();

    // Check Rust version
    print!("Rust compiler: ");
    match std::process::Command::new("rustc").arg("--version").output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("{} {}", "✓".green(), version.trim());
        }
        Err(_) => println!("{} Not found", "✗".red()),
    }

    // Check Claude CLI
    print!("Claude Code CLI: ");
    match std::process::Command::new("claude").arg("--version").output() {
        Ok(_) => println!("{} Installed", "✓".green()),
        Err(_) => {
            println!("{} Not found", "✗".red());
            println!("  Install from: https://claude.ai/code");
        }
    }

    // Check config file
    print!("Configuration: ");
    if Config::path().exists() {
        println!("{} {}", "✓".green(), Config::path().display());
    } else {
        println!("{} Not found (will create on first run)", "⚠".yellow());
    }

    // Check knowledge DB
    let config = load_or_create_config()?;
    print!("Knowledge database: ");
    if config.knowledge.local_db.exists() {
        println!("{} {}", "✓".green(), config.knowledge.local_db.display());
    } else {
        println!("{} Not yet initialized", "⚠".yellow());
    }

    // TODO: Check local AI model
    println!();
    println!("{}", "⚠️  Additional components not yet implemented:".yellow());
    println!("  • Local AI model (Llama 3.2)");
    println!("  • STT/TTS providers");
    println!("  • Knowledge system");

    Ok(())
}

fn handle_config_command(show: bool, edit: bool) -> anyhow::Result<()> {
    let config_path = Config::path();

    if show {
        if config_path.exists() {
            let config = Config::load(&config_path)?;
            let toml = toml::to_string_pretty(&config)?;
            println!("{}", toml);
        } else {
            println!("{} Config file not found", "✗".red());
            println!("Run {} to create", "faster setup".cyan());
        }
    } else if edit {
        // Open in default editor
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
        std::process::Command::new(editor)
            .arg(&config_path)
            .status()?;
    } else {
        println!("Config file: {}", config_path.display());
        println!();
        println!("Use:");
        println!("  {} to view configuration", "faster config --show".cyan());
        println!("  {} to edit configuration", "faster config --edit".cyan());
    }

    Ok(())
}

fn handle_knowledge_command(action: KnowledgeCommands) -> anyhow::Result<()> {
    match action {
        KnowledgeCommands::Patterns => {
            println!("{}", "Speech Patterns".bright_cyan());
            println!("{}", "⚠️  Not yet implemented".yellow());
        }
        KnowledgeCommands::Goals => {
            println!("{}", "Current Goals".bright_cyan());
            println!("{}", "⚠️  Not yet implemented".yellow());
        }
        KnowledgeCommands::Context => {
            println!("{}", "Current Context".bright_cyan());
            println!("{}", "⚠️  Not yet implemented".yellow());
        }
        KnowledgeCommands::Decisions => {
            println!("{}", "Recent Decisions".bright_cyan());
            println!("{}", "⚠️  Not yet implemented".yellow());
        }
        KnowledgeCommands::Clear => {
            println!("{}", "⚠️  This will delete all knowledge!".red().bold());
            println!("Type 'yes' to confirm:");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if input.trim() == "yes" {
                println!("{}", "Knowledge cleared".yellow());
                // TODO: Implement clear
            } else {
                println!("Cancelled");
            }
        }
    }

    Ok(())
}

fn setup_wizard() -> anyhow::Result<()> {
    println!("{}", "🚀 Faster Setup Wizard".bright_green().bold());
    println!();

    let config_path = Config::path();
    let config_dir = config_path.parent().unwrap();

    // Create config directory
    std::fs::create_dir_all(config_dir)?;

    // Create default config
    let config = Config::default();
    config.save(&config_path)?;

    println!("{} Created config file: {}", "✓".green(), config_path.display());
    println!();
    println!("Default configuration:");
    println!("  • STT: macOS native");
    println!("  • TTS: macOS native (Samantha voice)");
    println!("  • Model: Llama 3.2 (3B)");
    println!("  • Confirmation: Smart mode");
    println!();
    println!("Edit configuration: {}", "faster config --edit".cyan());
    println!("Test installation: {}", "faster --test".cyan());

    Ok(())
}

async fn queue_command(
    command: &str,
    config: &Config,
    model_override: Option<String>,
) -> anyhow::Result<()> {
    // Ensure database directory exists
    if let Some(parent) = config.knowledge.local_db.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let queue = TaskQueue::new(&config.knowledge.local_db.to_string_lossy()).await?;

    let model = model_override.or_else(|| {
        if !config.claude.model.is_empty() {
            Some(config.claude.model.clone())
        } else {
            None
        }
    });

    let task_id = queue.enqueue(command, model).await?;

    println!("{} Queued [{}]", "✓".green(), task_id.bright_cyan());
    println!();
    println!("Run {} to see queue status", "faster status".cyan());
    println!("Run {} to process queue", "faster daemon".cyan());

    Ok(())
}

async fn run_daemon(config: &Config) -> anyhow::Result<()> {
    println!("{}", "🚀 Starting daemon...".bright_green());

    let queue = TaskQueue::new(&config.knowledge.local_db.to_string_lossy()).await?;

    loop {
        // Get next task
        if let Some(task) = queue.dequeue().await? {
            println!("{} [{}] {}", "→".blue(), task.id.bright_cyan(), task.command);

            // Mark as running
            queue.update_status(&task.id, TaskStatus::Running).await?;

            // Create executor
            let mut executor = ClaudeExecutor::new(&config.claude.cli_path);
            if let Some(model) = &task.model {
                executor = executor.with_model(model);
            }

            // Execute
            match executor.execute(&task.command) {
                Ok(_) => {
                    queue.update_status(&task.id, TaskStatus::Completed).await?;
                    println!("{} [{}] Completed", "✓".green(), task.id.bright_cyan());
                }
                Err(e) => {
                    queue.fail(&task.id, &e.to_string()).await?;
                    eprintln!("{} [{}] Failed: {}", "✗".red(), task.id.bright_cyan(), e);
                }
            }

            println!();
        } else {
            // No tasks, wait a bit
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}

async fn show_status(config: &Config, show_all: bool) -> anyhow::Result<()> {
    let queue = TaskQueue::new(&config.knowledge.local_db.to_string_lossy()).await?;
    let tasks = queue.list().await?;

    if tasks.is_empty() {
        println!("{}", "No tasks in queue".dimmed());
        return Ok(());
    }

    println!("{}", "Task Queue".bright_cyan().bold());
    println!();

    for task in tasks {
        // Skip completed if not showing all
        if !show_all && (task.status == TaskStatus::Completed || task.status == TaskStatus::Cancelled) {
            continue;
        }

        let status_icon = match task.status {
            TaskStatus::Queued => "⏳",
            TaskStatus::Running => "→",
            TaskStatus::Completed => "✓",
            TaskStatus::Failed => "✗",
            TaskStatus::Cancelled => "⊘",
        };

        let status_color = match task.status {
            TaskStatus::Queued => task.status.as_str().yellow(),
            TaskStatus::Running => task.status.as_str().blue(),
            TaskStatus::Completed => task.status.as_str().green(),
            TaskStatus::Failed => task.status.as_str().red(),
            TaskStatus::Cancelled => task.status.as_str().dimmed(),
        };

        println!("{} [{}] {} {}",
            status_icon,
            task.id.bright_cyan(),
            status_color,
            task.command.dimmed()
        );

        if let Some(error) = task.error {
            println!("    {}: {}", "Error".red(), error);
        }
    }

    Ok(())
}

async fn cancel_task(config: &Config, task_id: &str) -> anyhow::Result<()> {
    let queue = TaskQueue::new(&config.knowledge.local_db.to_string_lossy()).await?;

    if let Some(task) = queue.get(task_id).await? {
        if task.status == TaskStatus::Running {
            println!("{} Cannot cancel running task", "✗".red());
            println!("  Kill the daemon to stop it");
            return Ok(());
        }

        queue.update_status(task_id, TaskStatus::Cancelled).await?;
        println!("{} Cancelled [{}]", "✓".green(), task_id.bright_cyan());
    } else {
        println!("{} Task not found: {}", "✗".red(), task_id);
    }

    Ok(())
}

async fn clear_completed(config: &Config) -> anyhow::Result<()> {
    let queue = TaskQueue::new(&config.knowledge.local_db.to_string_lossy()).await?;
    let count = queue.clear_completed().await?;

    println!("{} Cleared {} completed task(s)", "✓".green(), count);

    Ok(())
}

fn load_or_create_config() -> anyhow::Result<Config> {
    let config_path = Config::path();

    if config_path.exists() {
        Config::load(&config_path)
    } else {
        // Create default config silently
        let config_dir = config_path.parent().unwrap();
        std::fs::create_dir_all(config_dir)?;

        let config = Config::default();
        config.save(&config_path)?;

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        let cli = Cli::parse_from(["faster", "--debug"]);
        assert!(cli.debug);
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.stt.provider, "macos-native");
        assert_eq!(config.intent.confidence_threshold, 0.80);
    }
}
