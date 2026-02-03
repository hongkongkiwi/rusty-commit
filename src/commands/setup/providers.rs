//! Setup wizard data types and provider definitions

use std::fmt;

/// Provider option for the setup wizard
#[derive(Clone, Copy)]
pub struct ProviderOption {
    pub name: &'static str,
    pub display: &'static str,
    pub default_model: &'static str,
    pub requires_key: bool,
    pub category: ProviderCategory,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ProviderCategory {
    Popular,
    Local,
    Cloud,
    Enterprise,
    Specialized,
}

impl ProviderOption {
    pub fn all() -> Vec<Self> {
        vec![
            // Popular providers
            ProviderOption {
                name: "openai",
                display: "OpenAI (GPT-4o, GPT-4o-mini, GPT-5)",
                default_model: "gpt-4o-mini",
                requires_key: true,
                category: ProviderCategory::Popular,
            },
            ProviderOption {
                name: "anthropic",
                display: "Anthropic (Claude 3.5/4 Sonnet, Haiku, Opus)",
                default_model: "claude-3-5-haiku-20241022",
                requires_key: true,
                category: ProviderCategory::Popular,
            },
            ProviderOption {
                name: "gemini",
                display: "Google Gemini (2.5 Flash, 2.5 Pro)",
                default_model: "gemini-2.5-flash",
                requires_key: true,
                category: ProviderCategory::Popular,
            },
            // Local/Self-hosted
            ProviderOption {
                name: "ollama",
                display: "Ollama (Local models - free, private)",
                default_model: "llama3.2",
                requires_key: false,
                category: ProviderCategory::Local,
            },
            ProviderOption {
                name: "lmstudio",
                display: "LM Studio (Local GUI for LLMs)",
                default_model: "local-model",
                requires_key: false,
                category: ProviderCategory::Local,
            },
            ProviderOption {
                name: "llamacpp",
                display: "llama.cpp (Local inference)",
                default_model: "local-model",
                requires_key: false,
                category: ProviderCategory::Local,
            },
            // Cloud providers - Fast Inference
            ProviderOption {
                name: "groq",
                display: "Groq (Ultra-fast inference)",
                default_model: "llama-3.3-70b-versatile",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "cerebras",
                display: "Cerebras (Fast inference)",
                default_model: "llama-3.3-70b",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "sambanova",
                display: "SambaNova (Fast inference)",
                default_model: "Meta-Llama-3.3-70B-Instruct",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "nebius",
                display: "Nebius (GPU cloud inference)",
                default_model: "meta-llama/Llama-3.3-70B-Instruct",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            // Cloud providers - General
            ProviderOption {
                name: "xai",
                display: "xAI (Grok)",
                default_model: "grok-2",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "deepseek",
                display: "DeepSeek (V3, R1 Reasoner)",
                default_model: "deepseek-chat",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "openrouter",
                display: "OpenRouter (Access 100+ models)",
                default_model: "anthropic/claude-3.5-haiku",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "mistral",
                display: "Mistral AI",
                default_model: "mistral-small-latest",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "perplexity",
                display: "Perplexity AI",
                default_model: "sonar",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "together",
                display: "Together AI",
                default_model: "meta-llama/Llama-3.3-70B-Instruct-Turbo",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "fireworks",
                display: "Fireworks AI",
                default_model: "accounts/fireworks/models/llama-v3p3-70b-instruct",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "replicate",
                display: "Replicate",
                default_model: "meta/meta-llama-3-70b-instruct",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            // Enterprise
            ProviderOption {
                name: "azure",
                display: "Azure OpenAI",
                default_model: "gpt-4o",
                requires_key: true,
                category: ProviderCategory::Enterprise,
            },
            ProviderOption {
                name: "bedrock",
                display: "AWS Bedrock",
                default_model: "anthropic.claude-3-haiku-20240307-v1:0",
                requires_key: true,
                category: ProviderCategory::Enterprise,
            },
            ProviderOption {
                name: "vertex",
                display: "Google Vertex AI",
                default_model: "gemini-2.5-flash-001",
                requires_key: true,
                category: ProviderCategory::Enterprise,
            },
            ProviderOption {
                name: "cohere",
                display: "Cohere",
                default_model: "command-r",
                requires_key: true,
                category: ProviderCategory::Enterprise,
            },
            ProviderOption {
                name: "ai21",
                display: "AI21 Labs (Jamba)",
                default_model: "jamba-1.5-mini",
                requires_key: true,
                category: ProviderCategory::Enterprise,
            },
            // China-based Providers
            ProviderOption {
                name: "siliconflow",
                display: "SiliconFlow (China)",
                default_model: "deepseek-ai/DeepSeek-V3",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "zhipu",
                display: "Zhipu AI / ChatGLM (China)",
                default_model: "glm-4-flash",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "moonshot",
                display: "Moonshot AI / Kimi (China)",
                default_model: "moonshot-v1-8k",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            // Specialized Providers
            ProviderOption {
                name: "jina",
                display: "Jina AI (Embeddings & LLMs)",
                default_model: "jina-embeddings-v3",
                requires_key: true,
                category: ProviderCategory::Specialized,
            },
            ProviderOption {
                name: "helicone",
                display: "Helicone (LLM Observability)",
                default_model: "gpt-4o-mini",
                requires_key: true,
                category: ProviderCategory::Specialized,
            },
        ]
    }

    #[allow(dead_code)]
    pub fn by_name(name: &str) -> Option<Self> {
        Self::all().into_iter().find(|p| p.name == name)
    }
}

impl fmt::Display for ProviderCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderCategory::Popular => write!(f, "Popular"),
            ProviderCategory::Local => write!(f, "Local"),
            ProviderCategory::Cloud => write!(f, "Cloud"),
            ProviderCategory::Enterprise => write!(f, "Enterprise"),
            ProviderCategory::Specialized => write!(f, "Specialized"),
        }
    }
}

/// Commit format options
#[derive(Clone, Copy)]
pub enum CommitFormat {
    Conventional,
    Gitmoji,
    Simple,
}

impl CommitFormat {
    pub fn display(&self) -> &'static str {
        match self {
            CommitFormat::Conventional => "Conventional Commits (feat:, fix:, docs:, etc.)",
            CommitFormat::Gitmoji => "GitMoji (✨ feat:, 🐛 fix:, 📝 docs:, etc.)",
            CommitFormat::Simple => "Simple (no prefix)",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CommitFormat::Conventional => "conventional",
            CommitFormat::Gitmoji => "gitmoji",
            CommitFormat::Simple => "simple",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            CommitFormat::Conventional,
            CommitFormat::Gitmoji,
            CommitFormat::Simple,
        ]
    }
}

/// Language option for commit message output
#[derive(Clone, Copy)]
pub struct LanguageOption {
    pub code: &'static str,
    pub display: &'static str,
}

impl LanguageOption {
    pub fn all() -> Vec<Self> {
        vec![
            LanguageOption {
                code: "en",
                display: "English",
            },
            LanguageOption {
                code: "zh",
                display: "Chinese (中文)",
            },
            LanguageOption {
                code: "es",
                display: "Spanish (Español)",
            },
            LanguageOption {
                code: "fr",
                display: "French (Français)",
            },
            LanguageOption {
                code: "de",
                display: "German (Deutsch)",
            },
            LanguageOption {
                code: "ja",
                display: "Japanese (日本語)",
            },
            LanguageOption {
                code: "ko",
                display: "Korean (한국어)",
            },
            LanguageOption {
                code: "ru",
                display: "Russian (Русский)",
            },
            LanguageOption {
                code: "pt",
                display: "Portuguese (Português)",
            },
            LanguageOption {
                code: "it",
                display: "Italian (Italiano)",
            },
            LanguageOption {
                code: "other",
                display: "Other (specify)",
            },
        ]
    }
}
