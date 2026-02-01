//! Skills command implementation

use anyhow::{Context, Result};
use colored::Colorize;
use dirs;
use std::fs;

use crate::cli::{SkillsAction, SkillsCommand};
use crate::skills::{SkillCategory, SkillsManager};

pub async fn execute(cmd: SkillsCommand) -> Result<()> {
    match cmd.action {
        SkillsAction::List { category } => list_skills(category).await,
        SkillsAction::Create {
            name,
            category,
            project,
        } => create_skill(name, category, project).await,
        SkillsAction::Show { name } => show_skill(name).await,
        SkillsAction::Remove { name, force } => remove_skill(name, force).await,
        SkillsAction::Open => open_skills_dir().await,
        SkillsAction::Import { source, name } => import_skills(source, name).await,
        SkillsAction::Available { source } => list_available_skills(source).await,
    }
}

async fn list_skills(category_filter: Option<String>) -> Result<()> {
    let mut manager = SkillsManager::new()?;
    manager.discover()?;

    let skills = manager.skills();

    if skills.is_empty() {
        println!("{}", "No skills found.".yellow());
        println!();
        println!("Create your first skill with:");
        println!("  {}", "rco skills create my-skill".cyan());
        return Ok(());
    }

    // Filter by category if specified
    let filtered_skills: Vec<_> = if let Some(ref cat) = category_filter {
        let category = parse_category(cat);
        manager
            .by_category(&category)
            .into_iter()
            .cloned()
            .collect()
    } else {
        skills.to_vec()
    };

    if filtered_skills.is_empty() {
        println!(
            "{}",
            format!("No skills found in category: {}", category_filter.unwrap()).yellow()
        );
        return Ok(());
    }

    println!("{}", "Available Skills".bold().underline());
    println!();

    // Group by category
    let mut by_category: std::collections::HashMap<String, Vec<_>> =
        std::collections::HashMap::new();
    for skill in &filtered_skills {
        by_category
            .entry(skill.category().to_string())
            .or_default()
            .push(skill);
    }

    // Print by category
    let mut categories: Vec<_> = by_category.keys().collect();
    categories.sort();

    for category in categories {
        println!("{}", format!("[{}]", category).cyan().bold());
        for skill in by_category.get(category).unwrap() {
            let source_marker = match skill.source() {
                crate::skills::SkillSource::Builtin => format!(" {}", "[built-in]".dimmed()),
                crate::skills::SkillSource::Project => {
                    format!(" {}", "[project]".yellow().dimmed())
                }
                crate::skills::SkillSource::User => String::new(),
            };

            println!(
                "  {}{}\n    {}",
                skill.name().green(),
                source_marker,
                skill.description().dimmed()
            );

            // Show tags if any
            if !skill.manifest.skill.tags.is_empty() {
                let tags: Vec<_> = skill
                    .manifest
                    .skill
                    .tags
                    .iter()
                    .map(|t| format!("#{}", t))
                    .collect();
                println!("    {}", tags.join(" ").dimmed());
            }
        }
        println!();
    }

    println!(
        "Total: {} skill{}",
        filtered_skills.len(),
        if filtered_skills.len() == 1 { "" } else { "s" }
    );

    Ok(())
}

async fn create_skill(name: String, category: String, project: bool) -> Result<()> {
    let manager = SkillsManager::new()?;
    let skill_category = parse_category(&category);

    let skill_path = if project {
        // Create project-level skill
        let project_dir = manager.ensure_project_skills_dir()?.ok_or_else(|| {
            anyhow::anyhow!("Not in a git repository. Cannot create project-level skill.")
        })?;

        println!(
            "{} Creating new {} project skill '{}'...",
            "→".cyan(),
            skill_category.to_string().cyan(),
            name.green()
        );

        let skill_dir = project_dir.join(&name);
        if skill_dir.exists() {
            anyhow::bail!(
                "Project skill '{}' already exists at {}",
                name,
                skill_dir.display()
            );
        }

        fs::create_dir_all(&skill_dir)?;
        create_skill_files(&skill_dir, &name, skill_category)?;
        skill_dir
    } else {
        // Create user-level skill
        println!(
            "{} Creating new {} user skill '{}'...",
            "→".cyan(),
            skill_category.to_string().cyan(),
            name.green()
        );

        manager.create_skill(&name, skill_category)?
    };

    println!(
        "{} Skill created at: {}",
        "✓".green(),
        skill_path.display().to_string().cyan()
    );
    println!();
    println!("Next steps:");
    println!(
        "  1. Edit {} to customize your skill",
        skill_path.join("skill.toml").display().to_string().cyan()
    );
    println!(
        "  2. Modify {} with your custom prompt",
        skill_path.join("prompt.md").display().to_string().cyan()
    );
    println!(
        "  3. Use your skill: {}",
        format!("rco --skill {}", name).cyan()
    );

    if project {
        println!();
        println!(
            "{}",
            "Note: Project skills are shared with everyone who clones this repo."
                .yellow()
                .dimmed()
        );
        println!(
            "{}",
            "      Make sure to commit the .rco/skills/ directory to version control."
                .yellow()
                .dimmed()
        );
    }

    Ok(())
}

/// Create skill files (skill.toml and prompt.md)
fn create_skill_files(
    skill_dir: &std::path::Path,
    name: &str,
    category: crate::skills::SkillCategory,
) -> Result<()> {
    use crate::skills::{SkillManifest, SkillMeta};

    // Create skill.toml
    let manifest = SkillManifest {
        skill: SkillMeta {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: format!("A {} skill for rusty-commit", category),
            author: None,
            category,
            tags: vec![],
        },
        hooks: None,
        config: None,
    };

    let manifest_content = toml::to_string_pretty(&manifest)?;
    fs::write(skill_dir.join("skill.toml"), manifest_content)?;

    // Create prompt.md template
    let prompt_template = r#"# Custom Prompt Template

You are a commit message generator. Analyze the following diff and generate a commit message.

## Diff

```diff
{diff}
```

## Context

{context}

## Instructions

Generate a commit message that:
- Follows the conventional commit format
- Is clear and concise
- Describes the changes accurately
"#;

    fs::write(skill_dir.join("prompt.md"), prompt_template)?;

    Ok(())
}

async fn show_skill(name: String) -> Result<()> {
    let mut manager = SkillsManager::new()?;
    manager.discover()?;

    let skill = manager
        .find(&name)
        .ok_or_else(|| anyhow::anyhow!("Skill '{}' not found", name))?;

    println!("{}", skill.name().bold().underline());
    println!();
    println!("{}: {}", "Description".dimmed(), skill.description());
    println!(
        "{}: {}",
        "Category".dimmed(),
        skill.category().to_string().cyan()
    );
    println!("{}: {}", "Version".dimmed(), skill.manifest.skill.version);
    println!(
        "{}: {}",
        "Source".dimmed(),
        skill.source().to_string().yellow()
    );

    if let Some(ref author) = skill.manifest.skill.author {
        println!("{}: {}", "Author".dimmed(), author);
    }

    if !skill.manifest.skill.tags.is_empty() {
        println!(
            "{}: {}",
            "Tags".dimmed(),
            skill.manifest.skill.tags.join(", ")
        );
    }

    println!(
        "{}: {}",
        "Location".dimmed(),
        skill.path.display().to_string().dimmed()
    );

    // Show hooks
    if let Some(ref hooks) = skill.manifest.hooks {
        println!();
        println!("{}", "Hooks".dimmed());
        if let Some(ref pre_gen) = hooks.pre_gen {
            println!("  {}: {}", "pre_gen".cyan(), pre_gen);
        }
        if let Some(ref post_gen) = hooks.post_gen {
            println!("  {}: {}", "post_gen".cyan(), post_gen);
        }
        if let Some(ref format) = hooks.format {
            println!("  {}: {}", "format".cyan(), format);
        }
    }

    // Show prompt template preview
    match skill.load_prompt_template() {
        Ok(Some(template)) => {
            println!();
            println!("{}", "Prompt Template Preview".dimmed());
            println!();
            // Show first 10 lines
            let lines: Vec<_> = template.lines().take(10).collect();
            for line in lines {
                println!("  {}", line.dimmed());
            }
            if template.lines().count() > 10 {
                println!("  {} ...", "...".dimmed());
            }
        }
        Ok(None) => {
            println!();
            println!("{}", "No prompt template".dimmed());
        }
        Err(e) => {
            println!();
            println!("{}: {}", "Error loading template".red(), e);
        }
    }

    Ok(())
}

async fn remove_skill(name: String, force: bool) -> Result<()> {
    let mut manager = SkillsManager::new()?;
    manager.discover()?;

    // Check if skill exists
    if manager.find(&name).is_none() {
        anyhow::bail!("Skill '{}' not found", name);
    }

    // Confirm removal unless --force
    if !force {
        use dialoguer::{theme::ColorfulTheme, Confirm};

        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Are you sure you want to remove skill '{}'?", name))
            .default(false)
            .interact()?;

        if !confirmed {
            println!("{}", "Removal cancelled.".yellow());
            return Ok(());
        }
    }

    manager.remove_skill(&name)?;

    println!("{} Skill '{}' removed.", "✓".green(), name);

    Ok(())
}

async fn open_skills_dir() -> Result<()> {
    let manager = SkillsManager::new()?;
    manager.ensure_skills_dir()?;

    let path = manager.skills_dir();

    // Try to open with the default application
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .context("Failed to open skills directory")?;
    }

    #[cfg(target_os = "linux")]
    {
        // Try xdg-open first, then fall back to other options
        let result = std::process::Command::new("xdg-open").arg(path).spawn();

        if result.is_err() {
            // Try gnome-open or kde-open
            let _ = std::process::Command::new("gnome-open")
                .arg(path)
                .spawn()
                .or_else(|_| std::process::Command::new("kde-open").arg(path).spawn())
                .context("Failed to open skills directory. Try installing xdg-open.")?;
        }
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .context("Failed to open skills directory")?;
    }

    println!(
        "{} Opened skills directory: {}",
        "✓".green(),
        path.display()
    );

    Ok(())
}

async fn import_skills(source: String, specific_name: Option<String>) -> Result<()> {
    use crate::skills::external::{
        import_from_claude_code, import_from_gist, import_from_github, import_from_url,
        parse_source,
    };

    let manager = SkillsManager::new()?;
    let target_dir = manager.skills_dir();

    // Ensure skills directory exists
    if !target_dir.exists() {
        fs::create_dir_all(target_dir)
            .with_context(|| format!("Failed to create target directory: {:?}", target_dir))?;
    }

    let source = parse_source(&source)?;

    println!(
        "{} Importing from {}...",
        "→".cyan(),
        source.to_string().cyan()
    );
    println!();

    let imported = match source {
        crate::skills::external::ExternalSource::ClaudeCode => {
            if let Some(name) = specific_name {
                // Import specific skill
                let claude_dir = dirs::home_dir()
                    .context("Could not find home directory")?
                    .join(".claude")
                    .join("skills")
                    .join(&name);

                if !claude_dir.exists() {
                    anyhow::bail!("Claude Code skill '{}' not found at {:?}", name, claude_dir);
                }

                let target = target_dir.join(&name);
                crate::skills::external::convert_claude_skill(&claude_dir, &target, &name)?;
                vec![name]
            } else {
                import_from_claude_code(target_dir)?
            }
        }
        crate::skills::external::ExternalSource::GitHub { owner, repo, path } => {
            if let Some(name) = specific_name {
                // Import specific skill from GitHub
                let specific_path = path
                    .as_ref()
                    .map(|p| format!("{}/{}", p, name))
                    .unwrap_or_else(|| format!(".rco/skills/{}", name));

                import_from_github(&owner, &repo, Some(&specific_path), target_dir)?
            } else {
                import_from_github(&owner, &repo, path.as_deref(), target_dir)?
            }
        }
        crate::skills::external::ExternalSource::Gist { id } => {
            if specific_name.is_some() {
                println!(
                    "{}",
                    "Note: Gist import doesn't support filtering by name. Importing all..."
                        .yellow()
                );
            }
            let name = import_from_gist(&id, target_dir)?;
            vec![name]
        }
        crate::skills::external::ExternalSource::Url { url } => {
            let name = import_from_url(&url, specific_name.as_deref(), target_dir)?;
            vec![name]
        }
    };

    if imported.is_empty() {
        println!(
            "{}",
            "No new skills were imported (they may already exist).".yellow()
        );
    } else {
        println!(
            "{} Successfully imported {} skill(s):",
            "✓".green(),
            imported.len()
        );
        for name in &imported {
            println!("  • {}", name.green());
        }
        println!();
        println!(
            "Use {} to see all available skills.",
            "rco skills list".cyan()
        );
    }

    Ok(())
}

async fn list_available_skills(source: String) -> Result<()> {
    use crate::skills::external::list_claude_code_skills;

    match source.as_str() {
        "claude-code" | "claude" => {
            let skills = list_claude_code_skills()?;

            if skills.is_empty() {
                println!("{}", "No Claude Code skills found.".yellow());
                println!();
                println!("Claude Code skills are stored in: ~/.claude/skills/");
                return Ok(());
            }

            println!("{}", "Available Claude Code Skills".bold().underline());
            println!();
            println!(
                "{}",
                "Run 'rco skills import claude-code [name]' to import".dimmed()
            );
            println!();

            for (name, description) in skills {
                println!("{} {}", "•".cyan(), name.green());
                println!("  {}", description.dimmed());
            }

            println!();
            println!("To import all: {}", "rco skills import claude-code".cyan());
            println!(
                "To import one: {}",
                "rco skills import claude-code --name <skill-name>".cyan()
            );
        }
        _ => {
            anyhow::bail!(
                "Unknown source: {}. Currently supported: claude-code",
                source
            );
        }
    }

    Ok(())
}

fn parse_category(s: &str) -> SkillCategory {
    match s.to_lowercase().as_str() {
        "analyzer" | "analysis" => SkillCategory::Analyzer,
        "formatter" | "format" => SkillCategory::Formatter,
        "integration" | "integrate" => SkillCategory::Integration,
        "utility" | "util" => SkillCategory::Utility,
        _ => SkillCategory::Template,
    }
}
