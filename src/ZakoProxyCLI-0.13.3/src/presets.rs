//! Built-in provider presets for the "add account" flow.
//!
//! The catalog is deliberately declarative: every entry only states what a user would
//! otherwise have to type by hand (base URL, how the credential travels, which protocols
//! the vendor speaks, a seed model list). Nothing here reaches the network, and an entry
//! is never trusted over what the operator types — the dialog prefills and can be edited.
//!
//! Two families are covered:
//!
//! * **Official API** — vendors publishing a documented OpenAI- or Anthropic-compatible
//!   endpoint (OpenAI, Anthropic, DeepSeek, Kimi, GLM, Qwen, Gemini, …). Users paste a key.
//! * **Self-hosted / local runtimes** — Ollama, LM Studio, vLLM, and generic relays.
//!
//! Desktop clients whose login state is reverse-engineered (the Agent2API family) are
//! intentionally **not** shipped with a fabricated base URL: they are offered as a
//! "custom endpoint" preset with `endpoint: null`, so the operator supplies the URL they
//! actually use instead of inheriting a guess that would fail at request time.

use serde::Serialize;

/// How the stored credential is attached to an upstream request.
#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthStyle {
    /// `Authorization: Bearer <secret>` — the OpenAI convention.
    Bearer,
    /// `x-api-key: <secret>` plus `anthropic-version` — the Anthropic convention.
    Anthropic,
    /// `x-goog-api-key: <secret>` — Google's native header.
    Google,
}

impl AuthStyle {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bearer => "bearer",
            Self::Anthropic => "anthropic",
            Self::Google => "google",
        }
    }

    /// Parses a stored value, falling back to the protocol default.
    pub fn parse(value: &str, protocol: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "anthropic" | "x-api-key" => Self::Anthropic,
            "google" | "x-goog-api-key" => Self::Google,
            "bearer" | "authorization" => Self::Bearer,
            _ if protocol == "anthropic" => Self::Anthropic,
            _ => Self::Bearer,
        }
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    /// Aggregators and multi-vendor relays.
    Relay,
    /// First-party vendor APIs.
    Vendor,
    /// Everything that runs on the operator's own machine.
    Local,
    /// The operator supplies every field.
    Custom,
}

#[derive(Serialize)]
pub struct Preset {
    pub id: &'static str,
    pub name: &'static str,
    pub vendor: &'static str,
    pub category: Category,
    /// Base URL without the protocol suffix, or `None` when the operator must supply it.
    pub endpoint: Option<&'static str>,
    pub auth_style: AuthStyle,
    /// Protocols the vendor is known to serve through this endpoint.
    pub protocols: &'static [&'static str],
    /// Seed model names, used to fill the model picker before a live probe runs.
    pub models: &'static [&'static str],
    /// One line shown under the preset in the picker.
    pub note: &'static str,
    pub docs: Option<&'static str>,
}

macro_rules! preset {
    ($id:literal, $name:literal, $vendor:literal, $cat:ident, $endpoint:expr, $auth:ident,
     [$($proto:literal),*], [$($model:literal),*], $note:literal, $docs:expr) => {
        Preset {
            id: $id,
            name: $name,
            vendor: $vendor,
            category: Category::$cat,
            endpoint: $endpoint,
            auth_style: AuthStyle::$auth,
            protocols: &[$($proto),*],
            models: &[$($model),*],
            note: $note,
            docs: $docs,
        }
    };
    ($id:literal, $name:literal, $vendor:literal, $cat:ident, $endpoint:expr, $auth:ident,
     [$($proto:literal),*], [$($model:literal),*], $note:literal) => {
        Preset {
            id: $id,
            name: $name,
            vendor: $vendor,
            category: Category::$cat,
            endpoint: $endpoint,
            auth_style: AuthStyle::$auth,
            protocols: &[$($proto),*],
            models: &[$($model),*],
            note: $note,
            docs: None,
        }
    };
}
pub fn all() -> &'static [Preset] {
    PRESETS
}

pub fn find(id: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|preset| preset.id == id)
}

static PRESETS: &[Preset] = &[
    // ---------------------------------------------------------------- relays
    preset!(
        "openrouter",
        "OpenRouter",
        "OpenRouter",
        Relay,
        Some("https://openrouter.ai/api"),
        Bearer,
        ["chat", "anthropic"],
        [
            "openai/gpt-5",
            "anthropic/claude-sonnet-4.5",
            "google/gemini-2.5-pro",
            "deepseek/deepseek-chat"
        ],
        "聚合上游，模型名带厂商前缀",
        Some("https://openrouter.ai/docs")
    ),
    preset!(
        "siliconflow",
        "硅基流动 SiliconFlow",
        "SiliconFlow",
        Relay,
        Some("https://api.siliconflow.cn"),
        Bearer,
        ["chat"],
        [
            "deepseek-ai/DeepSeek-V3",
            "Qwen/Qwen2.5-72B-Instruct",
            "moonshotai/Kimi-K2-Instruct"
        ],
        "国内聚合，OpenAI 兼容",
        Some("https://docs.siliconflow.cn")
    ),
    preset!(
        "together",
        "Together AI",
        "Together",
        Relay,
        Some("https://api.together.xyz"),
        Bearer,
        ["chat"],
        ["deepseek-ai/DeepSeek-V3", "meta-llama/Llama-3.3-70B-Instruct-Turbo"],
        "OpenAI 兼容，按 token 计费"
    ),
    preset!(
        "fireworks",
        "Fireworks AI",
        "Fireworks",
        Relay,
        Some("https://api.fireworks.ai/inference"),
        Bearer,
        ["chat"],
        ["accounts/fireworks/models/deepseek-v3"],
        "OpenAI 兼容"
    ),
    preset!(
        "modelscope",
        "魔搭 ModelScope",
        "ModelScope",
        Relay,
        Some("https://api-inference.modelscope.cn"),
        Bearer,
        ["chat"],
        ["Qwen/Qwen3-235B-A22B"],
        "阿里达摩院社区推理，有免费额度"
    ),
    preset!(
        "newapi",
        "New API / One API",
        "自建中转",
        Relay,
        None,
        Bearer,
        ["chat", "anthropic"],
        [],
        "自建中转站，填入你的站点地址"
    ),
    preset!(
        "sub2api",
        "Sub2API",
        "自建中转",
        Relay,
        None,
        Bearer,
        ["chat", "anthropic"],
        [],
        "自建订阅分发站，可配合「服务提供商」绑定密钥"
    ),

    // ---------------------------------------------------------------- vendors
    preset!(
        "openai",
        "OpenAI",
        "OpenAI",
        Vendor,
        Some("https://api.openai.com"),
        Bearer,
        ["responses", "compact", "chat", "anthropic"],
        ["gpt-5", "gpt-5-mini", "o4-mini"],
        "官方 Responses / Chat 双协议",
        Some("https://platform.openai.com/docs")
    ),
    preset!(
        "anthropic",
        "Anthropic Claude",
        "Anthropic",
        Vendor,
        Some("https://api.anthropic.com"),
        Anthropic,
        ["anthropic", "chat"],
        [
            "claude-opus-4-5",
            "claude-sonnet-4-5",
            "claude-haiku-4-5"
        ],
        "官方 Messages 协议，使用 x-api-key",
        Some("https://docs.anthropic.com")
    ),
    preset!(
        "gemini",
        "Google Gemini",
        "Google",
        Vendor,
        Some("https://generativelanguage.googleapis.com/v1beta/openai"),
        Bearer,
        ["chat"],
        ["gemini-2.5-pro", "gemini-2.5-flash"],
        "走 Gemini 的 OpenAI 兼容端点",
        Some("https://ai.google.dev/gemini-api/docs/openai")
    ),
    preset!(
        "deepseek",
        "DeepSeek",
        "DeepSeek",
        Vendor,
        Some("https://api.deepseek.com"),
        Bearer,
        ["chat"],
        ["deepseek-chat", "deepseek-reasoner"],
        "官方直连，价格低",
        Some("https://api-docs.deepseek.com")
    ),
    preset!(
        "moonshot",
        "Kimi / Moonshot",
        "Moonshot",
        Vendor,
        Some("https://api.moonshot.cn"),
        Bearer,
        ["chat"],
        ["kimi-k2-0905-preview", "moonshot-v1-128k"],
        "月之暗面官方",
        Some("https://platform.moonshot.cn/docs")
    ),
    preset!(
        "zhipu",
        "智谱 GLM",
        "Zhipu AI",
        Vendor,
        Some("https://open.bigmodel.cn/api/paas/v4"),
        Bearer,
        ["chat"],
        ["glm-4.6", "glm-4.5-air"],
        "智谱开放平台（v4 路径）",
        Some("https://open.bigmodel.cn/dev/api")
    ),
    preset!(
        "dashscope",
        "通义千问 Qwen",
        "Alibaba",
        Vendor,
        Some("https://dashscope.aliyuncs.com/compatible-mode/v1"),
        Bearer,
        ["chat"],
        ["qwen-max", "qwen-plus", "qwen-turbo"],
        "阿里云百炼 OpenAI 兼容模式",
        Some("https://help.aliyun.com/zh/model-studio/")
    ),
    preset!(
        "minimax",
        "MiniMax",
        "MiniMax",
        Vendor,
        Some("https://api.minimax.chat"),
        Bearer,
        ["chat"],
        ["MiniMax-M2", "abab6.5s-chat"],
        "官方 OpenAI 兼容"
    ),
    preset!(
        "stepfun",
        "阶跃星辰 StepFun",
        "StepFun",
        Vendor,
        Some("https://api.stepfun.com"),
        Bearer,
        ["chat"],
        ["step-2-16k", "step-1v-8k"],
        "官方 OpenAI 兼容"
    ),
    preset!(
        "volcengine",
        "火山方舟 Volcengine",
        "ByteDance",
        Vendor,
        Some("https://ark.cn-beijing.volces.com/api/v3"),
        Bearer,
        ["chat"],
        ["doubao-pro-32k"],
        "豆包模型，需先在方舟创建推理接入点",
        Some("https://www.volcengine.com/docs/82379")
    ),
    preset!(
        "xai",
        "xAI Grok",
        "xAI",
        Vendor,
        Some("https://api.x.ai"),
        Bearer,
        ["chat"],
        ["grok-4", "grok-3-mini"],
        "官方 OpenAI 兼容"
    ),
    preset!(
        "mistral",
        "Mistral",
        "Mistral AI",
        Vendor,
        Some("https://api.mistral.ai"),
        Bearer,
        ["chat"],
        ["mistral-large-latest", "mistral-small-latest"],
        "官方 OpenAI 兼容"
    ),
    preset!(
        "groq",
        "Groq",
        "Groq",
        Vendor,
        Some("https://api.groq.com/openai/v1"),
        Bearer,
        ["chat"],
        ["llama-3.3-70b-versatile"],
        "超低延迟推理"
    ),

    // ---------------------------------------------------------------- local
    preset!(
        "ollama",
        "Ollama",
        "本地",
        Local,
        Some("http://127.0.0.1:11434"),
        Bearer,
        ["chat"],
        ["qwen3:8b", "llama3.2:3b"],
        "本机 Ollama，密钥随意填",
        Some("https://github.com/ollama/ollama/blob/main/docs/openai.md")
    ),
    preset!(
        "lmstudio",
        "LM Studio",
        "本地",
        Local,
        Some("http://127.0.0.1:1234"),
        Bearer,
        ["chat"],
        [],
        "LM Studio 本地服务"
    ),
    preset!(
        "vllm",
        "vLLM / SGLang",
        "自建",
        Local,
        None,
        Bearer,
        ["chat"],
        [],
        "自建推理服务，填入地址与端口"
    ),

    // ---------------------------------------------------------------- custom
    preset!(
        "custom-openai",
        "自定义 OpenAI 兼容",
        "自定义",
        Custom,
        None,
        Bearer,
        ["chat", "responses"],
        [],
        "任何 OpenAI 兼容端点"
    ),
    preset!(
        "custom-anthropic",
        "自定义 Anthropic 兼容",
        "自定义",
        Custom,
        None,
        Anthropic,
        ["anthropic"],
        [],
        "任何 Anthropic Messages 兼容端点"
    ),
    preset!(
        "client-login",
        "客户端登录态导入",
        "自定义",
        Custom,
        None,
        Bearer,
        ["chat"],
        [],
        "把桌面客户端里已登录的凭证粘贴进来，地址按该客户端填写"
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_endpoints_are_absolute() {
        let mut seen = std::collections::HashSet::new();
        for preset in all() {
            assert!(seen.insert(preset.id), "duplicate preset id {}", preset.id);
            if let Some(endpoint) = preset.endpoint {
                let url = reqwest::Url::parse(endpoint).expect(preset.id);
                assert!(url.host_str().is_some(), "{} has no host", preset.id);
                assert!(
                    url.scheme() == "http" || url.scheme() == "https",
                    "{} is not http(s)",
                    preset.id
                );
                assert!(
                    url.query().is_none() && url.fragment().is_none(),
                    "{} carries query/fragment",
                    preset.id
                );
            }
        }
    }

    #[test]
    fn every_endpoint_maps_to_a_serving_root() {
        for preset in all() {
            if let Some(endpoint) = preset.endpoint {
                let url = crate::proxy::upstream_url(endpoint, "chat/completions")
                    .unwrap_or_else(|error| panic!("{}: {error}", preset.id));
                let path = url.path();
                assert!(
                    path.ends_with("/chat/completions") || path.ends_with("/messages"),
                    "{} resolves to {path}",
                    preset.id
                );
                // A duplicated protocol segment would 404 at the vendor.
                assert!(
                    !path.contains("/v1/v1") && !path.contains("/v1beta/openai/v1"),
                    "{} resolves to {path}",
                    preset.id
                );
            }
        }
    }

    #[test]
    fn auth_style_falls_back_to_protocol() {
        assert!(matches!(AuthStyle::parse("", "anthropic"), AuthStyle::Anthropic));
        assert!(matches!(AuthStyle::parse("", "chat"), AuthStyle::Bearer));
        assert!(matches!(AuthStyle::parse("google", "chat"), AuthStyle::Google));
    }
}
