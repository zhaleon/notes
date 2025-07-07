pub mod common;

use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json;

// Valid roles for Gemini API
const ROLE_USER: &str = "user";
const ROLE_ASSISTANT: &str = "model";

// Define the Gemini API request structure
#[derive(Serialize, Debug)]
struct GeminiRequest {
    contents: Vec<Content>,
    generation_config: Option<GenerationConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Content {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parts: Option<Vec<Part>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Part {
    text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ThinkingConfig {
    #[serde(rename = "thinkingBudget")]
    thinking_budget: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct GenerationConfig {
    max_output_tokens: Option<i32>,
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "thinkingConfig")]
    thinking_config: Option<ThinkingConfig>,
}

// Define the Gemini API response structure
#[derive(Serialize, Deserialize, Debug)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Candidate {
    content: Content,
    #[serde(rename = "finishReason")]
    #[allow(dead_code)]
    finish_reason: String,
    #[allow(dead_code)]
    index: u32,
}

pub struct GeminiClient {
    api_key: String,
    http: reqwest::blocking::Client,
}

impl GeminiClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        // Create a client with custom timeouts to avoid hanging
        let client = reqwest::blocking::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(10)) // 10 second timeout
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
        // Print directly to stdout for debugging
        println!("[GEMINI_DEBUG] Starting get_completion with prompt: '{}'", prompt);
        info!("Getting completion for prompt: '{}'", prompt);
        
        // Create the request contents (single user prompt, no role field)
        let contents = vec![
            // System prompt to instruct Gemini to generate only 2-5 words for autocomplete
            Content {
                role: Some("user".to_string()), // Gemini does not support "system", so use "user"
                parts: Some(vec![Part { text: Some("You are an autocomplete assistant. Only return 2-5 words to continue the user's sentence. If the user's sentence does not end with a space or punctuation, start your completion with a space to ensure proper word separation.".to_string()) }]),
            },
            Content {
                role: Some("user".to_string()),
                parts: Some(vec![Part { text: Some(prompt.clone()) }]),
            }
        ];
        
        // Create generation config
        let generation_config = GenerationConfig {
            max_output_tokens: Some(max_tokens),
            temperature: Some(temperature),
            thinking_config: Some(ThinkingConfig { thinking_budget: 0 }),
        };
        
        // Create the request body
        let body = GeminiRequest {
            contents,
            generation_config: Some(generation_config),
        };
        
        // Make the API call
        // let url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent";
        let url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-lite-preview-06-17:generateContent";
        
        println!("[GEMINI_DEBUG] Sending request to Gemini API at {}", url);
        info!("Sending request to Gemini API at {}", url);
        
        // Log API key presence (not the actual key)
        println!("[GEMINI_DEBUG] API key is {}", if self.api_key.is_empty() { "EMPTY" } else { "present and has length" });
        if !self.api_key.is_empty() {
            println!("[GEMINI_DEBUG] API key length: {}", self.api_key.len());
        }
        
        let response_result = self.http
            .post(url)
            .header("Content-Type", "application/json")
            .header("x-goog-api-key", &self.api_key)
            .json(&body)
            .send();
            
        // Check for request sending errors
        if let Err(ref e) = response_result {
            println!("[GEMINI_DEBUG] Failed to send request: {}", e);
            println!("[GEMINI_DEBUG] Error kind: {:?}", e.to_string());
            return Err(anyhow!("Failed to send request to Gemini API: {}", e));
        }
        
        let response = response_result.unwrap();
        
        println!("[GEMINI_DEBUG] Response status: {}", response.status());
        info!("Response status: {}", response.status());
        
        // Check if the request was successful
        if !response.status().is_success() {
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            println!("[GEMINI_DEBUG] API error: {}", error_text);
            return Err(anyhow!("Gemini API returned error: {}", error_text));
        }
        
        // Get the response text
        let response_body = match response.text() {
            Ok(text) => text,
            Err(e) => {
                println!("[GEMINI_DEBUG] Failed to get response text: {}", e);
                return Err(anyhow!("Failed to get response text: {}", e));
            }
        };
        
        println!("[GEMINI_DEBUG] Raw response: {}", response_body);
        
        // Parse the response as a GeminiResponse
        let gemini_response: GeminiResponse = match serde_json::from_str(&response_body) {
            Ok(response) => response,
            Err(e) => {
                println!("[GEMINI_DEBUG] Failed to parse response: {}", e);
                println!("[GEMINI_DEBUG] Raw response for debugging: {}", response_body);
                return Err(anyhow!("Failed to parse Gemini API response: {}", e));
            }
        };
        
        println!("[GEMINI_DEBUG] Parsed response with {} candidates", gemini_response.candidates.len());
        println!("[GEMINI_DEBUG] Gemini response: {:#?}", gemini_response);
        
        // Extract the text from the response
        if let Some(candidate) = gemini_response.candidates.first() {
            // Check if parts field exists
            if let Some(parts) = &candidate.content.parts {
                println!("[GEMINI_DEBUG] First candidate has {} parts", parts.len());
                
                if let Some(part) = parts.first() {
                    if let Some(text) = &part.text {
                        println!("[GEMINI_DEBUG] Got completion text: '{}'", text);
                        info!("Got completion: '{}'", text);
                        return Ok(text.clone());
                    } else {
                        println!("[GEMINI_DEBUG] Part has no text field");
                    }
                } else {
                    println!("[GEMINI_DEBUG] Parts array is empty");
                }
            } else {
                // Handle case where parts is None
                println!("[GEMINI_DEBUG] Candidate has no parts field");
                // Return a default message since the API didn't provide any text
                return Ok("...".to_string());
            }
        } else {
            println!("[GEMINI_DEBUG] No candidates in response");
        }
        
        println!("[GEMINI_DEBUG] No text found in Gemini API response");
        Err(anyhow!("No text found in Gemini API response"))
    }

    pub fn check_server_status(&self) -> Result<bool> {
        // Simple check to see if the API key is set and we can make a basic request
        if self.api_key.is_empty() {
            warn!("API key is empty, server status check will fail");
            return Ok(false);
        }
        
        // Make a minimal request to check if the API is accessible
        let url = "https://generativelanguage.googleapis.com/v1beta/models";
        
        info!("Checking Gemini API status at {}", url);
        match self.http
            .get(url)
            .header("x-goog-api-key", &self.api_key)
            .send() {
                Ok(response) => {
                    let status = response.status().is_success();
                    info!("Gemini API status check: {}", if status { "OK" } else { "Failed" });
                    Ok(status)
                },
                Err(e) => {
                    warn!("Failed to connect to Gemini API: {}", e);
                    Ok(false)
                },
            }
    }
}

// End of GeminiClient implementation
