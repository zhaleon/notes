use serde::{Deserialize, Serialize};
use std::fs::{File, create_dir_all, read_dir, remove_file};
use std::io::{Read, Write};
use std::path::PathBuf;
use uuid::Uuid;

// LLM client module for local completions
mod llm_client;

// Embeddings module removed

// Define our Note structure
#[derive(Serialize, Deserialize, Clone)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
}

// Helper function to get the notes directory
fn notes_dir() -> PathBuf {
    let dir = dirs::home_dir().unwrap().join(".minimal-notes").join("notes");
    create_dir_all(&dir).ok();
    dir
}

// Define a module for our commands
pub mod commands {
    use super::*;
    
    
    // Basic text search for notes
    #[tauri::command]
    pub fn search_notes(query: String) -> Vec<Note> {
        if query.is_empty() {
            return list_notes();
        }
        
        // Perform basic text search
        let query = query.to_lowercase();
        let all_notes = list_notes();
        all_notes
            .into_iter()
            .filter(|note| {
                note.title.to_lowercase().contains(&query) || 
                note.content.to_lowercase().contains(&query)
            })
            .collect()
    }
    
    // Semantic search (simplified version - falls back to text search for now)
    #[tauri::command]
    pub fn semantic_search(query: String, _distance_cutoff: Option<f32>) -> Vec<Note> {
        // For now, just use the basic text search
        // In the future, this could be enhanced with embeddings or other semantic techniques
        search_notes(query)
    }
    
    // List all notes
    #[tauri::command]
    pub fn list_notes() -> Vec<Note> {
        let dir = notes_dir();
        let mut notes = vec![];
        if let Ok(entries) = read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(mut file) = File::open(entry.path()) {
                    let mut contents = String::new();
                    if file.read_to_string(&mut contents).is_ok() {
                        if let Ok(note) = serde_json::from_str::<Note>(&contents) {
                            notes.push(note);
                        }
                    }
                }
            }
        }
        notes.sort_by(|a, b| b.id.cmp(&a.id)); // newest first
        notes
    }
    
    // Create a new note
    #[tauri::command]
    pub fn create_note() -> Note {
        let note = Note {
            id: Uuid::new_v4().to_string(),
            title: "New Note".to_string(),
            content: "".to_string(),
        };
        
        // Save the note to disk
        if let Err(e) = save_note_to_disk(&note) {
            eprintln!("Error saving note: {}", e);
        }
        
        // Vector indexing removed
        
        note
    }
    
    // Save a note
    #[tauri::command]
    pub fn save_note(id: String, title: String, content: String) -> Result<(), String> {
        let note = Note { id: id.clone(), title, content };
        
        // Save the note to disk
        let result = save_note_to_disk(&note);
        
        // Vector indexing removed
        
        result
    }
    
    // Helper function to save a note to disk
    fn save_note_to_disk(note: &Note) -> Result<(), String> {
        let dir = notes_dir();
        let mut path = dir;
        path.push(format!("{}.json", note.id));
        File::create(&path)
            .and_then(|mut f| f.write_all(serde_json::to_string(&note).unwrap().as_bytes()))
            .map_err(|e| e.to_string())
    }
    
    // Delete a note
    #[tauri::command]
    pub fn delete_note(id: String) -> Result<(), String> {
        // Create a temporary note object with the ID to remove from the vector index
        let _note = Note {
            id: id.clone(),
            title: String::new(),
            content: String::new(),
        };
        
        // Vector indexing removed
        
        // Delete the note file
        let dir = notes_dir();
        let mut path = dir;
        path.push(format!("{}.json", id));
        remove_file(&path).map_err(|e| e.to_string())
    }
}

// Create a new module for completion commands
mod completion {
    use crate::llm_client::GeminiClient;
    use crate::llm_client::common::RequestMessage;
    use std::sync::Mutex;
    use std::sync::Arc;
    use log::{info, error};
    use once_cell::sync::Lazy;

    // Define the environment variable name for the Gemini API key
    const GEMINI_API_KEY_ENV: &str = "GEMINI_API_KEY";

    // Create a global Gemini client with an API key
    static CLIENT: Lazy<Arc<Mutex<GeminiClient>>> = Lazy::new(|| {
        // Get API key from environment variable
        let api_key = std::env::var(GEMINI_API_KEY_ENV)
            .unwrap_or_else(|_| {
                // Fallback to empty string if not found, which will cause runtime errors
                // when trying to use the API, but will allow the app to start
                error!("GEMINI_API_KEY environment variable not set. API calls will fail.");
                String::new()
            });

        Arc::new(Mutex::new(GeminiClient::new(api_key)))
    });

    // Get a text completion
    #[tauri::command]
    pub fn get_completion(prompt: String, max_tokens: i32, temperature: f32) -> Result<String, String> {
        // Print directly to stdout for debugging
        println!("[FRONTEND_DEBUG] Tauri command: get_completion called with prompt: '{}'", prompt);
        println!("[FRONTEND_DEBUG] max_tokens: {}, temperature: {}", max_tokens, temperature);
        info!("Tauri command: get_completion called with prompt: '{}', max_tokens: {}, temperature: {}", prompt, max_tokens, temperature);
        
        // Get the client
        println!("[FRONTEND_DEBUG] Acquiring lock on GeminiClient");
        let client_result = CLIENT.lock();
        if let Err(e) = client_result {
            let error_msg = format!("Failed to acquire lock on GeminiClient: {}", e);
            println!("[FRONTEND_DEBUG] {}", error_msg);
            return Err(error_msg);
        }
        
        let client = client_result.unwrap();
        println!("[FRONTEND_DEBUG] Successfully acquired lock on GeminiClient");
        
        // Check if the API key is configured
        if client.api_key().is_empty() {
            let error_msg = "Gemini API key not configured. Set the GEMINI_API_KEY environment variable.";
            println!("[FRONTEND_DEBUG] {}", error_msg);
            error!("API key is empty! Please set the GEMINI_API_KEY environment variable.");
            return Err(error_msg.to_string());
        }
        
        println!("[FRONTEND_DEBUG] API key is present, calling get_completion");
        info!("API key is configured, making request to Gemini API");
        
        // Make the request and log the result
        println!("[FRONTEND_DEBUG] Calling client.get_completion");
        let result = client.get_completion(prompt, max_tokens, temperature);
        
        match &result {
            Ok(text) => {
                println!("[FRONTEND_DEBUG] Successfully got completion: '{}'", text);
                info!("Successfully got completion: '{}'", text);
                Ok(text.clone())
            },
            Err(e) => {
                println!("[FRONTEND_DEBUG] Error getting completion: {}", e);
                error!("Error getting completion: {}", e);
                Err(e.to_string())
            },
        }
    }
    
    // Get a chat completion (simplified to use get_completion)
    #[tauri::command]
    pub fn chat_completion(messages: Vec<RequestMessage>) -> Result<String, String> {
        println!("[FRONTEND_DEBUG] Tauri command: chat_completion called with {} messages", messages.len());
        info!("Tauri command: chat_completion called with {} messages", messages.len());
        
        // Log message contents for debugging
        for (i, msg) in messages.iter().enumerate() {
            println!("[FRONTEND_DEBUG] Message {}: role='{}', content='{}'", i, msg.role, msg.content);
        }
        
        // Get the client
        println!("[FRONTEND_DEBUG] Acquiring lock on GeminiClient for chat_completion");
        let client_result = CLIENT.lock();
        if let Err(e) = client_result {
            let error_msg = format!("Failed to acquire lock on GeminiClient: {}", e);
            println!("[FRONTEND_DEBUG] {}", error_msg);
            return Err(error_msg);
        }
        
        let client = client_result.unwrap();
        println!("[FRONTEND_DEBUG] Successfully acquired lock on GeminiClient");
        
        // Check if the API key is configured
        if client.api_key().is_empty() {
            let error_msg = "Gemini API key not configured. Set the GEMINI_API_KEY environment variable.";
            println!("[FRONTEND_DEBUG] {}", error_msg);
            return Err(error_msg.to_string());
        }
        
        // Extract the last user message to use as prompt
        println!("[FRONTEND_DEBUG] Extracting last user message as prompt");
        let prompt = messages.iter()
            .filter(|msg| msg.role == "user")
            .last()
            .map(|msg| msg.content.clone())
            .unwrap_or_else(|| String::new());
            
        if prompt.is_empty() {
            let error_msg = "No user message found in the conversation";
            println!("[FRONTEND_DEBUG] {}", error_msg);
            return Err(error_msg.to_string());
        }
        
        println!("[FRONTEND_DEBUG] Extracted prompt: '{}'", prompt);
        
        // Call the get_completion method instead
        println!("[FRONTEND_DEBUG] Calling get_completion with prompt");
        let result = client.get_completion(prompt, 30, 0.7);
        
        match &result {
            Ok(text) => {
                println!("[FRONTEND_DEBUG] chat_completion success: '{}'", text);
                Ok(text.clone())
            },
            Err(e) => {
                println!("[FRONTEND_DEBUG] chat_completion error: {}", e);
                Err(e.to_string())
            },
        }
    }

    // Check if Gemini API is configured and working
    #[tauri::command]
    pub fn check_server_status() -> Result<bool, String> {
        println!("[FRONTEND_DEBUG] Checking Gemini API status");
        info!("Checking Gemini API status");
        
        // Get the client
        println!("[FRONTEND_DEBUG] Acquiring lock on GeminiClient for status check");
        let client_result = CLIENT.lock();
        if let Err(e) = client_result {
            let error_msg = format!("Failed to acquire lock on GeminiClient: {}", e);
            println!("[FRONTEND_DEBUG] {}", error_msg);
            return Err(error_msg);
        }
        
        let client = client_result.unwrap();
        println!("[FRONTEND_DEBUG] Successfully acquired lock on GeminiClient");
        
        // Check if the API key is configured
        if client.api_key().is_empty() {
            println!("[FRONTEND_DEBUG] Gemini API key not configured");
            error!("Gemini API key not configured");
            return Ok(false);
        }
        
        println!("[FRONTEND_DEBUG] API key is present and configured");
        
        // Try a minimal API request to check if API is working
        println!("[FRONTEND_DEBUG] Sending test request to Gemini API");
        let result = client.get_completion("Hello".to_string(), 5, 0.7);
        
        match &result {
            Ok(text) => {
                println!("[FRONTEND_DEBUG] Gemini API is available, response: '{}'", text);
                info!("Gemini API is available");
                Ok(true)
            },
            Err(e) => {
                println!("[FRONTEND_DEBUG] Gemini API is not available: {}", e);
                error!("Gemini API is not available: {}", e);
                Ok(false)
            }
        }
    }
}

// Main run function
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Launch local LLM server in the background
    {

    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_notes,
            commands::create_note,
            commands::save_note,
            commands::delete_note,
            commands::search_notes,
            commands::semantic_search,
            completion::get_completion,
            completion::chat_completion,
            completion::check_server_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
