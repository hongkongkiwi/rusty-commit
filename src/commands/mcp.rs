use anyhow::Result;
use colored::Colorize;
use rmcp::{
    handler::server::ServerHandler,
    model::*,
    service::{RequestContext, RoleServer},
    ErrorData as McpError, ServiceExt,
};
use std::future::Future;
use tokio::io::{stdin, stdout};

use crate::cli::McpCommand;
use crate::config::Config;
use crate::git;

/// Execute MCP command from CLI
pub async fn execute(cmd: McpCommand) -> Result<()> {
    match cmd.action {
        crate::cli::McpAction::Server { port: _ } => {
            anyhow::bail!(
                "TCP server mode is not yet implemented.\n\
                 Use 'rco mcp stdio' for MCP connections (Cursor, Claude Desktop, etc.).\n\
                 TCP server support is planned for a future release."
            );
        }
        crate::cli::McpAction::Stdio => start_stdio_server().await,
    }
}

/// Start MCP server over stdio using RMCP SDK
async fn start_stdio_server() -> Result<()> {
    eprintln!(
        "{}",
        "🚀 Starting Rusty Commit MCP Server over STDIO"
            .green()
            .bold()
    );
    eprintln!(
        "{}",
        "📡 Ready for MCP client connections (Cursor, Claude Desktop, etc.)".cyan()
    );

    let server = RustyCommitMcpServer::new();
    let transport = (stdin(), stdout());

    // Start the server
    let service = server.serve(transport).await?;

    // Wait for completion
    service.waiting().await?;

    Ok(())
}

/// Rusty Commit MCP Server implementation
#[derive(Clone)]
struct RustyCommitMcpServer;

impl RustyCommitMcpServer {
    fn new() -> Self {
        Self
    }
}

impl ServerHandler for RustyCommitMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::default(),
            server_info: Implementation {
                name: "rustycommit".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            instructions: Some("Rusty Commit MCP Server - Generate AI-powered commit messages for your Git repositories.".to_string()),
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        async move {
            let tools = vec![
                Tool {
                    name: "generate_commit_message".into(),
                    description: Some("Generate AI-powered commit message for staged git changes using Rusty Commit".into()),
                    input_schema: std::sync::Arc::new(
                        serde_json::json!({
                            "type": "object",
                            "properties": {
                                "context": {
                                    "type": "string",
                                    "description": "Additional context for the commit message"
                                },
                                "full_gitmoji": {
                                    "type": "boolean",
                                    "description": "Use full GitMoji specification",
                                    "default": false
                                },
                                "commit_type": {
                                    "type": "string",
                                    "description": "Commit format type (conventional, gitmoji)",
                                    "enum": ["conventional", "gitmoji"],
                                    "default": "conventional"
                                }
                            },
                            "required": []
                        }).as_object().unwrap().clone()
                    ),
                    output_schema: None,
                    annotations: None,
                },
                Tool {
                    name: "show_commit_prompt".into(),
                    description: Some("Show the prompt that would be sent to AI for commit message generation".into()),
                    input_schema: std::sync::Arc::new(
                        serde_json::json!({
                            "type": "object",
                            "properties": {
                                "context": {
                                    "type": "string",
                                    "description": "Additional context for the commit message"
                                },
                                "full_gitmoji": {
                                    "type": "boolean",
                                    "description": "Use full GitMoji specification",
                                    "default": false
                                }
                            },
                            "required": []
                        }).as_object().unwrap().clone()
                    ),
                    output_schema: None,
                    annotations: None,
                },
            ];

            Ok(ListToolsResult {
                tools,
                next_cursor: None,
            })
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        async move {
            match request.name.as_ref() {
                "generate_commit_message" => generate_commit_message_mcp(&request.arguments).await,
                "show_commit_prompt" => show_commit_prompt_mcp(&request.arguments).await,
                _ => Ok(CallToolResult::error(vec![Content::text(format!(
                    "Unknown tool: {}",
                    request.name
                ))])),
            }
        }
    }
}

/// Generate commit message via MCP
async fn generate_commit_message_mcp(
    arguments: &Option<serde_json::Map<String, serde_json::Value>>,
) -> Result<CallToolResult, McpError> {
    // Ensure we're in a git repository
    if let Err(e) = git::assert_git_repo() {
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "❌ Error: Not a git repository: {}",
            e
        ))]));
    }

    // Load configuration
    let mut config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "❌ Configuration error: {}",
                e
            ))]));
        }
    };

    // Apply commitlint rules (log warnings but don't fail)
    if let Err(e) = config.load_with_commitlint() {
        tracing::warn!("Failed to load commitlint config: {}", e);
    }
    if let Err(e) = config.apply_commitlint_rules() {
        tracing::warn!("Failed to apply commitlint rules: {}", e);
    }

    // Get staged diff
    let diff = match git::get_staged_diff() {
        Ok(d) => d,
        Err(e) => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "❌ Git error: {}",
                e
            ))]));
        }
    };

    if diff.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "⚠️  No staged changes found. Please stage your changes with 'git add' first.",
        )]));
    }

    // Extract arguments
    let args = arguments
        .as_ref()
        .map(|map| serde_json::Value::Object(map.clone()))
        .unwrap_or(serde_json::json!({}));
    let context = args["context"].as_str();
    let full_gitmoji = args["full_gitmoji"].as_bool().unwrap_or(false);

    // Override commit type if specified
    if let Some(commit_type) = args["commit_type"].as_str() {
        config.commit_type = Some(commit_type.to_string());
    }

    // Generate commit message
    match generate_commit_message_internal(&config, &diff, context, full_gitmoji).await {
        Ok(message) => {
            let provider_name = config.ai_provider.as_deref().unwrap_or("openai");
            let model_name = config.model.as_deref().unwrap_or("default");

            Ok(CallToolResult::success(vec![Content::text(format!(
                "🤖 **Generated Commit Message:**\n\n```\n{}\n```\n\n**Details:**\n- Provider: {}\n- Model: {}\n- Generated by: Rusty Commit v{}\n\n💡 You can now copy this message and use it in your commit.",
                message,
                provider_name,
                model_name,
                env!("CARGO_PKG_VERSION")
            ))]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "❌ Failed to generate commit message: {}",
            e
        ))])),
    }
}

/// Show commit prompt via MCP
async fn show_commit_prompt_mcp(
    arguments: &Option<serde_json::Map<String, serde_json::Value>>,
) -> Result<CallToolResult, McpError> {
    // Ensure we're in a git repository
    if let Err(e) = git::assert_git_repo() {
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "❌ Error: Not a git repository: {}",
            e
        ))]));
    }

    // Load configuration
    let mut config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "❌ Configuration error: {}",
                e
            ))]));
        }
    };

    // Apply commitlint rules (log warnings but don't fail)
    if let Err(e) = config.load_with_commitlint() {
        tracing::warn!("Failed to load commitlint config: {}", e);
    }
    if let Err(e) = config.apply_commitlint_rules() {
        tracing::warn!("Failed to apply commitlint rules: {}", e);
    }

    // Get staged diff
    let diff = match git::get_staged_diff() {
        Ok(d) => d,
        Err(e) => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "❌ Git error: {}",
                e
            ))]));
        }
    };

    if diff.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "⚠️  No staged changes found. Please stage your changes with 'git add' first.",
        )]));
    }

    // Extract arguments
    let args = arguments
        .as_ref()
        .map(|map| serde_json::Value::Object(map.clone()))
        .unwrap_or(serde_json::json!({}));
    let context = args["context"].as_str();
    let full_gitmoji = args["full_gitmoji"].as_bool().unwrap_or(false);

    // Generate prompt
    let prompt = config.get_effective_prompt(&diff, context, full_gitmoji);
    let provider_name = config.ai_provider.as_deref().unwrap_or("openai");
    let model_name = config.model.as_deref().unwrap_or("default");

    Ok(CallToolResult::success(vec![Content::text(format!(
        "🔍 **AI Prompt Preview:**\n\n```\n{}\n```\n\n**Configuration:**\n- Provider: {}\n- Model: {}\n- Generated by: Rusty Commit v{}\n\n💡 This is the exact prompt that would be sent to the AI model.",
        prompt,
        provider_name,
        model_name,
        env!("CARGO_PKG_VERSION")
    ))]))
}

/// Internal commit message generation (reused from commit.rs logic)
async fn generate_commit_message_internal(
    config: &Config,
    diff: &str,
    context: Option<&str>,
    full_gitmoji: bool,
) -> Result<String> {
    use crate::providers;

    let provider = providers::create_provider(config)?;
    let message = provider
        .generate_commit_message(diff, context, full_gitmoji, config)
        .await?;

    Ok(message)
}
