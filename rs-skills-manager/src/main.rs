mod config;
mod error;
mod init;
mod install;
mod path_utils;
mod skills;

use clap::Parser;
use std::path::Path;

use crate::error::AppError;

#[derive(Parser, Debug)]
#[command(name = "rs-skills-manager")]
#[command(
    about = "Install repo skills into platform directories via symlinks",
    long_about = "Install skills (directories) from ./skills (relative to current working directory) into configured platform target directories using symlinks.\n\nConfig file: ~/.config/rs-skills-manager/config.toml\n\nRules:\n- If link does not exist: create\n- If link exists and points to expected target: skip\n- Otherwise (file/dir or wrong target): error",
    after_help = "Tip: when using cargo run, pass CLI args after `--`.\nExamples:\n  cargo run -- --help\n  cargo run -- init\n  cargo run -- -o all\n  cargo run -- -i software-engineer -o kimi",
    args_conflicts_with_subcommands = true,
    subcommand_precedence_over_arg = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(
        short = 'i',
        value_name = "SKILL",
        help = "Skill name to install (repeatable). Omit to install all discovered skills"
    )]
    skills: Vec<String>,

    #[arg(
        short = 'o',
        value_name = "PLATFORM|all",
        help = "Target platform name or all"
    )]
    platform: Option<String>,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    Init {
        #[arg(long, help = "Overwrite config if it already exists")]
        force: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), AppError> {
    let cwd = std::env::current_dir().map_err(AppError::Io)?;
    if let Some(cmd) = cli.command {
        return match cmd {
            Commands::Init { force } => {
                let path = init::init_config(&cwd, init::InitOptions { force })?;
                println!("created config: {}", display_path(&path));
                Ok(())
            }
        };
    }

    let config = config::load_default_config()?;

    let repo_skills_dir = cwd.join("skills");
    let repo_skills_dir =
        std::fs::canonicalize(&repo_skills_dir).map_err(|e| AppError::RepoSkillsDirInvalid {
            path: repo_skills_dir.clone(),
            source: e,
        })?;

    let selected_skills = if cli.skills.is_empty() {
        skills::discover_skills(&repo_skills_dir)?
    } else {
        skills::validate_skills(&repo_skills_dir, &cli.skills)?
    };

    let platform = cli.platform.unwrap_or_else(|| "all".to_string());
    let platform_names: Vec<String> = if platform == "all" {
        let mut names: Vec<String> = config.platforms.keys().cloned().collect();
        names.sort();
        names
    } else {
        if !config.platforms.contains_key(&platform) {
            return Err(AppError::PlatformNotFound { platform });
        }
        vec![platform]
    };

    for platform_name in platform_names {
        let platform =
            config
                .platforms
                .get(&platform_name)
                .ok_or_else(|| AppError::PlatformNotFound {
                    platform: platform_name.clone(),
                })?;

        for target in &platform.targets {
            let target_dir = path_utils::resolve_path(&target.dir, &cwd)?;
            std::fs::create_dir_all(&target_dir).map_err(|e| AppError::CreateDir {
                dir: target_dir.clone(),
                source: e,
            })?;

            for skill in &selected_skills {
                let link_path = target_dir.join(skill);
                let link_target = repo_skills_dir.join(skill);
                let link_target =
                    std::fs::canonicalize(&link_target).map_err(|e| AppError::SkillNotFound {
                        skill: skill.clone(),
                        path: link_target.clone(),
                        source: e,
                    })?;

                match install::ensure_correct_symlink(&link_path, &link_target)? {
                    install::InstallOutcome::Created => {
                        println!(
                            "created {} -> {}",
                            display_path(&link_path),
                            display_path(&link_target)
                        );
                    }
                    install::InstallOutcome::Skipped => {
                        println!("skipped {}", display_path(&link_path));
                    }
                }
            }
        }
    }

    Ok(())
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
