use anyhow::{Context, Result};
use async_openai::{
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use async_trait::async_trait;

use super::{split_prompt, AIProvider};
use crate::config::accounts::AccountConfig;
use crate::config::Config;
use crate::utils::retry::retry_async;

pub struct OpenAIProvider {
    client: Client<OpenAIConfig>,
    model: String,
}

impl OpenAIProvider {
    pub fn new(config: &Config) -> Result<Self> {
        let api_key = config
            .api_key
            .as_ref()
            .context("OpenAI API key not configured.\nRun: rco config set RCO_API_KEY=<your_key>\nGet your API key from: https://platform.openai.com/api-keys")?;

        let openai_config = OpenAIConfig::new().with_api_key(api_key).with_api_base(
            config
                .api_url
                .as_deref()
                .unwrap_or("https://api.openai.com/v1"),
        );

        let client = Client::with_config(openai_config);
        let model = config.model.as_deref().unwrap_or("gpt-4o-mini").to_string();

        Ok(Self { client, model })
    }

    /// Create provider from account configuration
    #[allow(dead_code)]
    pub fn from_account(account: &AccountConfig, api_key: &str, config: &Config) -> Result<Self> {
        let openai_config = OpenAIConfig::new().with_api_key(api_key).with_api_base(
            account
                .api_url
                .as_deref()
                .or(config.api_url.as_deref())
                .unwrap_or("https://api.openai.com/v1"),
        );

        let client = Client::with_config(openai_config);
        let model = account
            .model
            .as_deref()
            .or(config.model.as_deref())
            .unwrap_or("gpt-4o-mini")
            .to_string();

        Ok(Self { client, model })
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn generate_commit_message(
        &self,
        diff: &str,
        context: Option<&str>,
        full_gitmoji: bool,
        config: &Config,
    ) -> Result<String> {
        let (system_prompt, user_prompt) = split_prompt(diff, context, config, full_gitmoji);

        let messages = vec![
            ChatCompletionRequestSystemMessage::from(system_prompt).into(),
            ChatCompletionRequestUserMessage::from(user_prompt).into(),
        ];

        // Handle model-specific parameters
        let request = if self.model.contains("gpt-5-nano") {
            // GPT-5-nano doesn't support temperature=0, use 1.0 (default)
            CreateChatCompletionRequestArgs::default()
                .model(&self.model)
                .messages(messages)
                .temperature(1.0)
                .max_tokens(config.tokens_max_output.unwrap_or(500) as u16)
                .build()?
        } else {
            // Standard models support temperature=0.7 and max_tokens
            CreateChatCompletionRequestArgs::default()
                .model(&self.model)
                .messages(messages)
                .temperature(0.7)
                .max_tokens(config.tokens_max_output.unwrap_or(500) as u16)
                .build()?
        };

        let response = retry_async(|| async {
            match self.client.chat().create(request.clone()).await {
                Ok(resp) => Ok(resp),
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("401") || error_msg.contains("invalid_api_key") {
                        Err(anyhow::anyhow!("Invalid OpenAI API key. Please check your API key configuration."))
                    } else if error_msg.contains("insufficient_quota") {
                        Err(anyhow::anyhow!("OpenAI API quota exceeded. Please check your billing status."))
                    } else {
                        Err(anyhow::anyhow!(e).context("Failed to generate commit message from OpenAI"))
                    }
                }
            }
        }).await.context("Failed to generate commit message from OpenAI after retries. Please check your internet connection and API configuration.")?;

        let message = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .context("OpenAI returned an empty response. The model may be overloaded - please try again.")?
            .trim()
            .to_string();

        Ok(message)
    }
}

/// OpenAICompatibleProvider - A wrapper that handles OpenAI-compatible providers
/// This struct registers all OpenAI-compatible API providers in the registry
#[allow(dead_code)]
pub struct OpenAICompatibleProvider {
    pub name: &'static str,
    pub aliases: Vec<&'static str>,
    pub default_api_url: &'static str,
    pub default_model: Option<&'static str>,
    pub compatible_providers: std::collections::HashMap<&'static str, &'static str>,
}

impl OpenAICompatibleProvider {
    pub fn new() -> Self {
        let mut compat = std::collections::HashMap::new();

        // Core OpenAI-compatible providers
        compat.insert("deepseek", "https://api.deepseek.com/v1");
        compat.insert("groq", "https://api.groq.com/openai/v1");
        compat.insert("openrouter", "https://openrouter.ai/api/v1");
        compat.insert("together", "https://api.together.ai/v1");
        compat.insert("deepinfra", "https://api.deepinfra.com/v1/openai");
        compat.insert("mistral", "https://api.mistral.ai/v1");
        compat.insert("github-models", "https://models.inference.ai.azure.com");
        compat.insert("fireworks", "https://api.fireworks.ai/v1");
        compat.insert("fireworks-ai", "https://api.fireworks.ai/v1");
        compat.insert("moonshot", "https://api.moonshot.cn/v1");
        compat.insert("moonshot-ai", "https://api.moonshot.cn/v1");
        compat.insert("dashscope", "https://dashscope.console.aliyuncs.com/api/v1");
        compat.insert("alibaba", "https://dashscope.console.aliyuncs.com/api/v1");
        compat.insert("qwen", "https://dashscope.console.aliyuncs.com/api/v1");
        compat.insert(
            "qwen-coder",
            "https://dashscope.console.aliyuncs.com/api/v1",
        );
        compat.insert("codex", "https://api.openai.com/v1");

        // From OpenCode (Enterprise & Cloud)
        compat.insert("cohere", "https://api.cohere.com/v1");
        compat.insert("ai21", "https://api.ai21.com/studio/v1");
        compat.insert("ai21-labs", "https://api.ai21.com/studio/v1");
        compat.insert("upstage", "https://api.upstage.ai/v1/solar");
        compat.insert("solar", "https://api.upstage.ai/v1/solar");

        // From OpenCode (GPU Cloud & Inference)
        compat.insert("nebius", "https://api.studio.nebius.ai/v1");
        compat.insert("ovh", "https://api.ovhcloud.com/v1");
        compat.insert("ovhcloud", "https://api.ovhcloud.com/v1");
        compat.insert("scaleway", "https://api.scaleway.ai/v1");
        compat.insert("friendli", "https://api.friendli.ai/v1");
        compat.insert("baseten", "https://api.baseten.co/v1");
        compat.insert("chutes", "https://api.chutes.ai/v1");
        compat.insert("ionet", "https://api.io.net/v1");
        compat.insert("modelscope", "https://api.modelscope.cn/v1");
        compat.insert("requesty", "https://api.requesty.ai/v1");
        compat.insert("morph", "https://api.morph.so/v1");
        compat.insert("synthetic", "https://api.syntheticai.com/v1");
        compat.insert("nano-gpt", "https://api.nano-gpt.com/v1");
        compat.insert("zenmux", "https://api.zenmux.com/v1");
        compat.insert("v0", "https://api.v0.dev/v1");
        compat.insert("iflowcn", "https://api.iflow.cn/v1");
        compat.insert("venice", "https://api.venice.ai/v1");
        compat.insert("cortecs", "https://api.cortecs.ai/v1");
        compat.insert("kimi-coding", "https://api.moonshot.cn/v1");
        compat.insert("abacus", "https://api.abacus.ai/v1");
        compat.insert("bailing", "https://api.bailing.ai/v1");
        compat.insert("fastrouter", "https://api.fastrouter.ai/v1");
        compat.insert("inference", "https://api.inference.net/v1");
        compat.insert("submodel", "https://api.submodel.ai/v1");
        compat.insert("zai", "https://api.z.ai/v1");
        compat.insert("zai-coding", "https://api.z.ai/v1");
        compat.insert("zhipu-coding", "https://open.bigmodel.cn/api/paas/v4");
        compat.insert("poe", "https://api.poe.com/v1");
        compat.insert("cerebras", "https://api.cerebras.ai/v1");
        compat.insert("lmstudio", "http://localhost:1234/v1");
        compat.insert("sambanova", "https://api.sambanova.ai/v1");
        compat.insert("novita", "https://api.novita.ai/v3/openai");
        compat.insert("predibase", "https://api.predibase.com/v1");
        compat.insert("tensorops", "https://api.tensorops.ai/v1");
        compat.insert("hyperbolic", "https://api.hyperbolic.ai/v1");
        compat.insert("kluster", "https://api.kluster.ai/v1");
        compat.insert("lambda", "https://api.lambda.ai/v1");
        compat.insert("replicate", "https://api.replicate.com/v1");
        compat.insert("targon", "https://api.targon.com/v1");
        compat.insert("corcel", "https://api.corcel.io/v1");
        compat.insert("cybernative", "https://api.cybernative.ai/v1");
        compat.insert("edgen", "https://api.edgen.co/v1");
        compat.insert("gigachat", "https://api.gigachat.ru/v1");
        compat.insert("hydra", "https://api.hydraai.com/v1");
        compat.insert("jina", "https://api.jina.ai/v1");
        compat.insert("lingyi", "https://api.lingyiwanwu.com/v1");
        compat.insert("monica", "https://api.monica.ai/v1");
        compat.insert("pollinations", "https://api.pollinations.ai/v1");
        compat.insert("rawechat", "https://api.rawe.chat/v1");
        compat.insert("shuttleai", "https://api.shuttleai.com/v1");
        compat.insert("teknium", "https://api.teknium.ai/v1");
        compat.insert("theb", "https://api.theb.ai/v1");
        compat.insert("tryleap", "https://api.tryleap.ai/v1");
        compat.insert(
            "workers-ai",
            "https://api.cloudflare.com/client/v4/accounts/{account_id}/ai/v1",
        );

        // China-based providers
        compat.insert("siliconflow", "https://api.siliconflow.cn/v1");
        compat.insert("zhipu", "https://open.bigmodel.cn/api/paas/v4");
        compat.insert("minimax", "https://api.minimax.chat/v1");
        compat.insert("glm", "https://open.bigmodel.cn/api/paas/v4");

        // Additional providers from OpenCommit
        compat.insert("aimlapi", "https://api.aimlapi.com/v1");

        Self {
            name: "openai",
            aliases: vec!["openai"],
            default_api_url: "https://api.openai.com/v1",
            default_model: Some("gpt-4o-mini"),
            compatible_providers: compat,
        }
    }
}

impl Default for OpenAICompatibleProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl super::registry::ProviderBuilder for OpenAICompatibleProvider {
    fn name(&self) -> &'static str {
        self.name
    }

    fn aliases(&self) -> Vec<&'static str> {
        self.aliases.clone()
    }

    fn category(&self) -> super::registry::ProviderCategory {
        super::registry::ProviderCategory::OpenAICompatible
    }

    fn create(&self, config: &Config) -> Result<Box<dyn super::AIProvider>> {
        Ok(Box::new(OpenAIProvider::new(config)?))
    }

    fn requires_api_key(&self) -> bool {
        true
    }

    fn default_model(&self) -> Option<&'static str> {
        self.default_model
    }
}
