use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Debug)]
pub struct GeminiRequest {
    pub contents: Vec<Content>,
    pub generation_config: Option<GenerationConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parts: Option<Vec<Part>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Part {
    pub text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ThinkingConfig {
    #[serde(rename = "thinkingBudget")]
    pub thinking_budget: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenerationConfig {
    pub max_output_tokens: Option<i32>,
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "thinkingConfig")]
    pub thinking_config: Option<ThinkingConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeminiResponse {
    pub candidates: Vec<Candidate>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Candidate {
    pub content: Content,
    #[serde(rename = "finishReason")]
    #[allow(dead_code)]
    pub finish_reason: String,
    #[allow(dead_code)]
    pub index: u32,
}

pub struct GeminiClient {
    pub api_key: String,
    pub http: reqwest::blocking::Client,
}

impl GeminiClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        let client = reqwest::blocking::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| {
                error!("Failed to build HTTP client with custom timeout, using default");
                reqwest::blocking::Client::new()
            });
        Self {
            api_key: api_key.into(),
            http: client,
        }
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn get_completion(&self, prompt: String, max_tokens: i32, temperature: f32) -> Result<String> {
        println!("[GEMINI_DEBUG] Starting get_completion with prompt: '{}'", prompt);
        info!("Getting completion for prompt: '{}'", prompt);

        let contents = vec![
            Content {
                role: Some("user".to_string()),
                parts: Some(vec![Part { text: Some("You are an autocomplete assistant. Only return 2-5 words to continue the user's sentence. If the user's sentence does not end with a space or punctuation, start your completion with a space to ensure proper word separation.".to_string()) }]),
            },
            Content {
                role: Some("user".to_string()),
                parts: Some(vec![Part { text: Some(prompt.clone()) }]),
            }
        ];

        let generation_config = GenerationConfig {
            max_output_tokens: Some(max_tokens),
            temperature: Some(temperature),
            thinking_config: Some(ThinkingConfig { thinking_budget: 0 }),
        };

        let body = GeminiRequest {
            contents,
            generation_config: Some(generation_config),
        };

        let url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-lite-preview-06-17:generateContent";

        println!("[GEMINI_DEBUG] Sending request to Gemini API at {}", url);
        info!("Sending request to Gemini API at {}", url);

        let response_result = self.http
            .post(url)
            .header("x-goog-api-key", &self.api_key)
            .json(&body)
            .send();

        if response_result.is_err() {
            let err = response_result.unwrap_err();
            println!("[GEMINI_DEBUG] API request failed: {}", err);
            return Err(anyhow!("Gemini API request failed: {}", err));
        }

        let response = response_result.unwrap();
        if !response.status().is_success() {
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            println!("[GEMINI_DEBUG] API error: {}", error_text);
            return Err(anyhow!("Gemini API error: {}", error_text));
        }

        let response_body = match response.text() {
            Ok(text) => text,
            Err(e) => {
                println!("[GEMINI_DEBUG] Failed to read response body: {}", e);
                return Err(anyhow!("Failed to read Gemini API response body: {}", e));
            }
        };

        println!("[GEMINI_DEBUG] Raw response: {}", response_body);

        let gemini_response: GeminiResponse = match serde_json::from_str(&response_body) {
            Ok(response) => response,
            Err(e) => {
                println!("[GEMINI_DEBUG] Failed to parse response JSON: {}", e);
                return Err(anyhow!("Failed to parse Gemini API response JSON: {}", e));
            }
        };

        println!("[GEMINI_DEBUG] Parsed response with {} candidates", gemini_response.candidates.len());
        println!("[GEMINI_DEBUG] Gemini response: {:#?}", gemini_response);

        if let Some(candidate) = gemini_response.candidates.first() {
            if let Some(parts) = &candidate.content.parts {
                println!("[GEMINI_DEBUG] First candidate has {} parts", parts.len());
                if let Some(part) = parts.first() {
                    if let Some(text) = &part.text {
                        return Ok(text.clone());
                    }
                } else {
                    println!("[GEMINI_DEBUG] Parts array is empty");
                }
            } else {
                return Ok("...".to_string());
            }
        }
        println!("[GEMINI_DEBUG] No text found in Gemini API response");
        Err(anyhow!("No text found in Gemini API response"))
    }
}
