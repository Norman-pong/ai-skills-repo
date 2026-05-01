mod config;
mod error;
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

#[derive(Parser, Debug)]
#[command(name = "sklink")]
#[command(
    about = "Install skills into platform directories via local store and symlinks",
    long_about = "Install skills (directories) into a local store and then link them into configured platform target directories.\n\nLocal store: ~/.config/sklink/skills\nConfig file: ~/.config/sklink/config.toml\n\nRules:\n- Skills are copied into the local store before linking\n- If a local store skill already exists: error unless --force is used\n- If link does not exist: create\n- If link exists and points to expected target: skip\n- Otherwise (file/dir or wrong target): error",
    after_help = "Tip: when using cargo run, pass CLI args after `--`.\nExamples:\n  cargo run -- --help\n  cargo run -- init\n  cargo run -- list\n  cargo run -- list --installed\n  cargo run -- -p all\n  cargo run -- -i software-engineer -p kimi\n  cargo run -- --force -i software-engineer -p kimi",
    args_conflicts_with_subcommands = true,
    subcommand_precedence_over_arg = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(
        short = 'i',
        long = "install",
        value_name = "SKILL|PATH",
        help = "Skill name or path to a skill directory (repeatable). Omit to install all discovered skills"
    )]
    skills: Vec<String>,

    #[arg(
        short = 'p',
        long = "platform",
        value_name = "PLATFORM|all",
        help = "Target platform name or all"
    )]
    platform: Option<String>,

    #[arg(long, help = "Overwrite existing skill in local store")]
    force: bool,
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

    let config = config::load_default_config()?;
    let store_dir = store::default_store_dir(&cwd)?;
    std::fs::create_dir_all(&store_dir).map_err(|e| AppError::CreateDir {
        dir: store_dir.clone(),
        source: e,
    })?;
    let store_dir = std::fs::canonicalize(&store_dir).map_err(AppError::Io)?;

    let repo_skills_dir = skills::detect_repo_skills_dir(&cwd).ok();

    let source_skills = if cli.skills.is_empty() {
        let dir = repo_skills_dir.clone().unwrap_or_else(|| store_dir.clone());
        skills::discover_skills(&dir)?
    } else {
        resolve_requested_skills(
            repo_skills_dir.as_deref(),
            &store_dir,
            &cwd,
            &cli.skills,
            cli.force,
        )?
    };

    let mut selected_skills = Vec::new();
    let is_bulk_install = cli.skills.is_empty();
    for source in source_skills {
        let src = std::fs::canonicalize(&source.dir).map_err(AppError::Io)?;
        let expected_store = store_dir.join(&source.name);
        let src_is_store = src == expected_store;
        let store_exists = expected_store.exists();

        let dir = if src_is_store {
            src
        } else if is_bulk_install && store_exists && !cli.force {
            expected_store
        } else if !is_bulk_install && store_exists && !cli.force {
            return Err(AppError::StoreSkillAlreadyExists {
                skill: source.name,
                path: expected_store,
            });
        } else {
            store::stage_skill_to_store(&store_dir, &source.name, &src, cli.force)?
        };
        selected_skills.push(skills::SkillDir {
            name: source.name,
            dir,
        });
    }

    let platform = cli.platform.unwrap_or_else(|| "all".to_string());
    let platform_names: Vec<String> = if platform == "all" {
        let mut names: Vec<String> = config.platforms.keys().cloned().collect();
        names.sort();
        names
    } else {
        if !config.platforms.contains_key(&platform) {
            let mut names: Vec<String> = config.platforms.keys().cloned().collect();
            names.sort();
            return Err(AppError::PlatformNotFound {
                platform,
                available: names.join(", "),
            });
        }
        vec![platform]
    };

    for platform_name in platform_names {
        let Some(platform) = config.platforms.get(&platform_name) else {
            eprintln!("warning: platform not found: {platform_name}");
            continue;
        };

        for target in &platform.targets {
            let target_dir = path_utils::resolve_path(&target.dir, &cwd)?;
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

fn list_available(cwd: &Path) -> Result<(), AppError> {
    let store_dir = store::default_store_dir(cwd)?;

    let skills_dir = skills::detect_repo_skills_dir(cwd).unwrap_or(store_dir);
    let skills = skills::discover_skills(&skills_dir)?;
    for skill in skills {
        println!("{}", skill.name);
    }
    Ok(())
}

fn resolve_requested_skills(
    repo_skills_dir: Option<&Path>,
    store_dir: &Path,
    cwd: &Path,
    requested: &[String],
    prefer_repo: bool,
) -> Result<Vec<skills::SkillDir>, AppError> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    for raw in requested {
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
            let repo_candidate = repo_skills_dir.map(|d| d.join(raw));
            if prefer_repo && repo_candidate.as_ref().is_some_and(|d| d.is_dir()) {
                (raw.clone(), repo_candidate.unwrap())
            } else {
                let store_candidate = store_dir.join(raw);
                if store_candidate.is_dir() {
                    (raw.clone(), store_candidate)
                } else if let Some(repo_candidate) = repo_candidate {
                    (raw.clone(), repo_candidate)
                } else {
                    return Err(AppError::SkillNotFound {
                        skill: raw.clone(),
                        path: store_candidate,
                        source: std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "skill not found in local store and repo skills dir not found",
                        ),
                    });
                }
            }
        };

        if !seen.insert(name.clone()) {
            continue;
        }

        validate_skill_dir(raw, &raw_dir)?;
        let dir = std::fs::canonicalize(&raw_dir).map_err(|e| AppError::SkillNotFound {
            skill: raw.clone(),
            path: raw_dir.clone(),
            source: e,
        })?;
        out.push(skills::SkillDir { name, dir });
    }

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
