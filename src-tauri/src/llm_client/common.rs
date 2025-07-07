use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: Function,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub text_response: Option<String>,
    pub tool_calls: Vec<ToolCall>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: ToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

// Role mapping trait for different LLM providers
pub trait RoleMapper {
    fn map_role(&self, role: &str) -> &'static str;
    
    fn map_messages<F, T>(&self, messages: &[RequestMessage], mapper: F) -> Vec<T>
    where
        F: Fn(&RequestMessage, &'static str) -> T,
    {
        messages
            .iter()
            .map(|msg| {
                let provider_role = self.map_role(&msg.role);
                mapper(msg, provider_role)
            })
            .collect()
    }
}

// Gemini-specific role mapper
pub struct GeminiRoleMapper;

impl GeminiRoleMapper {
    pub fn new() -> Self {
        Self {}
    }
}

impl RoleMapper for GeminiRoleMapper {
    fn map_role(&self, role: &str) -> &'static str {
        match role {
            "user" => "user",
            "assistant" => "model",
            "system" => "user", // Gemini handles system messages differently
            _ => "user", // Default to user for unknown roles
        }
    }
}
