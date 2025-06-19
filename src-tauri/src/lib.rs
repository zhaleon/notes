use serde::{Deserialize, Serialize};
use std::fs::{File, create_dir_all, read_dir, remove_file};
use std::io::{Read, Write};
use std::path::PathBuf;
use uuid::Uuid;

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
        let note = Note {
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
