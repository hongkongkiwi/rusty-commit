// AI Provider modules - conditionally compiled based on features
#[cfg(feature = "anthropic")]
pub mod anthropic;
#[cfg(feature = "azure")]
pub mod azure;
#[cfg(feature = "bedrock")]
pub mod bedrock;
#[cfg(feature = "gemini")]
pub mod gemini;
#[cfg(feature = "huggingface")]
pub mod huggingface;
#[cfg(feature = "flowise")]
pub mod flowise;
#[cfg(feature = "mlx")]
pub mod mlx;
#[cfg(feature = "nvidia")]
pub mod nvidia;
#[cfg(feature = "ollama")]
pub mod ollama;
#[cfg(feature = "openai")]
pub mod openai;
#[cfg(feature = "perplexity")]
pub mod perplexity;
#[cfg(feature = "vertex")]
pub mod vertex;
#[cfg(feature = "xai")]
pub mod xai;

// Provider registry for extensible provider management
pub mod registry;

use crate::config::accounts::AccountConfig;
use crate::config::Config;
use anyhow::{Context, Result};
use async_trait::async_trait;
use once_cell::sync::Lazy;

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn generate_commit_message(
        &self,
        diff: &str,
        context: Option<&str>,
        full_gitmoji: bool,
        config: &Config,
    ) -> Result<String>;

    /// Generate multiple commit message variations
    async fn generate_commit_messages(
        &self,
        diff: &str,
        context: Option<&str>,
        full_gitmoji: bool,
        config: &Config,
        count: u8,
    ) -> Result<Vec<String>> {
        use futures::stream::StreamExt;

        if count <= 1 {
            // For single message, no parallelism needed
            match self
                .generate_commit_message(diff, context, full_gitmoji, config)
                .await
            {
                Ok(msg) => Ok(vec![msg]),
                Err(e) => {
                    tracing::warn!("Failed to generate message: {}", e);
                    Ok(vec![])
                }
            }
        } else {
            // Generate messages in parallel using FuturesUnordered
            let futures = (0..count)
                .map(|_| self.generate_commit_message(diff, context, full_gitmoji, config));
            let mut stream = futures::stream::FuturesUnordered::from_iter(futures);

            let mut messages = Vec::with_capacity(count as usize);
            while let Some(result) = stream.next().await {
                match result {
                    Ok(msg) => messages.push(msg),
                    Err(e) => tracing::warn!("Failed to generate message: {}", e),
                }
            }
            Ok(messages)
        }
    }

    /// Generate a PR description from commits
    #[cfg(any(feature = "openai", feature = "xai"))]
    async fn generate_pr_description(
        &self,
        commits: &[String],
        diff: &str,
        config: &Config,
    ) -> Result<String> {
        let commits_text = commits.join("\n");
        let prompt = format!(
            "Generate a professional pull request description based on the following commits:\n\n{}\n\nDiff:\n{}\n\nFormat the output as:\n## Summary\n## Changes\n## Testing\n## Breaking Changes\n\nKeep it concise and informative.",
            commits_text, diff
        );

        let messages = vec![
            async_openai::types::chat::ChatCompletionRequestSystemMessage::from(
                "You are an expert at writing pull request descriptions.",
            )
            .into(),
            async_openai::types::chat::ChatCompletionRequestUserMessage::from(prompt).into(),
        ];

        let request = async_openai::types::chat::CreateChatCompletionRequestArgs::default()
            .model(
                config
                    .model
                    .clone()
                    .unwrap_or_else(|| "gpt-3.5-turbo".to_string()),
            )
            .messages(messages)
            .temperature(0.7)
            .max_tokens(config.tokens_max_output.unwrap_or(1000) as u16)
            .build()?;

        // Create a new client for this request
        let api_key = config
            .api_key
            .as_ref()
            .context("API key not configured. Run: rco config set RCO_API_KEY=<your_key>")?;
        let api_url = config
            .api_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");

        let openai_config = async_openai::config::OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(api_url);

        let client = async_openai::Client::with_config(openai_config);

        let response = client.chat().create(request).await?;

        let message = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .context("AI returned an empty response")?
            .trim()
            .to_string();

        Ok(message)
    }

    /// Generate a PR description - stub when OpenAI/xAI features are disabled
    #[cfg(not(any(feature = "openai", feature = "xai")))]
    async fn generate_pr_description(
        &self,
        _commits: &[String],
        _diff: &str,
        _config: &Config,
    ) -> Result<String> {
        anyhow::bail!(
            "PR description generation requires the 'openai' or 'xai' feature to be enabled"
        );
    }
}

/// Global provider registry - automatically populated based on enabled features
pub static PROVIDER_REGISTRY: Lazy<registry::ProviderRegistry> = Lazy::new(|| {
    let reg = registry::ProviderRegistry::new();

    // Register OpenAI-compatible providers (require openai feature)
    #[cfg(feature = "openai")]
    {
        let _ = reg.register(Box::new(openai::OpenAICompatibleProvider::new()));
    }

    // Register dedicated providers
    #[cfg(feature = "anthropic")]
    {
        let _ = reg.register(Box::new(anthropic::AnthropicProviderBuilder));
    }

    #[cfg(feature = "ollama")]
    {
        let _ = reg.register(Box::new(ollama::OllamaProviderBuilder));
    }

    #[cfg(feature = "gemini")]
    {
        let _ = reg.register(Box::new(gemini::GeminiProviderBuilder));
    }

    #[cfg(feature = "azure")]
    {
        let _ = reg.register(Box::new(azure::AzureProviderBuilder));
    }

    #[cfg(feature = "perplexity")]
    {
        let _ = reg.register(Box::new(perplexity::PerplexityProviderBuilder));
    }

    #[cfg(feature = "xai")]
    {
        let _ = reg.register(Box::new(xai::XAIProviderBuilder));
    }

    #[cfg(feature = "huggingface")]
    {
        let _ = reg.register(Box::new(huggingface::HuggingFaceProviderBuilder));
    }

    #[cfg(feature = "bedrock")]
    {
        let _ = reg.register(Box::new(bedrock::BedrockProviderBuilder));
    }

    #[cfg(feature = "vertex")]
    {
        let _ = reg.register(Box::new(vertex::VertexProviderBuilder));
    }

    #[cfg(feature = "mlx")]
    {
        let _ = reg.register(Box::new(mlx::MlxProviderBuilder));
    }

    #[cfg(feature = "nvidia")]
    {
        let _ = reg.register(Box::new(nvidia::NvidiaProviderBuilder));
    }

    #[cfg(feature = "flowise")]
    {
        let _ = reg.register(Box::new(flowise::FlowiseProviderBuilder));
    }

    reg
});

/// Create an AI provider instance from configuration
pub fn create_provider(config: &Config) -> Result<Box<dyn AIProvider>> {
    let provider_name = config.ai_provider.as_deref().unwrap_or("openai");

    // Try to create from registry
    if let Some(provider) = PROVIDER_REGISTRY.create(provider_name, config)? {
        return Ok(provider);
    }

    // Provider not found - build error message with available providers
    let available: Vec<String> = PROVIDER_REGISTRY
        .all()
        .unwrap_or_default()
        .iter()
        .map(|e| {
            let aliases = if e.aliases.is_empty() {
                String::new()
            } else {
                format!(" ({})", e.aliases.join(", "))
            };
            format!("- {}{}", e.name, aliases)
        })
        .chain(std::iter::once(format!(
            "- {} OpenAI-compatible providers (deepseek, groq, openrouter, etc.)",
            PROVIDER_REGISTRY
                .by_category(registry::ProviderCategory::OpenAICompatible)
                .map_or(0, |v| v.len())
        )))
        .filter(|s| !s.contains("0 OpenAI-compatible"))
        .collect();

    if available.is_empty() {
        anyhow::bail!(
            "No AI provider features enabled. Please enable at least one provider feature:\n\
             --features openai,anthropic,ollama,gemini,azure,perplexity,xai,huggingface,bedrock,vertex"
        );
    }

    anyhow::bail!(
        "Unsupported or disabled AI provider: {}\n\n\
         Available providers (based on enabled features):\n{}\n\n\
         Set with: rco config set RCO_AI_PROVIDER=<provider_name>",
        provider_name,
        available.join("\n")
    )
}

#[allow(dead_code)]
/// Get list of all available provider names
pub fn available_providers() -> Vec<&'static str> {
    let mut providers = PROVIDER_REGISTRY
        .all()
        .unwrap_or_default()
        .iter()
        .flat_map(|e| std::iter::once(e.name).chain(e.aliases.iter().copied()))
        .collect::<Vec<_>>();

    #[cfg(feature = "openai")]
    {
        providers.extend_from_slice(&[
            // Major providers
            "deepseek",
            "groq",
            "openrouter",
            "together",
            "deepinfra",
            "mistral",
            "github-models",
            "fireworks",
            "moonshot",
            "dashscope",
            // From OpenCommit
            "aimlapi",
            // From OpenCode
            "cohere",
            "ai21",
            "cloudflare",
            "siliconflow",
            "zhipu",
            "minimax",
            "upstage",
            "nebius",
            "ovh",
            "scaleway",
            "friendli",
            "baseten",
            "chutes",
            "ionet",
            "modelscope",
            "requesty",
            "morph",
            "synthetic",
            "nano-gpt",
            "zenmux",
            "v0",
            "iflowcn",
            "venice",
            "cortecs",
            "kimi-coding",
            "abacus",
            "bailing",
            "fastrouter",
            "inference",
            "submodel",
            "zai",
            "zai-coding",
            "zhipu-coding",
            "poe",
            "cerebras",
            "lmstudio",
            "sambanova",
            "novita",
            "predibase",
            "tensorops",
            "hyperbolic",
            "kluster",
            "lambda",
            "replicate",
            "targon",
            "corcel",
            "cybernative",
            "edgen",
            "gigachat",
            "hydra",
            "jina",
            "lingyi",
            "monica",
            "pollinations",
            "rawechat",
            "shuttleai",
            "teknium",
            "theb",
            "tryleap",
            "workers-ai",
        ]);
    }

    providers
}

/// Get provider info for display
#[allow(dead_code)]
pub fn provider_info(provider: &str) -> Option<String> {
    PROVIDER_REGISTRY.get(provider).map(|e| {
        let aliases = if e.aliases.is_empty() {
            String::new()
        } else {
            format!(" (aliases: {})", e.aliases.join(", "))
        };
        let model = e
            .default_model
            .map(|m| format!(", default model: {}", m))
            .unwrap_or_default();
        format!("{}{}{}", e.name, aliases, model)
    })
}

/// Split the prompt into system and user parts for providers that support it
pub fn split_prompt(
    diff: &str,
    context: Option<&str>,
    config: &Config,
    full_gitmoji: bool,
) -> (String, String) {
    let system_prompt = build_system_prompt(config, full_gitmoji);
    let user_prompt = build_user_prompt(diff, context, full_gitmoji, config);
    (system_prompt, user_prompt)
}

/// Build the system prompt part (role definition, rules)
fn build_system_prompt(config: &Config, full_gitmoji: bool) -> String {
    let mut prompt = String::new();

    prompt.push_str("You are an expert at writing clear, concise git commit messages.\n\n");

    // Core constraints
    prompt.push_str("OUTPUT RULES:\n");
    prompt.push_str("- Return ONLY the commit message, with no additional explanation, markdown formatting, or code blocks\n");
    prompt.push_str("- Do not include any reasoning, thinking, analysis, <thinking> tags, or XML-like tags in your response\n");
    prompt.push_str("- Never explain your choices or provide commentary\n");
    prompt.push_str("- If you cannot generate a meaningful commit message, return \"chore: update\"\n\n");

    // Add style guidance from history if enabled
    if config.learn_from_history.unwrap_or(false) {
        if let Some(style_guidance) = get_style_guidance(config) {
            prompt.push_str("REPO STYLE (learned from commit history):\n");
            prompt.push_str(&style_guidance);
            prompt.push('\n');
        }
    }

    // Add locale if specified
    if let Some(locale) = &config.language {
        prompt.push_str(&format!(
            "- Generate the commit message in {} language\n",
            locale
        ));
    }

    // Add commit type preference
    let commit_type = config.commit_type.as_deref().unwrap_or("conventional");
    match commit_type {
        "conventional" => {
            prompt.push_str("- Use conventional commit format: <type>(<scope>): <description>\n");
            prompt.push_str(
                "- Types: feat, fix, docs, style, refactor, perf, test, build, ci, chore\n",
            );
            if config.omit_scope.unwrap_or(false) {
                prompt.push_str("- Omit the scope, use format: <type>: <description>\n");
            }
        }
        "gitmoji" => {
            if full_gitmoji {
                prompt.push_str("- Use GitMoji format with full emoji specification from https://gitmoji.dev/\n");
                prompt.push_str("- Common emojis: ✨(feat), 🐛(fix), 📝(docs), 🚀(deploy), ♻️(refactor), ✅(test), 🔧(chore), ⚡(perf), 🎨(style), 📦(build), 👷(ci)\n");
                prompt.push_str("- For breaking changes, add 💥 after the type\n");
            } else {
                prompt.push_str("- Use GitMoji format: <emoji> <type>: <description>\n");
                prompt.push_str("- Common emojis: 🐛(fix), ✨(feat), 📝(docs), 🚀(deploy), ✅(test), ♻️(refactor), 🔧(chore), ⚡(perf), 🎨(style), 📦(build), 👷(ci)\n");
            }
        }
        _ => {}
    }

    // Description requirements
    let max_length = config.description_max_length.unwrap_or(100);
    prompt.push_str(&format!(
        "- Keep the description under {} characters\n",
        max_length
    ));

    if config.description_capitalize.unwrap_or(true) {
        prompt.push_str("- Capitalize the first letter of the description\n");
    }

    if !config.description_add_period.unwrap_or(false) {
        prompt.push_str("- Do not end the description with a period\n");
    }

    // Add commit body guidance if enabled
    if config.enable_commit_body.unwrap_or(false) {
        prompt.push_str("\nCOMMIT BODY (optional):\n");
        prompt.push_str("- Add a blank line after the description, then explain WHY the change was made\n");
        prompt.push_str("- Use bullet points for multiple changes\n");
        prompt.push_str("- Wrap body text at 72 characters\n");
        prompt.push_str("- Focus on motivation and context, not what changed (that's in the diff)\n");
    }

    prompt
}

/// Get style guidance from commit history analysis
fn get_style_guidance(config: &Config) -> Option<String> {
    use crate::git;
    use crate::utils::commit_style::CommitStyleProfile;

    // Get cached style profile or analyze fresh
    if let Some(cached) = &config.style_profile {
        // Use cached profile if available
        return Some(cached.clone());
    }

    // Analyze recent commits - default now 50 for better learning
    let count = config.history_commits_count.unwrap_or(50);

    match git::get_recent_commit_messages(count) {
        Ok(commits) => {
            if commits.is_empty() {
                return None;
            }

            let profile = CommitStyleProfile::analyze_from_commits(&commits);

            // Only use profile if we have enough confident data (at least 10 commits with patterns)
            // Increased from 5 to 10 for better confidence
            if profile.is_empty() || commits.len() < 10 {
                return None;
            }

            Some(profile.to_prompt_guidance())
        }
        Err(e) => {
            tracing::warn!("Failed to get commit history for style analysis: {}", e);
            None
        }
    }
}

/// Build the user prompt part (actual task + diff)
fn build_user_prompt(diff: &str, context: Option<&str>, _full_gitmoji: bool, _config: &Config) -> String {
    let mut prompt = String::new();

    // Add project context if available
    if let Some(project_context) = get_project_context() {
        prompt.push_str(&format!("Project Context: {}\n\n", project_context));
    }

    // Add file type summary with detailed extension info
    let file_summary = extract_file_summary(diff);
    if !file_summary.is_empty() {
        prompt.push_str(&format!("Files Changed: {}\n\n", file_summary));
    }

    // Add chunk indicator with more detail if diff was chunked
    if diff.contains("---CHUNK") {
        let chunk_count = diff.matches("---CHUNK").count();
        if chunk_count > 1 {
            prompt.push_str(&format!(
                "Note: This diff was split into {} chunks due to size. Focus on the overall purpose of the changes across all chunks.\n\n",
                chunk_count
            ));
        } else {
            prompt.push_str("Note: The diff was split into chunks due to size. Focus on the overall purpose of the changes.\n\n");
        }
    }

    // Add context if provided
    if let Some(ctx) = context {
        prompt.push_str(&format!("Additional context: {}\n\n", ctx));
    }

    prompt.push_str("Generate a commit message for the following git diff:\n");
    prompt.push_str("```diff\n");
    prompt.push_str(diff);
    prompt.push_str("\n```\n");

    // Add reminder about output format
    prompt.push_str("\nRemember: Return ONLY the commit message, no explanations or markdown.");

    prompt
}

/// Extract file type summary from diff
fn extract_file_summary(diff: &str) -> String {
    let mut files: Vec<String> = Vec::new();
    let mut extensions: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut file_types: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    
    for line in diff.lines() {
        if line.starts_with("+++ b/") {
            let path = line.strip_prefix("+++ b/").unwrap_or(line);
            if path != "/dev/null" {
                files.push(path.to_string());
                // Extract extension and categorize
                if let Some(ext) = std::path::Path::new(path).extension() {
                    if let Some(ext_str) = ext.to_str() {
                        let ext_lower = ext_str.to_lowercase();
                        extensions.insert(ext_lower.clone());
                        
                        // Categorize file type
                        let category = categorize_file_type(&ext_lower);
                        *file_types.entry(category).or_insert(0) += 1;
                    }
                } else {
                    // No extension - might be a config file or script
                    if path.contains("Makefile") || path.contains("Dockerfile") || path.contains("LICENSE") {
                        *file_types.entry("config".to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
    }
    
    if files.is_empty() {
        return String::new();
    }
    
    // Build summary
    let mut summary = format!("{} file(s)", files.len());
    
    // Add file type categories
    if !file_types.is_empty() {
        let mut type_list: Vec<_> = file_types.into_iter().collect();
        type_list.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending
        
        let type_str: Vec<_> = type_list.iter()
            .map(|(t, c)| format!("{} {}", c, t))
            .collect();
        summary.push_str(&format!(" ({})", type_str.join(", ")));
    }
    
    // Add extension info if not too many
    if !extensions.is_empty() && extensions.len() <= 5 {
        let ext_list: Vec<_> = extensions.into_iter().collect();
        summary.push_str(&format!(" [.{}]", ext_list.join(", .")));
    }
    
    // Add first few file names if small number
    if files.len() <= 3 {
        summary.push_str(&format!(": {}", files.join(", ")));
    }
    
    summary
}

/// Categorize file extension into a type
fn categorize_file_type(ext: &str) -> String {
    match ext {
        // Programming languages
        "rs" => "Rust",
        "py" => "Python",
        "js" => "JavaScript",
        "ts" => "TypeScript",
        "jsx" | "tsx" => "React",
        "go" => "Go",
        "java" => "Java",
        "kt" => "Kotlin",
        "swift" => "Swift",
        "c" | "cpp" | "cc" | "h" | "hpp" => "C/C++",
        "rb" => "Ruby",
        "php" => "PHP",
        "cs" => "C#",
        "scala" => "Scala",
        "r" => "R",
        "m" => "Objective-C",
        "lua" => "Lua",
        "pl" => "Perl",
        
        // Web
        "html" | "htm" => "HTML",
        "css" | "scss" | "sass" | "less" => "CSS",
        "vue" => "Vue",
        "svelte" => "Svelte",
        
        // Data/Config
        "json" => "JSON",
        "yaml" | "yml" => "YAML",
        "toml" => "TOML",
        "xml" => "XML",
        "csv" => "CSV",
        "sql" => "SQL",
        
        // Documentation
        "md" | "markdown" => "Markdown",
        "rst" => "reStructuredText",
        "txt" => "Text",
        
        // Build/Config
        "sh" | "bash" | "zsh" | "fish" => "Shell",
        "ps1" => "PowerShell",
        "bat" | "cmd" => "Batch",
        "dockerfile" => "Docker",
        "makefile" | "mk" => "Make",
        "cmake" => "CMake",
        
        // Other
        _ => "Other",
    }.to_string()
}

/// Get project context from .rco/context.txt or README
fn get_project_context() -> Option<String> {
    use std::path::Path;
    
    // Try .rco/context.txt first
    if let Ok(repo_root) = crate::git::get_repo_root() {
        let context_path = Path::new(&repo_root).join(".rco").join("context.txt");
        if context_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&context_path) {
                let trimmed = content.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
        
        // Try README.md - extract first paragraph
        let readme_path = Path::new(&repo_root).join("README.md");
        if readme_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&readme_path) {
                // Find first non-empty line that's not a header
                for line in content.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with('#') {
                        // Return first sentence or up to 100 chars
                        let context = if let Some(idx) = trimmed.find('.') {
                            trimmed[..idx + 1].to_string()
                        } else {
                            trimmed.chars().take(100).collect()
                        };
                        if !context.is_empty() {
                            return Some(context);
                        }
                    }
                }
            }
        }
        
        // Try Cargo.toml for Rust projects
        let cargo_path = Path::new(&repo_root).join("Cargo.toml");
        if cargo_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_path) {
                // Extract description from Cargo.toml
                let mut in_package = false;
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed == "[package]" {
                        in_package = true;
                    } else if trimmed.starts_with('[') && trimmed != "[package]" {
                        in_package = false;
                    } else if in_package && trimmed.starts_with("description") {
                        if let Some(idx) = trimmed.find('=') {
                            let desc = trimmed[idx+1..].trim().trim_matches('"');
                            if !desc.is_empty() {
                                return Some(format!("Rust project: {}", desc));
                            }
                        }
                    }
                }
            }
        }
        
        // Try package.json for Node projects
        let package_path = Path::new(&repo_root).join("package.json");
        if package_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&package_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(desc) = json.get("description").and_then(|d| d.as_str()) {
                        if !desc.is_empty() {
                            return Some(format!("Node.js project: {}", desc));
                        }
                    }
                }
            }
        }
    }
    
    None
}

/// Build the combined prompt for providers without system message support
pub fn build_prompt(
    diff: &str,
    context: Option<&str>,
    config: &Config,
    full_gitmoji: bool,
) -> String {
    let (system, user) = split_prompt(diff, context, config, full_gitmoji);
    format!("{}\n\n---\n\n{}", system, user)
}

/// Create an AI provider from an account configuration
#[allow(dead_code)]
pub fn create_provider_for_account(
    account: &AccountConfig,
    config: &Config,
) -> Result<Box<dyn AIProvider>> {
    use crate::auth::token_storage;
    use crate::config::secure_storage;

    let provider = account.provider.to_lowercase();

    // Extract credentials from the account's auth method
    let credentials = match &account.auth {
        crate::config::accounts::AuthMethod::ApiKey { key_id } => {
            // Get API key from secure storage using the account's key_id
            token_storage::get_api_key_for_account(key_id)?
                .or_else(|| secure_storage::get_secret(key_id).ok().flatten())
        }
        crate::config::accounts::AuthMethod::OAuth {
            provider: _oauth_provider,
            account_id,
        } => {
            // Get OAuth access token from secure storage
            token_storage::get_tokens_for_account(account_id)?.map(|t| t.access_token)
        }
        crate::config::accounts::AuthMethod::EnvVar { name } => std::env::var(name).ok(),
        crate::config::accounts::AuthMethod::Bearer { token_id } => {
            // Get bearer token from secure storage
            token_storage::get_bearer_token_for_account(token_id)?
                .or_else(|| secure_storage::get_secret(token_id).ok().flatten())
        }
    };

    match provider.as_str() {
        #[cfg(feature = "openai")]
        "openai" | "codex" => {
            if let Some(key) = credentials.as_ref() {
                Ok(Box::new(openai::OpenAIProvider::from_account(
                    account, key, config,
                )?))
            } else {
                Ok(Box::new(openai::OpenAIProvider::new(config)?))
            }
        }
        #[cfg(feature = "anthropic")]
        "anthropic" | "claude" | "claude-code" => {
            if let Some(key) = credentials.as_ref() {
                Ok(Box::new(anthropic::AnthropicProvider::from_account(
                    account, key, config,
                )?))
            } else {
                Ok(Box::new(anthropic::AnthropicProvider::new(config)?))
            }
        }
        #[cfg(feature = "ollama")]
        "ollama" => {
            if let Some(key) = credentials.as_ref() {
                Ok(Box::new(ollama::OllamaProvider::from_account(
                    account, key, config,
                )?))
            } else {
                Ok(Box::new(ollama::OllamaProvider::new(config)?))
            }
        }
        #[cfg(feature = "gemini")]
        "gemini" => {
            if let Some(key) = credentials.as_ref() {
                Ok(Box::new(gemini::GeminiProvider::from_account(
                    account, key, config,
                )?))
            } else {
                Ok(Box::new(gemini::GeminiProvider::new(config)?))
            }
        }
        #[cfg(feature = "azure")]
        "azure" | "azure-openai" => {
            if let Some(key) = credentials.as_ref() {
                Ok(Box::new(azure::AzureProvider::from_account(
                    account, key, config,
                )?))
            } else {
                Ok(Box::new(azure::AzureProvider::new(config)?))
            }
        }
        #[cfg(feature = "perplexity")]
        "perplexity" => {
            if let Some(key) = credentials.as_ref() {
                Ok(Box::new(perplexity::PerplexityProvider::from_account(
                    account, key, config,
                )?))
            } else {
                Ok(Box::new(perplexity::PerplexityProvider::new(config)?))
            }
        }
        #[cfg(feature = "xai")]
        "xai" | "grok" | "x-ai" => {
            if let Some(key) = credentials.as_ref() {
                Ok(Box::new(xai::XAIProvider::from_account(
                    account, key, config,
                )?))
            } else {
                Ok(Box::new(xai::XAIProvider::new(config)?))
            }
        }
        #[cfg(feature = "huggingface")]
        "huggingface" | "hf" => {
            if let Some(key) = credentials.as_ref() {
                Ok(Box::new(huggingface::HuggingFaceProvider::from_account(
                    account, key, config,
                )?))
            } else {
                Ok(Box::new(huggingface::HuggingFaceProvider::new(config)?))
            }
        }
        #[cfg(feature = "bedrock")]
        "bedrock" | "aws-bedrock" | "amazon-bedrock" => Ok(Box::new(
            bedrock::BedrockProvider::from_account(account, "", config)?,
        )),
        #[cfg(feature = "vertex")]
        "vertex" | "vertex-ai" | "google-vertex" | "gcp-vertex" => Ok(Box::new(
            vertex::VertexProvider::from_account(account, "", config)?,
        )),
        #[cfg(feature = "mlx")]
        "mlx" | "mlx-lm" | "apple-mlx" => {
            if let Some(_key) = credentials.as_ref() {
                Ok(Box::new(mlx::MlxProvider::from_account(
                    account, "", config,
                )?))
            } else {
                Ok(Box::new(mlx::MlxProvider::new(config)?))
            }
        }
        #[cfg(feature = "nvidia")]
        "nvidia" | "nvidia-nim" | "nim" | "nvidia-ai" => {
            if let Some(key) = credentials.as_ref() {
                Ok(Box::new(nvidia::NvidiaProvider::from_account(
                    account, key, config,
                )?))
            } else {
                Ok(Box::new(nvidia::NvidiaProvider::new(config)?))
            }
        }
        #[cfg(feature = "flowise")]
        "flowise" | "flowise-ai" => {
            if let Some(_key) = credentials.as_ref() {
                Ok(Box::new(flowise::FlowiseProvider::from_account(
                    account, "", config,
                )?))
            } else {
                Ok(Box::new(flowise::FlowiseProvider::new(config)?))
            }
        }
        _ => {
            anyhow::bail!(
                "Unsupported AI provider for account: {}\n\n\
                 Account provider: {}\n\
                 Supported providers: openai, anthropic, ollama, gemini, azure, perplexity, xai, huggingface, bedrock, vertex",
                account.alias,
                provider
            );
        }
    }
}
