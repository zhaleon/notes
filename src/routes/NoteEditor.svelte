<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { notes, selectedNote, searchResults } from './notesStore';
  import { invoke } from '@tauri-apps/api/core';
  import { get } from 'svelte/store';
  import { debounce } from '$lib/utils';

  let localContent = '';
  let localTitle = '';
  let completion = '';
  let isLoading = false;

  $: note = $selectedNote;
  $: localContent = note ? note.content : '';
  $: localTitle = note ? note.title : '';

  // Debounced function to get completions using Tauri command
  const getCompletion = debounce(async (text: string) => {
    if (!text.trim()) {
      completion = '';
      return;
    }
    
    try {
      console.log(`[Frontend] Requesting completion for: '${text}'`);
      const startTime = performance.now();
      isLoading = true;
      
      // Use Tauri command instead of HTTP request
      completion = await invoke('autocomplete', { prompt: text });
      
      const elapsed = performance.now() - startTime;
      console.log(`[Frontend] Received completion in ${elapsed.toFixed(2)}ms: '${completion}'`);
    } catch (error) {
      console.error('[Frontend] Error getting completion:', error);
      completion = '';
    } finally {
      isLoading = false;
    }
  }, 150); // Reduced debounce time for more responsive feel

  // Auto-save with debounce
  let saveTimeout: ReturnType<typeof setTimeout> | null = null;
  
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
    if (saveTimeout !== null) clearTimeout(saveTimeout);
    
    // Set a new timeout to save after 300ms of inactivity (reduced from 500ms)
    saveTimeout = setTimeout(async () => {
      await invoke('save_note', { id: note.id, title: localTitle, content: localContent });
    }, 300);
  }
  
  function handleContentInput() {
    autoSave();
    // Get completion for the current line
    const lines = localContent.split('\n');
    const currentLine = lines[lines.length - 1];
    console.log(`[Frontend] Current line: '${currentLine}'`);
    getCompletion(currentLine);
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

  onDestroy(() => {
    if (saveTimeout) clearTimeout(saveTimeout);
    getCompletion.cancel?.();
  });
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
    <div class="editor-container">
      <textarea
        bind:value={localContent}
        on:input={handleContentInput}
        on:keydown={e => {
          // Handle tab key
          if (e.key === 'Tab') {
            e.preventDefault(); // Always prevent default tab behavior
            
            if (completion) {
              // If there's a completion, accept it
              localContent += completion;
              completion = '';
            } else {
              // Otherwise insert a tab character
              const selStart = e.target.selectionStart;
              const selEnd = e.target.selectionEnd;
              
              // Insert tab at cursor position
              localContent = localContent.substring(0, selStart) + '\t' + 
                            localContent.substring(selEnd);
              
              // Set cursor position after the tab
              setTimeout(() => {
                e.target.selectionStart = e.target.selectionEnd = selStart + 1;
              }, 0);
            }
            
            // Trigger auto-save after tab insertion
            setTimeout(autoSave, 0);
          }
        }}
        class="note-content"
        placeholder="Start writing your note here..."
        rows={15}
      ></textarea>
      
      {#if completion}
        <div class="completion" class:loading={isLoading}>
          {completion}
        </div>
      {/if}
    </div>
  </div>
{:else}
  <p>Select a note or create a new one.</p>
{/if}

<style>
  .editor-container {
    position: relative;
    width: 100%;
  }

  .note-editor {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    height: 100%;
  }
  
  .completion {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 100%;
    color: #666;
    padding: 0.5rem;
    margin-bottom: 0.5rem;
    border: 1px solid #e0e0e0;
    border-radius: 4px;
    background-color: #f9f9f9;
    opacity: 1;
    transition: opacity 0.3s ease;
    pointer-events: none;
  }
  
  .completion.loading {
    opacity: 0.7;
  }
  
  .completion::before {
    content: 'Suggestion: ';
    font-weight: bold;
    color: #4CAF50;
  }
  
  .note-content {
    width: 100%;
    box-sizing: border-box;
    outline: none;
    resize: none;
    min-height: 200px;
    font-family: inherit;
    font-size: 1em;
    line-height: 1.5;
  }

  .note-title:focus {
    border-color: #c0d0e8;
  }
  
  .note-content:focus {
    border-color: #c0d0e8;
    box-shadow: 0 0 0 1px #f0f4f8;
    outline: none;
    resize: none;
    flex: 1;
    border: 1px solid #ddd;
    border-radius: 4px;
    padding: 0.7em;
    line-height: 1.5;
    font-size: 1em;
    font-family: inherit;
    color: #333;
    background: #fff;
    transition: border-color 0.2s, box-shadow 0.2s;
  }

  .note-title:focus {
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
