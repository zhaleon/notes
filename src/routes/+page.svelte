<script>
  import { onMount, onDestroy } from 'svelte';
  import { notes, selectedNote, searchResults, searchQuery, useSemanticSearch, similarityThreshold } from './notesStore';
  import NoteEditor from './NoteEditor.svelte';
  import CommandPalette from './CommandPalette.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { get } from 'svelte/store';

  let paletteOpen = false;
  function openPalette(e) {
    // ctrl+k or cmd+k
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
      e.preventDefault();
      paletteOpen = true;
    }
  }
  function closePalette() {
    paletteOpen = false;
  }
  onMount(() => {
    window.addEventListener('keydown', openPalette);
  });
  onDestroy(() => {
    window.removeEventListener('keydown', openPalette);
  });

  // Load notes from backend on mount
  onMount(async () => {
    const loaded = await invoke('list_notes');
    notes.set(loaded);
    searchResults.set(loaded);
  });
  
  // Search notes
  /**
   * @param {string} query - The search query
   * @returns {Promise<void>}
   */
  async function searchNotes(query) {
    searchQuery.set(query);
    
    if (query.trim() === '') {
      // If query is empty, show all notes
      searchResults.set(get(notes));
      return;
    }
    
    try {
      let results;
      
      // Use semantic search or basic search based on the toggle
      if (get(useSemanticSearch)) {
        // Pass the similarity threshold to the semantic search
        const threshold = get(similarityThreshold);
        results = await invoke('semantic_search', { query, distanceCutoff: threshold });
      } else {
        results = await invoke('search_notes', { query });
      }
      
      searchResults.set(results);
    } catch (error) {
      console.error('Search error:', error);
      // Fall back to basic search if semantic search fails
      if (get(useSemanticSearch)) {
        const results = await invoke('search_notes', { query });
        searchResults.set(results);
      }
    }
  }
  
  /**
   * Highlights the matched text in a string
   * @param {string} text - The text to highlight matches in
   * @param {string} query - The search query
   * @returns {string} HTML string with highlighted matches
   */
  function highlightMatches(text, query) {
    if (!query || query.trim() === '') return text;
    
    // Escape special regex characters to prevent errors
    const escapedQuery = query.trim().replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const regex = new RegExp(`(${escapedQuery})`, 'gi');
    return text.replace(regex, '<span class="highlight">$1</span>');
  }

  // Select a note
  /**
   * @param {{id: string, title: string, content: string}} note - The note to select
   */
  function selectNote(note) {
    selectedNote.set(note);
  }

  // Create a new note
  /**
   * @returns {Promise<void>}
   */
  async function createNote() {
    /** @type {{id: string, title: string, content: string}} */
    const newNote = await invoke('create_note');
    
    // Update both stores
    notes.update(ns => [newNote, ...ns]);
    searchResults.update(sr => [newNote, ...sr]);
    
    selectedNote.set(newNote);
  }
  

</script>

<main class="container notes-app">
  <CommandPalette open={paletteOpen} on:close={closePalette} />
  <!-- This span is used to make Svelte recognize the highlight class -->
  <span class="highlight" style="display: none;"></span>
  
  <div class="notes-layout">
    <aside class="notes-list">
      <h1>Minimal Notes</h1>
      <div class="search-container">
        <input 
          type="text" 
          placeholder="Search notes..." 
          class="search-input"
          on:input={(e) => searchNotes(e.currentTarget.value)}
          bind:value={$searchQuery}
        />
        <!-- Semantic Search toggle and threshold slider removed -->
      </div>
      <button on:click={createNote} class="create-btn">+ New Note</button>
      
      <ul>
        {#each $searchResults as note (note.id)}
          <li>
            <button 
              class="note-button" 
              class:selected={$selectedNote && $selectedNote.id === note.id} 
              on:click={() => selectNote(note)}
              on:keydown={(e) => e.key === 'Enter' && selectNote(note)}
              aria-label="Select note: {note.title || 'Untitled'}"
            >
              <h3>{@html highlightMatches(note.title, $searchQuery)}</h3>
              <p class="note-preview">{@html highlightMatches(note.content.substring(0, 60) + (note.content.length > 60 ? '...' : ''), $searchQuery)}</p>
            </button>
          </li>
        {/each}
      </ul>
    </aside>
    <section class="editor-section">
      <NoteEditor />
    </section>
  </div>
</main>

<style>
:global(html), :global(body) {
  margin: 0;
  padding: 0;
  overflow: hidden;
  height: 100%;
  width: 100%;
}

:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  background: #2f2f2f;
}

.container.notes-app {
  padding: 0;
  margin: 0;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  max-height: 100vh;
}

.notes-layout {
  display: flex;
  flex: 1;
  background: #fff;
  box-shadow: 0 2px 8px rgba(0,0,0,0.06);
  overflow: hidden;
}

.notes-list {
  width: 200px; /* Reduced from 240px */
  border-right: 1px solid #ddd;
  padding: 1.2em 0.8em;
  background: #f5f8fa;
  height: 100%;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  box-shadow: inset -5px 0 15px -5px rgba(0,0,0,0.05);
}

.notes-list h1 {
  font-size: 1.6rem;
  margin-top: 0;
  margin-bottom: 0.8em;
  padding-left: 0.3rem;
  color: #333;
  font-weight: 600;
  border-bottom: 2px solid #e0e4e8;
  padding-bottom: 0.5em;
}

.notes-list ul {
  list-style: none;
  padding: 0;
  margin-top: 0.5em;
}
.notes-list ul {
  list-style: none;
  padding: 0;
}
.notes-list li {
  margin-bottom: 0.3em;
}
.note-button {
  width: 100%;
  text-align: left;
  padding: 0.6em;
  background: white;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  margin-bottom: 0.6em;
  cursor: pointer;
  transition: all 0.2s ease;
  color: #333; /* Explicitly set text color */
}

.note-button h3 {
  margin: 0 0 0.3em 0;
  font-size: 0.9em;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.note-preview {
  margin: 0;
  font-size: 0.75em;
  color: #666;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.4;
}

.note-button:hover {
  background: #e8f0fe;
  border-color: #c0d0e8;
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(0,0,0,0.08);
}

.note-button.selected {
  background: #d8e6fd;
  border-color: #a8c0e0;
  color: #1a73e8;
  font-weight: 600;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.search-container {
  margin-bottom: 1em;
  width: 100%;
}

.search-input {
  width: 100%;
  padding: 0.5em;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 0.85em;
  margin-bottom: 0.5em;
  box-sizing: border-box;
  outline: none;
  transition: all 0.2s ease;
  background-color: #f8f9fa;
}

.search-input:focus {
  border-color: #4a7dfc;
  box-shadow: 0 0 0 2px rgba(74, 125, 252, 0.1);
  background-color: #fff;
}


.highlight {
  background-color: rgba(255, 230, 0, 0.4);
  padding: 0 2px;
  border-radius: 2px;
  font-weight: 500;
}

.create-btn {
  width: 100%;
  background: #4a7dfc;
  color: white;
  border: none;
  padding: 0.8em;
  border-radius: 6px;
  font-size: 0.9em;
  cursor: pointer;
  margin-bottom: 1em;
  transition: background 0.2s ease;
}

.create-btn:hover {
  background: #3a6ae8;
  transform: translateY(-1px);
  box-shadow: 0 3px 6px rgba(0,0,0,0.15);
}

.create-btn:active {
  background: #2a5ad8;
  transform: translateY(1px);
  box-shadow: 0 1px 2px rgba(0,0,0,0.1);
}


.editor-section {
  flex: 1;
  padding: 1.2em 1.5em;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
@media (max-width: 700px) {
  .notes-layout {
    flex-direction: column;
  }
  .notes-list {
    border-right: none;
    border-bottom: 1px solid #eee;
    min-width: unset;
  }
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}
</style>
