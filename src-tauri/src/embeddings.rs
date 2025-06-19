use crate::Note;
use hnsw_rs::hnsw::Hnsw;
use hnsw_rs::dist::DistCosine;
// use anndists::dist::DistCosine; // Comment out if this exists
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;

// Define error type for embedding operations
#[derive(Debug)]
pub enum EmbeddingError {
    NotFound,
}

impl std::fmt::Display for EmbeddingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EmbeddingError::NotFound => write!(f, "Item not found"),
        }
    }
}

impl std::error::Error for EmbeddingError {}

// Initialize a global embedding manager
lazy_static! {
    pub static ref EMBEDDING_MANAGER: Arc<Mutex<EmbeddingManager>> = {
        Arc::new(Mutex::new(EmbeddingManager::new()))
    };
}

// Get the global embedding manager
#[allow(dead_code)]
pub fn get_embedding_manager() -> Arc<Mutex<EmbeddingManager>> {
    EMBEDDING_MANAGER.clone()
}

// Function moved to be a method of EmbeddingManager

// EmbeddingManager struct to manage HNSW index and note mappings
pub struct EmbeddingManager {
    index: Option<Hnsw<f32, DistCosine>>,
    note_to_id: HashMap<String, usize>,
    id_to_note: HashMap<usize, String>,
    next_id: usize,
}

impl EmbeddingManager {
    pub fn generate_simple_embedding(text: &str) -> Vec<f32> {
        // TODO use an embedding model here
        let mut embedding = vec![0.0; 128];
        for (i, _c) in text.chars().enumerate() {
            let idx = (_c as usize) % 128;
            embedding[idx] += 1.0 / (i as f32 + 1.0);
        }

        let magnitude: f32 = embedding.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            embedding.iter_mut().for_each(|val| *val /= magnitude);
        }
        embedding
    }
    
    pub fn new() -> Self {
        EmbeddingManager {
            index: None,
            note_to_id: HashMap::new(),
            id_to_note: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn initialize(&mut self) -> Result<(), EmbeddingError> {
        let max_elements = 10000; // Maximum number of elements in the index
        let max_nb_connection = 16; // Maximum number of connections per element
        let ef_construction = 200; // Size of the dynamic candidate list for construction
        let nb_layer = 16; // Number of layers in the graph
        
        self.index = Some(Hnsw::new(
            max_nb_connection,
            max_elements,
            nb_layer,
            ef_construction,
            DistCosine {}
        ));
        
        Ok(())
    }

    pub fn add_note(&mut self, note: &Note) -> Result<(), EmbeddingError> {
        if self.index.is_none() {
            self.initialize()?;
        }
        let index = self.index.as_mut().unwrap();
        let text = format!("{} {}", note.title, note.content);
        let embedding = Self::generate_simple_embedding(&text);
        let id = self.next_id;
        
        // Create a tuple with the embedding slice and the ID
        let data_point = (&embedding, id);
        index.insert(data_point);
        
        self.note_to_id.insert(note.id.clone(), id);
        self.id_to_note.insert(id, note.id.clone());
        self.next_id += 1;
        Ok(())
    }

    pub fn update_note(&mut self, note: &Note) -> Result<(), EmbeddingError> {
        // Remove the old note if it exists
        if self.note_to_id.contains_key(&note.id) {
            self.remove_note(note)?;
        }
        // Add the updated note
        self.add_note(note)
    }

    pub fn remove_note(&mut self, note: &Note) -> Result<(), EmbeddingError> {
        if let Some(id) = self.note_to_id.remove(&note.id) {
            self.id_to_note.remove(&id);
            // Note: HNSW doesn't support removal, so we just remove from our mappings
            // The actual vector in the index will remain but won't be accessible
            Ok(())
        } else {
            Err(EmbeddingError::NotFound)
        }
    }

    pub fn search(&mut self, query: &str, k: usize, distance_cutoff: Option<f32>) -> Result<Vec<String>, EmbeddingError> {
        if self.index.is_none() {
            return Ok(vec![]);
        }
        
        let index = self.index.as_ref().unwrap();
        let embedding = Self::generate_simple_embedding(query);
        let ef_search = 50; // Size of the dynamic candidate list for searching
        
        let neighbors = index.search(&embedding, k, ef_search);
        
        let mut result = Vec::new();
        for neighbor in neighbors {
            // Apply distance cutoff if specified
            if let Some(cutoff) = distance_cutoff {
                if neighbor.distance > cutoff {
                    continue; // Skip this result if it's beyond the cutoff
                }
            }
            
            if let Some(note_id) = self.id_to_note.get(&neighbor.d_id) {
                result.push(note_id.clone());
            }
        }
        
        Ok(result)
    }
    
    pub fn rebuild_index(&mut self, notes: &[Note]) -> Result<(), EmbeddingError> {
        // Clear existing data
        self.index = None;
        self.note_to_id.clear();
        self.id_to_note.clear();
        self.next_id = 0;
        
        // Initialize a new index
        self.initialize()?;
        
        // Add all notes to the index
        for note in notes {
            self.add_note(note)?;
        }
        
        Ok(())
    }
}


