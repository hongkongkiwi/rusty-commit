//! Skills System for Rusty Commit
//!
//! Skills are modular extensions that allow users to customize and extend
//! rusty-commit's functionality. They can be used to:
//!
//! - Define custom commit message templates
//! - Add custom analysis modules
//! - Create custom output formatters
//! - Hook into the commit generation pipeline
//!
//! # Skill Structure
//!
//! Skills are stored in `~/.config/rustycommit/skills/` and are either:
//! - **Built-in**: Included with rusty-commit
//! - **Local**: User-created skills in the skills directory
//! - **Project**: Team-shared skills in `.rco/skills/`
//! - **External**: Imported from Claude Code, GitHub, etc.
//!
//! # Skill Manifest
//!
//! Each skill has a `skill.toml` manifest:
//! ```toml
//! [skill]
//! name = "conventional-with-scope"
//! version = "1.0.0"
//! description = "Conventional commits with automatic scope detection"
//! author = "Your Name"
//!
//! [skill.hooks]
//! pre_gen = "pre_gen.sh"      # Optional: runs before AI generation
//! post_gen = "post_gen.sh"    # Optional: runs after AI generation
//! format = "format.sh"        # Optional: formats the output
//! ```
//!
//! # External Skills
//!
//! rusty-commit can import skills from:
//! - **Claude Code**: `~/.claude/skills/` - Claude Code custom skills
//! - **GitHub**: Repositories with `.rco/skills/` directory
//! - **GitHub Gist**: Single-file skill definitions
//! - **URL**: Direct download from any HTTP(S) URL

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Skills directory name
pub const SKILLS_DIR: &str = "skills";

/// Skill manifest filename
pub const SKILL_MANIFEST: &str = "skill.toml";

/// A skill manifest defining the skill's metadata and capabilities
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillManifest {
    /// Skill metadata
    pub skill: SkillMeta,
    /// Optional hooks for the skill
    #[serde(default)]
    pub hooks: Option<SkillHooks>,
    /// Configuration schema (optional)
    #[serde(default)]
    pub config: Option<HashMap<String, SkillConfigOption>>,
}

/// Skill metadata
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillMeta {
    /// Unique name for the skill
    pub name: String,
    /// Semantic version
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// Author name or email
    pub author: Option<String>,
    /// Skill category
    #[serde(default)]
    pub category: SkillCategory,
    /// Tags for discovery
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Skill categories
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SkillCategory {
    /// Prompt templates and generators
    #[default]
    Template,
    /// Analysis and transformation
    Analyzer,
    /// Output formatting
    Formatter,
    /// Integration with external tools
    Integration,
    /// Utility functions
    Utility,
}

impl std::fmt::Display for SkillCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillCategory::Template => write!(f, "template"),
            SkillCategory::Analyzer => write!(f, "analyzer"),
            SkillCategory::Formatter => write!(f, "formatter"),
            SkillCategory::Integration => write!(f, "integration"),
            SkillCategory::Utility => write!(f, "utility"),
        }
    }
}

/// Skill hooks for pipeline integration
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SkillHooks {
    /// Runs before AI generation (receives diff, can modify)
    pub pre_gen: Option<String>,
    /// Runs after AI generation (receives message, can modify)
    pub post_gen: Option<String>,
    /// Formats the final output
    pub format: Option<String>,
}

/// Configuration option schema
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillConfigOption {
    /// Option type (string, bool, number)
    pub r#type: String,
    /// Default value
    #[serde(default)]
    pub default: Option<toml::Value>,
    /// Description
    pub description: String,
    /// Whether it's required
    #[serde(default)]
    pub required: bool,
}

/// A loaded skill with its manifest and path
#[derive(Debug, Clone)]
pub struct Skill {
    /// Skill manifest
    pub manifest: SkillManifest,
    /// Path to the skill directory
    pub path: PathBuf,
    /// Source/origin of the skill
    pub source: SkillSource,
}

impl Skill {
    /// Get the skill name
    pub fn name(&self) -> &str {
        &self.manifest.skill.name
    }

    /// Get the skill description
    pub fn description(&self) -> &str {
        &self.manifest.skill.description
    }

    /// Get the skill category
    pub fn category(&self) -> &SkillCategory {
        &self.manifest.skill.category
    }

    /// Get the skill source
    pub fn source(&self) -> &SkillSource {
        &self.source
    }

    /// Check if the skill is built-in
    pub fn is_builtin(&self) -> bool {
        matches!(self.source, SkillSource::Builtin)
    }

    /// Check if the skill is from the project
    pub fn is_project_skill(&self) -> bool {
        matches!(self.source, SkillSource::Project)
    }

    /// Check if the skill has a pre_gen hook
    pub fn has_pre_gen(&self) -> bool {
        self.manifest
            .hooks
            .as_ref()
            .and_then(|h| h.pre_gen.as_ref())
            .is_some()
    }

    /// Check if the skill has a post_gen hook
    pub fn has_post_gen(&self) -> bool {
        self.manifest
            .hooks
            .as_ref()
            .and_then(|h| h.post_gen.as_ref())
            .is_some()
    }

    /// Get the pre_gen hook path
    pub fn pre_gen_path(&self) -> Option<PathBuf> {
        self.manifest.hooks.as_ref().and_then(|h| {
            h.pre_gen
                .as_ref()
                .map(|script| self.path.join(script))
        })
    }

    /// Get the post_gen hook path
    pub fn post_gen_path(&self) -> Option<PathBuf> {
        self.manifest.hooks.as_ref().and_then(|h| {
            h.post_gen
                .as_ref()
                .map(|script| self.path.join(script))
        })
    }

    /// Load a prompt template from the skill if available
    pub fn load_prompt_template(&self) -> Result<Option<String>> {
        let prompt_path = self.path.join("prompt.md");
        if prompt_path.exists() {
            let content = fs::read_to_string(&prompt_path)
                .with_context(|| format!("Failed to read prompt template: {}", prompt_path.display()))?;
            Ok(Some(content))
        } else {
            Ok(None)
        }
    }
}

/// Skill source/origin
#[derive(Debug, Clone, PartialEq)]
pub enum SkillSource {
    /// Built-in skill shipped with rusty-commit
    Builtin,
    /// User-specific skill in ~/.config/rustycommit/skills/
    User,
    /// Project-level skill in .rco/skills/
    Project,
}

impl std::fmt::Display for SkillSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillSource::Builtin => write!(f, "built-in"),
            SkillSource::User => write!(f, "user"),
            SkillSource::Project => write!(f, "project"),
        }
    }
}

/// Skills manager - handles discovery and loading of skills
pub struct SkillsManager {
    /// User skills directory
    user_skills_dir: PathBuf,
    /// Project skills directory (optional)
    project_skills_dir: Option<PathBuf>,
    /// Loaded skills
    skills: Vec<Skill>,
}

impl SkillsManager {
    /// Create a new skills manager
    pub fn new() -> Result<Self> {
        let user_skills_dir = Self::user_skills_dir()?;
        let project_skills_dir = Self::project_skills_dir()?;
        Ok(Self {
            user_skills_dir,
            project_skills_dir,
            skills: Vec::new(),
        })
    }

    /// Get the user skills directory
    fn user_skills_dir() -> Result<PathBuf> {
        let config_dir = if let Ok(config_home) = std::env::var("RCO_CONFIG_HOME") {
            PathBuf::from(config_home)
        } else {
            dirs::home_dir()
                .context("Could not find home directory")?
                .join(".config")
                .join("rustycommit")
        };
        Ok(config_dir.join(SKILLS_DIR))
    }

    /// Get the project skills directory (if in a git repo)
    fn project_skills_dir() -> Result<Option<PathBuf>> {
        use crate::git;
        
        // Try to find the git repo root
        if let Ok(repo_root) = git::get_repo_root() {
            let project_skills = Path::new(&repo_root).join(".rco").join("skills");
            if project_skills.exists() {
                return Ok(Some(project_skills));
            }
        }
        Ok(None)
    }

    /// Check if project skills are available
    pub fn has_project_skills(&self) -> bool {
        self.project_skills_dir.is_some()
    }

    /// Get the project skills directory
    pub fn project_skills_path(&self) -> Option<&Path> {
        self.project_skills_dir.as_deref()
    }

    /// Ensure the skills directory exists
    pub fn ensure_skills_dir(&self) -> Result<()> {
        if !self.user_skills_dir.exists() {
            fs::create_dir_all(&self.user_skills_dir)
                .with_context(|| format!("Failed to create skills directory: {}", self.user_skills_dir.display()))?;
        }
        Ok(())
    }

    /// Discover and load all skills (user + project)
    /// Project skills take precedence over user skills with the same name
    pub fn discover(&mut self) -> Result<&mut Self> {
        self.skills.clear();
        let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();

        // First, load project skills (they take precedence)
        if let Some(ref project_dir) = self.project_skills_dir {
            if project_dir.exists() {
                for entry in fs::read_dir(project_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        if let Ok(skill) = Self::load_skill(&path, SkillSource::Project) {
                            seen_names.insert(skill.name().to_string());
                            self.skills.push(skill);
                        }
                    }
                }
            }
        }

        // Then, load user skills (skip duplicates)
        if self.user_skills_dir.exists() {
            for entry in fs::read_dir(&self.user_skills_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if let Ok(skill) = Self::load_skill(&path, SkillSource::User) {
                        if !seen_names.contains(skill.name()) {
                            self.skills.push(skill);
                        }
                    }
                }
            }
        }

        // Sort skills by name
        self.skills.sort_by(|a, b| a.name().cmp(b.name()));

        Ok(self)
    }

    /// Load a skill from a directory
    fn load_skill(path: &Path, source: SkillSource) -> Result<Skill> {
        let manifest_path = path.join(SKILL_MANIFEST);
        let manifest_content = fs::read_to_string(&manifest_path)
            .with_context(|| format!("Failed to read skill manifest: {}", manifest_path.display()))?;
        let manifest: SkillManifest = toml::from_str(&manifest_content)
            .with_context(|| format!("Failed to parse skill manifest: {}", manifest_path.display()))?;

        Ok(Skill {
            manifest,
            path: path.to_path_buf(),
            source,
        })
    }

    /// Get all loaded skills
    pub fn skills(&self) -> &[Skill] {
        &self.skills
    }

    /// Find a skill by name
    pub fn find(&self, name: &str) -> Option<&Skill> {
        self.skills.iter().find(|s| s.name() == name)
    }

    /// Find a skill by name (mutable)
    pub fn find_mut(&mut self, name: &str) -> Option<&mut Skill> {
        self.skills.iter_mut().find(|s| s.name() == name)
    }

    /// Get skills by category
    pub fn by_category(&self, category: &SkillCategory) -> Vec<&Skill> {
        self.skills
            .iter()
            .filter(|s| std::mem::discriminant(s.category()) == std::mem::discriminant(category))
            .collect()
    }

    /// Create a new skill from a template
    pub fn create_skill(&self, name: &str, category: SkillCategory) -> Result<PathBuf> {
        self.ensure_skills_dir()?;

        let skill_dir = self.user_skills_dir.join(name);
        if skill_dir.exists() {
            anyhow::bail!("Skill '{}' already exists at {}", name, skill_dir.display());
        }

        fs::create_dir_all(&skill_dir)?;

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
        fs::write(skill_dir.join(SKILL_MANIFEST), manifest_content)?;

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

        Ok(skill_dir)
    }

    /// Remove a skill (only user skills can be removed)
    pub fn remove_skill(&mut self, name: &str) -> Result<()> {
        // Find the skill to check if it's a user skill
        if let Some(skill) = self.find(name) {
            if !matches!(skill.source, SkillSource::User) {
                anyhow::bail!(
                    "Cannot remove {} skill '{}'. Only user skills can be removed.",
                    skill.source, name
                );
            }
        }

        let skill_dir = self.user_skills_dir.join(name);
        if !skill_dir.exists() {
            anyhow::bail!("Skill '{}' not found", name);
        }

        fs::remove_dir_all(&skill_dir)
            .with_context(|| format!("Failed to remove skill directory: {}", skill_dir.display()))?;

        // Remove from loaded skills
        self.skills.retain(|s| s.name() != name);

        Ok(())
    }

    /// Get the user skills directory path
    pub fn skills_dir(&self) -> &Path {
        &self.user_skills_dir
    }

    /// Get or create the project skills directory
    pub fn ensure_project_skills_dir(&self) -> Result<Option<PathBuf>> {
        use crate::git;
        
        if let Ok(repo_root) = git::get_repo_root() {
            let project_skills = Path::new(&repo_root).join(".rco").join("skills");
            if !project_skills.exists() {
                fs::create_dir_all(&project_skills)
                    .with_context(|| format!("Failed to create project skills directory: {}", project_skills.display()))?;
            }
            return Ok(Some(project_skills));
        }
        Ok(None)
    }
}

impl Default for SkillsManager {
    fn default() -> Self {
        Self::new().expect("Failed to create skills manager")
    }
}

/// Built-in skills that are always available
pub mod builtin {

    /// Get the conventional commit skill prompt
    pub fn conventional_prompt(diff: &str, context: Option<&str>, language: &str) -> String {
        let context_str = context.unwrap_or("None");
        format!(
            r#"You are an expert at writing conventional commit messages.

Analyze the following git diff and generate a conventional commit message.

## Rules
- Use format: <type>(<scope>): <description>
- Types:
  - feat: A new feature
  - fix: A bug fix
  - docs: Documentation only changes
  - style: Changes that don't affect code meaning (formatting, semicolons, etc.)
  - refactor: Code change that neither fixes a bug nor adds a feature
  - perf: Code change that improves performance
  - test: Adding or correcting tests
  - build: Changes to build system or dependencies
  - ci: Changes to CI configuration
  - chore: Other changes that don't modify src or test files
- Keep the description under 72 characters
- Use imperative mood ("add" not "added")
- Be concise but descriptive
- Scope is optional but recommended for monorepos or large projects
- For breaking changes, add ! after type/scope: feat(api)!: change API response format

## Context
{}

## Language
{}

## Diff

```diff
{}
```

Generate ONLY the commit message, no explanation:"#,
            context_str, language, diff
        )
    }

    /// Get the gitmoji skill prompt
    pub fn gitmoji_prompt(diff: &str, context: Option<&str>, language: &str) -> String {
        let context_str = context.unwrap_or("None");
        format!(
            r#"You are an expert at writing GitMoji commit messages.

Analyze the following git diff and generate a GitMoji commit message.

## Rules
- Start with an appropriate emoji
- Use format: :emoji: <description> OR emoji <description>
- Common emojis (from gitmoji.dev):
  - ✨ :sparkles: (feat) - Introduce new features
  - 🐛 :bug: (fix) - Fix a bug
  - 📝 :memo: (docs) - Add or update documentation
  - 💄 :lipstick: (style) - Add or update the UI/style files
  - ♻️ :recycle: (refactor) - Refactor code
  - ✅ :white_check_mark: (test) - Add or update tests
  - 🔧 :wrench: (chore) - Add or update configuration files
  - ⚡️ :zap: (perf) - Improve performance
  - 👷 :construction_worker: (ci) - Add or update CI build system
  - 📦 :package: (build) - Add or update compiled files/packages
  - 🎨 :art: - Improve structure/format of the code
  - 🔥 :fire: - Remove code or files
  - 🚀 :rocket: - Deploy stuff
  - 🔒 :lock: - Fix security issues
  - ⬆️ :arrow_up: - Upgrade dependencies
  - ⬇️ :arrow_down: - Downgrade dependencies
  - 📌 :pushpin: - Pin dependencies to specific versions
  - ➕ :heavy_plus_sign: - Add dependencies
  - ➖ :heavy_minus_sign: - Remove dependencies
  - 🔀 :twisted_rightwards_arrows: - Merge branches
  - 💥 :boom: - Introduce breaking changes
  - 🚑 :ambulance: - Critical hotfix
  - 🍱 :bento: - Add or update assets
  - 🗑️ :wastebasket: - Deprecate code
  - ⚰️ :coffin: - Remove dead code
  - 🧪 :test_tube: - Add failing test
  - 🩹 :adhesive_bandage: - Simple fix for a non-critical issue
  - 🌐 :globe_with_meridians: - Internationalization and localization
  - 💡 :bulb: - Add or update comments in source code
  - 🗃️ :card_file_box: - Database related changes
- Keep the description under 72 characters
- Use imperative mood
- For breaking changes, add 💥 after the emoji

## Context
{}

## Language
{}

## Diff

```diff
{}
```

Generate ONLY the commit message, no explanation:"#,
            context_str, language, diff
        )
    }
}

/// External skill importers
pub mod external {
    use super::*;
    use std::process::Command;

    /// Available external skill sources
    #[derive(Debug, Clone)]
    pub enum ExternalSource {
        /// Claude Code skills directory
        ClaudeCode,
        /// GitHub repository
        GitHub { owner: String, repo: String, path: Option<String> },
        /// GitHub Gist
        Gist { id: String },
        /// Direct URL
        Url { url: String },
    }

    impl std::fmt::Display for ExternalSource {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ExternalSource::ClaudeCode => write!(f, "claude-code"),
                ExternalSource::GitHub { owner, repo, .. } => write!(f, "github:{}/{}", owner, repo),
                ExternalSource::Gist { id } => write!(f, "gist:{}", id),
                ExternalSource::Url { url } => write!(f, "url:{}", url),
            }
        }
    }

    /// Parse an external source string
    /// 
    /// Supported formats:
    /// - `claude-code` - Import from Claude Code skills
    /// - `github:owner/repo` - Import from GitHub repo (looks for .rco/skills/)
    /// - `github:owner/repo/path/to/skill` - Import specific skill from repo
    /// - `gist:abc123` - Import from GitHub Gist
    /// - `https://...` - Import from direct URL
    pub fn parse_source(source: &str) -> Result<ExternalSource> {
        if source == "claude-code" || source == "claude" {
            Ok(ExternalSource::ClaudeCode)
        } else if let Some(github_ref) = source.strip_prefix("github:") {
            // Parse github:owner/repo or github:owner/repo/path
            let parts: Vec<&str> = github_ref.split('/').collect();
            if parts.len() < 2 {
                anyhow::bail!("Invalid GitHub reference. Use format: github:owner/repo or github:owner/repo/path");
            }
            let owner = parts[0].to_string();
            let repo = parts[1].to_string();
            let path = if parts.len() > 2 {
                Some(parts[2..].join("/"))
            } else {
                None
            };
            Ok(ExternalSource::GitHub { owner, repo, path })
        } else if let Some(gist_id) = source.strip_prefix("gist:") {
            Ok(ExternalSource::Gist { id: gist_id.to_string() })
        } else if source.starts_with("http://") || source.starts_with("https://") {
            Ok(ExternalSource::Url { url: source.to_string() })
        } else {
            anyhow::bail!("Unknown source format: {}. Use 'claude-code', 'github:owner/repo', 'gist:id', or a URL", source)
        }
    }

    /// Import skills from Claude Code
    /// 
    /// Claude Code stores skills in ~/.claude/skills/
    pub fn import_from_claude_code(target_dir: &Path) -> Result<Vec<String>> {
        let claude_skills_dir = dirs::home_dir()
            .context("Could not find home directory")?
            .join(".claude")
            .join("skills");

        if !claude_skills_dir.exists() {
            anyhow::bail!("Claude Code skills directory not found at ~/.claude/skills/");
        }

        let mut imported = Vec::new();

        for entry in fs::read_dir(&claude_skills_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let skill_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                // Convert Claude Code skill to rusty-commit format
                let target_skill_dir = target_dir.join(&skill_name);
                
                if target_skill_dir.exists() {
                    tracing::warn!("Skill '{}' already exists, skipping", skill_name);
                    continue;
                }

                fs::create_dir_all(&target_skill_dir)?;
                
                // Convert and copy files
                convert_claude_skill(&path, &target_skill_dir, &skill_name)?;
                
                imported.push(skill_name);
            }
        }

        Ok(imported)
    }

    /// Convert a Claude Code skill to rusty-commit format
    pub fn convert_claude_skill(source: &Path, target: &Path, name: &str) -> Result<()> {
        // Claude Code skills typically have:
        // - README.md or INSTRUCTIONS.md
        // - Various tool definitions
        
        // Create skill.toml
        let description = if source.join("README.md").exists() {
            // Try to extract first line from README
            let readme = fs::read_to_string(source.join("README.md"))?;
            readme.lines().next().unwrap_or("Imported from Claude Code").to_string()
        } else {
            format!("Imported from Claude Code: {}", name)
        };

        let manifest = SkillManifest {
            skill: SkillMeta {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                description,
                author: Some("Imported from Claude Code".to_string()),
                category: SkillCategory::Template,
                tags: vec!["claude-code".to_string(), "imported".to_string()],
            },
            hooks: None,
            config: None,
        };

        fs::write(target.join("skill.toml"), toml::to_string_pretty(&manifest)?)?;

        // Try to find and convert instructions to prompt.md
        let instruction_files = ["INSTRUCTIONS.md", "README.md", "PROMPT.md", "prompt.md"];
        let mut found_instructions = false;
        
        for file in &instruction_files {
            let source_file = source.join(file);
            if source_file.exists() {
                let content = fs::read_to_string(&source_file)?;
                // Convert to rusty-commit prompt format
                let prompt = format!(
                    "# Imported from Claude Code Skill: {}\n\n{}\n\n## Diff\n\n```diff\n{{diff}}\n```\n\n## Context\n\n{{context}}",
                    name,
                    content
                );
                fs::write(target.join("prompt.md"), prompt)?;
                found_instructions = true;
                break;
            }
        }

        if !found_instructions {
            // Create a basic prompt template
            let prompt = format!(
                "# Skill: {}\n\nThis skill was imported from Claude Code.\n\n## Diff\n\n```diff\n{{diff}}\n```\n\n## Context\n\n{{context}}",
                name
            );
            fs::write(target.join("prompt.md"), prompt)?;
        }

        // Copy any additional files (except tool definitions which are Claude-specific)
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_str = file_name.to_string_lossy();
            
            // Skip tool definition files and files we've already handled
            if file_str.ends_with(".json") && file_str.contains("tool") {
                continue; // Claude-specific tool definitions
            }
            if ["skill.toml", "prompt.md", "README.md", "INSTRUCTIONS.md"].contains(&file_str.as_ref()) {
                continue;
            }
            
            // Copy other files
            let target_file = target.join(&file_name);
            if entry.path().is_file() {
                fs::copy(entry.path(), target_file)?;
            }
        }

        Ok(())
    }

    /// Import skills from GitHub
    /// 
    /// Clones the repo temporarily and copies skills from .rco/skills/
    pub fn import_from_github(
        owner: &str,
        repo: &str,
        path: Option<&str>,
        target_dir: &Path,
    ) -> Result<Vec<String>> {
        use std::env;
        
        // Create temp directory
        let temp_dir = env::temp_dir().join(format!("rco-github-import-{}-{}", owner, repo));
        
        // Clean up any existing temp directory
        if temp_dir.exists() {
            let _ = fs::remove_dir_all(&temp_dir);
        }

        // Clone the repository (shallow clone for speed)
        println!("Cloning {}/{}...", owner, repo);
        let status = Command::new("git")
            .args([
                "clone",
                "--depth", "1",
                &format!("https://github.com/{}/{}", owner, repo),
                temp_dir.to_string_lossy().as_ref(),
            ])
            .status()
            .context("Failed to run git clone. Is git installed?")?;

        if !status.success() {
            anyhow::bail!("Failed to clone repository {}/{}", owner, repo);
        }

        // Determine source path
        let source_path = if let Some(p) = path {
            temp_dir.join(p)
        } else {
            temp_dir.join(".rco").join("skills")
        };

        if !source_path.exists() {
            let _ = fs::remove_dir_all(&temp_dir);
            anyhow::bail!("No skills found at {} in {}/{}", source_path.display(), owner, repo);
        }

        // Import skills
        let mut imported = Vec::new();
        
        for entry in fs::read_dir(&source_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let skill_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                let target_skill_dir = target_dir.join(&skill_name);
                
                if target_skill_dir.exists() {
                    tracing::warn!("Skill '{}' already exists, skipping", skill_name);
                    continue;
                }

                // Copy the skill directory
                copy_dir_all(&path, &target_skill_dir)?;
                
                // Update the skill.toml to mark as imported
                let skill_toml = target_skill_dir.join("skill.toml");
                if skill_toml.exists() {
                    if let Ok(content) = fs::read_to_string(&skill_toml) {
                        if let Ok(mut manifest) = toml::from_str::<SkillManifest>(&content) {
                            manifest.skill.tags.push("github".to_string());
                            manifest.skill.tags.push("imported".to_string());
                            let _ = fs::write(&skill_toml, toml::to_string_pretty(&manifest)?);
                        }
                    }
                }
                
                imported.push(skill_name);
            }
        }

        // Clean up temp directory
        let _ = fs::remove_dir_all(&temp_dir);

        Ok(imported)
    }

    /// Import from GitHub Gist
    pub fn import_from_gist(gist_id: &str, target_dir: &Path) -> Result<String> {
        // Fetch gist metadata
        let gist_url = format!("https://api.github.com/gists/{}", gist_id);
        
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&gist_url)
            .header("User-Agent", "rusty-commit")
            .send()
            .context("Failed to fetch gist from GitHub API")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch gist: HTTP {}", response.status());
        }

        let gist_data: serde_json::Value = response.json()
            .context("Failed to parse gist response")?;

        let files = gist_data["files"].as_object()
            .ok_or_else(|| anyhow::anyhow!("Invalid gist data: no files"))?;

        if files.is_empty() {
            anyhow::bail!("Gist contains no files");
        }

        // Use the first file as the skill name
        let (filename, file_data) = files.iter().next().unwrap();
        let skill_name = filename.trim_end_matches(".md").trim_end_matches(".toml");
        
        let target_skill_dir = target_dir.join(skill_name);
        if target_skill_dir.exists() {
            anyhow::bail!("Skill '{}' already exists", skill_name);
        }

        fs::create_dir_all(&target_skill_dir)?;

        // Get file content
        if let Some(content) = file_data["content"].as_str() {
            // Determine if it's a skill.toml or prompt.md
            if filename.ends_with(".toml") {
                fs::write(target_skill_dir.join("skill.toml"), content)?;
            } else {
                // Assume it's a prompt template
                let prompt = format!(
                    "# Imported from Gist: {}\n\n{}\n\n## Diff\n\n```diff\n{{diff}}\n```\n\n## Context\n\n{{context}}",
                    gist_id,
                    content
                );
                fs::write(target_skill_dir.join("prompt.md"), prompt)?;
                
                // Create a basic skill.toml
                let manifest = SkillManifest {
                    skill: SkillMeta {
                        name: skill_name.to_string(),
                        version: "1.0.0".to_string(),
                        description: format!("Imported from Gist: {}", gist_id),
                        author: gist_data["owner"]["login"].as_str().map(|s| s.to_string()),
                        category: SkillCategory::Template,
                        tags: vec!["gist".to_string(), "imported".to_string()],
                    },
                    hooks: None,
                    config: None,
                };
                fs::write(target_skill_dir.join("skill.toml"), toml::to_string_pretty(&manifest)?)?;
            }
        }

        Ok(skill_name.to_string())
    }

    /// Import from a direct URL
    pub fn import_from_url(url: &str, name: Option<&str>, target_dir: &Path) -> Result<String> {
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(url)
            .header("User-Agent", "rusty-commit")
            .send()
            .context("Failed to download from URL")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to download: HTTP {}", response.status());
        }

        let content = response.text()?;
        
        // Determine skill name from URL or provided name
        let skill_name = name.map(|s| s.to_string()).unwrap_or_else(|| {
            url.split('/').last()
                .and_then(|s| s.split('.').next())
                .unwrap_or("imported-skill")
                .to_string()
        });

        let target_skill_dir = target_dir.join(&skill_name);
        if target_skill_dir.exists() {
            anyhow::bail!("Skill '{}' already exists", skill_name);
        }

        fs::create_dir_all(&target_skill_dir)?;

        // Check if content looks like TOML (skill.toml) or Markdown (prompt.md)
        if content.trim().starts_with('[') && content.contains("[skill]") {
            fs::write(target_skill_dir.join("skill.toml"), content)?;
        } else {
            // Assume it's a prompt template
            let prompt = format!(
                "# Imported from URL\n\n{}\n\n## Diff\n\n```diff\n{{diff}}\n```\n\n## Context\n\n{{context}}",
                content
            );
            fs::write(target_skill_dir.join("prompt.md"), prompt)?;
            
            // Create a basic skill.toml
            let manifest = SkillManifest {
                skill: SkillMeta {
                    name: skill_name.clone(),
                    version: "1.0.0".to_string(),
                    description: format!("Imported from {}", url),
                    author: None,
                    category: SkillCategory::Template,
                    tags: vec!["url".to_string(), "imported".to_string()],
                },
                hooks: None,
                config: None,
            };
            fs::write(target_skill_dir.join("skill.toml"), toml::to_string_pretty(&manifest)?)?;
        }

        Ok(skill_name)
    }

    /// Copy directory recursively
    fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)?;
        
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().unwrap();
            let dst_path = dst.join(file_name);
            
            if path.is_dir() {
                copy_dir_all(&path, &dst_path)?;
            } else {
                fs::copy(&path, &dst_path)?;
            }
        }
        
        Ok(())
    }

    /// List available Claude Code skills without importing
    pub fn list_claude_code_skills() -> Result<Vec<(String, String)>> {
        let claude_skills_dir = dirs::home_dir()
            .context("Could not find home directory")?
            .join(".claude")
            .join("skills");

        if !claude_skills_dir.exists() {
            return Ok(Vec::new());
        }

        let mut skills = Vec::new();
        
        for entry in fs::read_dir(&claude_skills_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                // Try to get description from README
                let description = if path.join("README.md").exists() {
                    let readme = fs::read_to_string(path.join("README.md")).unwrap_or_default();
                    readme.lines().next().unwrap_or("No description").to_string()
                } else {
                    "Claude Code skill".to_string()
                };
                
                skills.push((name, description));
            }
        }
        
        Ok(skills)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_category_display() {
        assert_eq!(SkillCategory::Template.to_string(), "template");
        assert_eq!(SkillCategory::Analyzer.to_string(), "analyzer");
        assert_eq!(SkillCategory::Formatter.to_string(), "formatter");
    }

    #[test]
    fn test_manifest_parsing() {
        let toml = r#"
[skill]
name = "test-skill"
version = "1.0.0"
description = "A test skill"
author = "Test Author"
category = "template"
tags = ["test", "example"]

[skill.hooks]
pre_gen = "pre_gen.sh"
post_gen = "post_gen.sh"
"#;

        let manifest: SkillManifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.skill.name, "test-skill");
        assert_eq!(manifest.skill.version, "1.0.0");
        assert!(matches!(manifest.skill.category, SkillCategory::Template));
        assert_eq!(manifest.skill.tags.len(), 2);
    }
}
