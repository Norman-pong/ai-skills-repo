mod config;
mod error;
mod git_source;
mod init;
mod install;
mod path_utils;
mod skills;
mod store;

use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

use crate::error::AppError;

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
#[command(name = "sklink", long_about = LONG_ABOUT, after_help = AFTER_HELP, args_conflicts_with_subcommands = true, subcommand_precedence_over_arg = true)]
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
                let path = init::init_config(&cwd, init::InitOptions { force })?;
                println!("created config: {}", display_path(&path));
                Ok(())
            }
            Commands::List { installed } => {
                if installed {
                    return list_installed(&cwd);
                }
                list_available(&cwd)
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

    let store_dir = store::default_store_dir(&cwd)?;

    if !cli.outputs.is_empty() {
        std::fs::create_dir_all(&store_dir).map_err(|e| AppError::CreateDir {
            dir: store_dir.clone(),
            source: e,
        })?;
        let store_dir = std::fs::canonicalize(&store_dir).map_err(AppError::Io)?;
        return output_from_store(
            &cwd,
            &store_dir,
            &cli.outputs,
            cli.output_dir.as_deref(),
            cli.export,
        );
    }

    let mut staged = Vec::new();
    if !cli.install_sources.is_empty() {
        std::fs::create_dir_all(&store_dir).map_err(|e| AppError::CreateDir {
            dir: store_dir.clone(),
            source: e,
        })?;
        let store_dir = std::fs::canonicalize(&store_dir).map_err(AppError::Io)?;
        staged = install_into_store(&cwd, &store_dir, &cli.install_sources, cli.force)?;
        for skill in &staged {
            println!("stored {} -> {}", skill.name, display_path(&skill.dir));
        }
    }

    if cli.async_sync {
        std::fs::create_dir_all(&store_dir).map_err(|e| AppError::CreateDir {
            dir: store_dir.clone(),
            source: e,
        })?;
        let store_dir = std::fs::canonicalize(&store_dir).map_err(AppError::Io)?;
        let config = config::load_default_config()?;
        let _ = staged;
        sync_store_to_platforms(&cwd, &store_dir, &config, cli.platform.as_deref())?;
    }

    Ok(())
}

fn list_available(cwd: &Path) -> Result<(), AppError> {
    let store_dir = store::default_store_dir(cwd)?;

    let skills_dir = skills::detect_repo_skills_dir(cwd).unwrap_or(store_dir);
    let skills = skills::discover_skills(&skills_dir)?;
    for skill in skills {
        println!("{}", skill.name);
    }
    Ok(())
}

fn install_into_store(
    cwd: &Path,
    store_dir: &Path,
    sources: &[String],
    force: bool,
) -> Result<Vec<skills::SkillDir>, AppError> {
    let repo_skills_dir = skills::detect_repo_skills_dir(cwd).ok();
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    for raw in sources {
        if git_source::looks_like_git_url(raw) {
            let staged = git_source::stage_from_git_url(raw, store_dir, cwd, force)?;
            for skill in staged {
                if seen.insert(skill.name.clone()) {
                    out.push(skill);
                }
            }
            continue;
        }

        let (name, raw_dir) = if looks_like_path(raw) {
            let dir = path_utils::resolve_path(raw, cwd)?;
            let name = dir
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .ok_or_else(|| AppError::SkillNotFound {
                    skill: raw.clone(),
                    path: dir.clone(),
                    source: std::io::Error::other("missing directory name"),
                })?;
            (name, dir)
        } else {
            let Some(repo_skills_dir) = repo_skills_dir.as_ref() else {
                return Err(AppError::RepoSkillsDirInvalid {
                    path: cwd.join("skills"),
                    source: std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "skills dir not found",
                    ),
                });
            };
            (raw.clone(), repo_skills_dir.join(raw))
        };

        if !seen.insert(name.clone()) {
            continue;
        }

        validate_skill_dir(raw, &raw_dir)?;
        let raw_dir = std::fs::canonicalize(&raw_dir).map_err(|e| AppError::SkillNotFound {
            skill: raw.clone(),
            path: raw_dir.clone(),
            source: e,
        })?;

        let dir = store::stage_skill_to_store(store_dir, &name, &raw_dir, force)?;
        out.push(skills::SkillDir {
            name,
            dir: std::fs::canonicalize(dir).map_err(AppError::Io)?,
        });
    }

    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

fn looks_like_path(raw: &str) -> bool {
    raw.contains('/') || raw.starts_with('.') || raw.starts_with('~')
}

fn validate_skill_dir(skill: &str, dir: &PathBuf) -> Result<(), AppError> {
    let meta = std::fs::metadata(dir).map_err(|e| AppError::SkillNotFound {
        skill: skill.to_string(),
        path: dir.clone(),
        source: e,
    })?;
    if !meta.is_dir() {
        return Err(AppError::SkillNotFound {
            skill: skill.to_string(),
            path: dir.clone(),
            source: std::io::Error::other("not a directory"),
        });
    }
    Ok(())
}

fn sync_store_to_platforms(
    cwd: &Path,
    store_dir: &Path,
    config: &config::Config,
    platform: Option<&str>,
) -> Result<(), AppError> {
    let selected_skills = skills::discover_skills(store_dir)?;

    let platform = platform.unwrap_or("all");
    let platform_names: Vec<String> = if platform == "all" {
        let mut names: Vec<String> = config.platforms.keys().cloned().collect();
        names.sort();
        names
    } else {
        if !config.platforms.contains_key(platform) {
            let mut names: Vec<String> = config.platforms.keys().cloned().collect();
            names.sort();
            return Err(AppError::PlatformNotFound {
                platform: platform.to_string(),
                available: names.join(", "),
            });
        }
        vec![platform.to_string()]
    };

    for platform_name in platform_names {
        let Some(platform) = config.platforms.get(&platform_name) else {
            eprintln!("warning: platform not found: {platform_name}");
            continue;
        };

        for target in &platform.targets {
            let target_dir = path_utils::resolve_path(&target.dir, cwd)?;
            let meta = match std::fs::metadata(&target_dir) {
                Ok(m) => m,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    eprintln!(
                        "warning: target dir not found (skipped): platform={platform_name} dir={}",
                        display_path(&target_dir)
                    );
                    continue;
                }
                Err(err) => {
                    eprintln!(
                        "warning: failed to read target dir (skipped): platform={platform_name} dir={} err={err}",
                        display_path(&target_dir)
                    );
                    continue;
                }
            };
            if !meta.is_dir() {
                eprintln!(
                    "warning: target path is not a directory (skipped): platform={platform_name} dir={}",
                    display_path(&target_dir)
                );
                continue;
            }

            for skill in &selected_skills {
                let link_path = target_dir.join(&skill.name);
                match install::ensure_correct_symlink(&link_path, &skill.dir)? {
                    install::InstallOutcome::Created => {
                        println!(
                            "created {} -> {}",
                            display_path(&link_path),
                            display_path(&skill.dir)
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

fn output_from_store(
    cwd: &Path,
    store_dir: &Path,
    outputs: &[String],
    output_dir: Option<&str>,
    export: bool,
) -> Result<(), AppError> {
    let output_dir = output_dir.unwrap_or(".agent/skills");
    let output_dir = path_utils::resolve_path(output_dir, cwd)?;
    std::fs::create_dir_all(&output_dir).map_err(|e| AppError::CreateDir {
        dir: output_dir.clone(),
        source: e,
    })?;

    let mut seen = HashSet::new();
    for name in outputs {
        if !seen.insert(name.clone()) {
            continue;
        }

        let store_skill = store_dir.join(name);
        validate_skill_dir(name, &store_skill)?;
        let dest = output_dir.join(name);

        if export {
            if dest.exists() {
                return Err(AppError::OutputPathExists { path: dest });
            }
            store::copy_dir_recursive(&store_skill, &dest)?;
            println!("exported {} -> {}", name, display_path(&dest));
        } else {
            match install::ensure_correct_symlink(&dest, &store_skill)? {
                install::InstallOutcome::Created => {
                    println!(
                        "created {} -> {}",
                        display_path(&dest),
                        display_path(&store_skill)
                    );
                }
                install::InstallOutcome::Skipped => {
                    println!("skipped {}", display_path(&dest));
                }
            }
        }
    }

    Ok(())
}

fn list_installed(cwd: &Path) -> Result<(), AppError> {
    let config = config::load_default_config()?;
    let store_dir = store::default_store_dir(cwd)?;
    let store_dir = std::fs::canonicalize(&store_dir).ok();

    let mut platform_names: Vec<String> = config.platforms.keys().cloned().collect();
    platform_names.sort();

    for platform_name in platform_names {
        let Some(platform) = config.platforms.get(&platform_name) else {
            continue;
        };

        println!("{platform_name}");

        let mut target_dirs = Vec::new();
        for target in &platform.targets {
            let target_dir = path_utils::resolve_path(&target.dir, cwd)?;
            let meta = match std::fs::metadata(&target_dir) {
                Ok(m) => m,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    eprintln!(
                        "warning: target dir not found (skipped): platform={platform_name} dir={}",
                        display_path(&target_dir)
                    );
                    continue;
                }
                Err(err) => {
                    eprintln!(
                        "warning: failed to read target dir (skipped): platform={platform_name} dir={} err={err}",
                        display_path(&target_dir)
                    );
                    continue;
                }
            };
            if !meta.is_dir() {
                eprintln!(
                    "warning: target path is not a directory (skipped): platform={platform_name} dir={}",
                    display_path(&target_dir)
                );
                continue;
            }
            target_dirs.push(target_dir);
        }

        target_dirs.sort_by_key(|a| display_path(a));

        for (target_idx, target_dir) in target_dirs.iter().enumerate() {
            let target_prefix = if target_idx + 1 == target_dirs.len() {
                "└──"
            } else {
                "├──"
            };
            println!("{target_prefix} {}", display_path(target_dir));

            let entries = match std::fs::read_dir(target_dir) {
                Ok(e) => e,
                Err(err) => {
                    eprintln!(
                        "warning: failed to list target dir (skipped): platform={platform_name} dir={} err={err}",
                        display_path(target_dir)
                    );
                    continue;
                }
            };

            let mut rendered = Vec::new();
            for entry in entries {
                let entry = match entry {
                    Ok(e) => e,
                    Err(err) => {
                        eprintln!(
                            "warning: failed to read target entry (skipped): platform={platform_name} dir={} err={err}",
                            display_path(target_dir)
                        );
                        continue;
                    }
                };
                let name = entry.file_name().to_string_lossy().to_string();
                if name == ".DS_Store" {
                    continue;
                }

                let path = entry.path();
                let meta = match std::fs::symlink_metadata(&path) {
                    Ok(m) => m,
                    Err(err) => {
                        let line = format!("[?] {name} (error:{err})");
                        rendered.push((name, line));
                        continue;
                    }
                };

                let line = if meta.file_type().is_symlink() {
                    match std::fs::read_link(&path) {
                        Ok(raw_target) => {
                            let resolved = install::resolve_symlink_target(&path, &raw_target);
                            match std::fs::canonicalize(&resolved) {
                                Ok(resolved) => {
                                    let status = match &store_dir {
                                        Some(store_dir) => {
                                            let expected = store_dir.join(&name);
                                            let expected = std::fs::canonicalize(expected).ok();
                                            if expected.is_some_and(|e| e == resolved) {
                                                "ok".to_string()
                                            } else {
                                                "outside-store".to_string()
                                            }
                                        }
                                        None => "unknown-store".to_string(),
                                    };
                                    format!("[L] {name} -> {} ({status})", display_path(&resolved))
                                }
                                Err(err) => format!("[L] {name} (broken:{err})"),
                            }
                        }
                        Err(err) => format!("[L] {name} (broken:{err})"),
                    }
                } else if meta.is_dir() {
                    format!("[D] {name}")
                } else if meta.is_file() {
                    format!("[F] {name}")
                } else {
                    format!("[?] {name}")
                };

                rendered.push((name, line));
            }

            rendered.sort_by(|a, b| a.0.cmp(&b.0));

            for (entry_idx, (_, line)) in rendered.iter().enumerate() {
                let has_more_targets = target_idx + 1 != target_dirs.len();
                let indent = if has_more_targets { "│   " } else { "    " };
                let prefix = if entry_idx + 1 == rendered.len() {
                    "└──"
                } else {
                    "├──"
                };
                println!("{indent}{prefix} {line}");
            }
        }
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
