<script>
  import { onMount } from 'svelte';
  import { notes, selectedNote, searchResults } from './notesStore';
  import { invoke } from '@tauri-apps/api/core';
  import { get } from 'svelte/store';

  let localContent = '';
  let localTitle = '';

  $: note = $selectedNote;
  $: localContent = note ? note.content : '';
  $: localTitle = note ? note.title : '';

  // Auto-save with debounce
  /** @type {ReturnType<typeof setTimeout>|undefined} */
  let saveTimeout;
  
  /**
   * Auto-saves the note after a short delay
   */
  function autoSave() {
    if (!note) return;
    
    // Immediately update the UI for responsiveness
    const updatedNote = { ...note, title: localTitle, content: localContent };
    
    // Update the main notes store
    notes.update(ns => ns.map(n => n.id === note.id ? updatedNote : n));
    
    // Update the search results store too
    searchResults.update(sr => sr.map(n => n.id === note.id ? updatedNote : n));
    
    // Update the selected note
    selectedNote.set(updatedNote);
    
    // Clear any existing timeout
    clearTimeout(saveTimeout);
    
    // Set a new timeout to save after 300ms of inactivity (reduced from 500ms)
    saveTimeout = setTimeout(async () => {
      await invoke('save_note', { id: note.id, title: localTitle, content: localContent });
    }, 300);
  }
  
  // Delete note function (kept for potential future use)
  async function deleteNote() {
    if (!note) return;
    await invoke('delete_note', { id: note.id });
    
    // Update both stores
    notes.update(ns => ns.filter(n => n.id !== note.id));
    searchResults.update(sr => sr.filter(n => n.id !== note.id));
    
    selectedNote.set(null);
  }
</script>

{#if note}
  <div class="note-editor">
    <input
      class="note-title"
      type="text"
      bind:value={localTitle}
      placeholder="Title"
      on:input={autoSave}
    />
    <textarea
      class="note-content"
      bind:value={localContent}
      placeholder="Write your note here..."
      on:input={autoSave}
    ></textarea>
  </div>
{:else}
  <p>Select a note or create a new one.</p>
{/if}

<style>
.note-editor {
  display: flex;
  flex-direction: column;
  gap: 0.6em;
  height: 100%;
}

.note-title {
  font-size: 1.2em;
  padding: 0.7em;
  border: 1px solid #ddd;
  border-radius: 4px;
  width: 100%;
  box-sizing: border-box;
  outline: none;
}

.note-title:focus {
  border-color: #c0d0e8;
  box-shadow: 0 0 0 1px #f0f4f8;
}

.note-content {
  font-family: inherit;
  font-size: 1em;
  padding: 0.7em;
  resize: none;
  flex: 1;
  border: 1px solid #ddd;
  border-radius: 4px;
  width: 100%;
  box-sizing: border-box;
  outline: none;
  overflow-y: auto;
  min-height: calc(100vh - 160px);
}

.note-content:focus {
  border-color: #c0d0e8;
  box-shadow: 0 0 0 1px #f0f4f8;
}

p {
  color: #666;
  font-size: 1.1em;
  text-align: center;
  margin-top: 2em;
}
</style>
