use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use std::path::Path;

use sklink::error::AppError;

const LONG_ABOUT: &str = "Install skills via a local store and symlinks.

Local store: ~/.config/sklink/skills
Config file: ~/.config/sklink/config.toml

Modes:
- Install to local store: -i/--install <SRC>...
- Sync local store to platform targets: --async [-p/--platform <PLATFORM|all>]
- Output skills to a project directory: -o/--output <SKILL>... [--dir <DIR>] [--export]

Rules:
- Install copies skills into the local store
- Sync links from target dirs to the local store
- Output links (or copies with --export) from the local store into a directory
- If link exists and points to expected target: skip
- Otherwise (file/dir or wrong target): error";

const AFTER_HELP: &str = "Quick setup for shell completions:
  zsh:  mkdir -p ~/.zsh/completions && sklink completions zsh > ~/.zsh/completions/_sklink
  bash: sklink completions bash | sudo tee /etc/bash_completion.d/sklink
  fish: mkdir -p ~/.config/fish/completions && sklink completions fish > ~/.config/fish/completions/sklink.fish

Tip: when using cargo run, pass CLI args after `--` (e.g. cargo run -- --help)";

#[derive(Parser, Debug)]
#[command(name = "sklink", version, long_about = LONG_ABOUT, after_help = AFTER_HELP, args_conflicts_with_subcommands = true, subcommand_precedence_over_arg = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(
        short = 'i',
        long = "install",
        value_name = "SRC",
        help = "Install source (skill name, local dir, or git url). Repeatable."
    )]
    install_sources: Vec<String>,

    #[arg(
        short = 'p',
        long = "platform",
        value_name = "PLATFORM|all",
        help = "Limit --async to a specific platform or all",
        requires = "async_sync"
    )]
    platform: Option<String>,

    #[arg(long, help = "Overwrite existing skill in local store")]
    #[arg(requires = "install_sources", conflicts_with = "outputs")]
    force: bool,

    #[arg(
        long = "async",
        help = "Sync local store skills into platform target directories"
    )]
    async_sync: bool,

    #[arg(
        short = 'o',
        long = "output",
        value_name = "SKILL",
        help = "Output skill from local store into a directory (repeatable)",
        conflicts_with_all = ["install_sources", "force", "async_sync", "platform"]
    )]
    outputs: Vec<String>,

    #[arg(
        long = "dir",
        value_name = "DIR",
        help = "Output directory for -o/--output (default: .agent/skills)",
        requires = "outputs"
    )]
    output_dir: Option<String>,

    #[arg(
        long,
        help = "Copy skills instead of creating symlinks (only for -o/--output)"
    )]
    #[arg(requires = "outputs")]
    export: bool,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    Init {
        #[arg(long, help = "Overwrite config if it already exists")]
        force: bool,
    },
    List {
        #[arg(long, help = "Show installed skills across configured targets")]
        installed: bool,
    },
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        print_error(&err);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), AppError> {
    let cwd = std::env::current_dir().map_err(AppError::Io)?;
    if let Some(cmd) = cli.command {
        return match cmd {
            Commands::Init { force } => {
                let path = sklink::init::init_config(&cwd, sklink::init::InitOptions { force })?;
                println!("created config: {}", display_path(&path));
                Ok(())
            }
            Commands::List { installed } => {
                if installed {
                    return sklink::skills::list_installed(&cwd);
                }
                sklink::skills::list_available(&cwd)
            }
            Commands::Completions { shell } => {
                let mut cmd = Cli::command();
                generate(shell, &mut cmd, "sklink", &mut std::io::stdout());
                Ok(())
            }
        };
    }

    if cli.install_sources.is_empty() && cli.outputs.is_empty() && !cli.async_sync {
        let mut cmd = Cli::command();
        cmd.print_help().map_err(AppError::Io)?;
        println!();
        return Ok(());
    }

    let store_dir = sklink::store::default_store_dir(&cwd)?;

    if !cli.outputs.is_empty() {
        let store_dir = sklink::store::ensure_store_dir(&store_dir)?;
        return sklink::store::output_from_store(
            &cwd,
            &store_dir,
            &cli.outputs,
            cli.output_dir.as_deref(),
            cli.export,
        );
    }

    let mut staged = Vec::new();
    if !cli.install_sources.is_empty() {
        let store_dir = sklink::store::ensure_store_dir(&store_dir)?;
        staged =
            sklink::store::install_into_store(&cwd, &store_dir, &cli.install_sources, cli.force)?;
        for skill in &staged {
            println!("stored {} -> {}", skill.name, display_path(&skill.dir));
        }
    }

    if cli.async_sync {
        let store_dir = sklink::store::ensure_store_dir(&store_dir)?;
        let config = sklink::config::load_default_config()?;
        let _ = staged;
        sklink::install::sync_store_to_platforms(
            &cwd,
            &store_dir,
            &config,
            cli.platform.as_deref(),
        )?;
    }

    Ok(())
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn print_error(err: &AppError) {
    eprintln!("{err}");
    match err {
        AppError::ConfigRead { .. } | AppError::ConfigParse { .. } => {
            eprintln!("hint: run 'sklink init' to generate a default config");
        }
        AppError::ConfigAlreadyExists { .. } => {
            eprintln!("hint: run 'sklink init --force' to overwrite the existing config");
        }
        _ => {}
    }
}
