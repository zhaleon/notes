<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher, tick } from 'svelte';
  import { notes, selectedNote, searchResults } from './notesStore';
  import { invoke } from "@tauri-apps/api/core";
  import { get } from 'svelte/store';
  import { debounce } from '$lib/utils';

  const dispatch = createEventDispatcher();
  
  // Reactive variables for the currently edited note
  let localContent = '';
  let localTitle = '';
  // Current inline completion suggestion
  let suggestion = '';
  let textareaEl: HTMLTextAreaElement | null = null;
  let overlayEl: HTMLDivElement | null = null;

  const requestCompletion = debounce(async () => {
    if (!localContent) { suggestion = ''; return; }
    const words = localContent.split(/\s+/);
    const lastWords = words.slice(Math.max(0, words.length - 10)).join(' ');
    if (!lastWords) { suggestion = ''; return; }
    try {
      const comp = await invoke('get_completion', {
        prompt: lastWords,
        maxTokens: 10,
        temperature: 0.7
      }) as string;
      suggestion = comp.trim();
    } catch (e) {
      console.error('completion error', e);
    }
  }, 400);

  function syncOverlayScroll() {
    if (textareaEl && overlayEl) {
      overlayEl.scrollTop = textareaEl.scrollTop;
      overlayEl.scrollLeft = textareaEl.scrollLeft;
    }
  }
  
  // Update local state when selectedNote changes
  $: {
    if ($selectedNote) {
      localContent = $selectedNote.content;
      localTitle = $selectedNote.title;
    } else {
      localContent = '';
      localTitle = '';
    }
  }

  let saveTimeout: ReturnType<typeof setTimeout> | null = null;
  function autoSave() {
    if (!$selectedNote) return;
    
    const updatedNote = { 
      ...$selectedNote, 
      title: localTitle, 
      content: localContent 
    };
    
    notes.update(ns => ns.map(n => n.id === $selectedNote.id ? updatedNote : n));
    searchResults.update(sr => sr.map(n => n.id === $selectedNote.id ? updatedNote : n));
    selectedNote.set(updatedNote);
    
    if (saveTimeout !== null) clearTimeout(saveTimeout);
    
    saveTimeout = setTimeout(async () => {
      try {
        await invoke('save_note', { 
          id: $selectedNote.id, 
          title: localTitle, 
          content: localContent 
        });
      } catch (error) {
        console.error('Error saving note:', error);
      }
    }, 300);
  }
  
  let prevContent = '';
  let prevSuggestion = '';

function handleContentInput(e: Event) {
  const oldContent = prevContent;
  const newContent = localContent;
  // Only handle single character additions for now
  if (
    suggestion &&
    newContent.length === oldContent.length + 1 &&
    newContent.endsWith(suggestion[0])
  ) {
    // User typed the next suggestion character
    suggestion = suggestion.slice(1);
    prevContent = newContent;
    prevSuggestion = suggestion;
    autoSave();
    return;
  }
  // Otherwise, hide suggestion immediately and request a new one
  suggestion = '';
  prevContent = newContent;
  prevSuggestion = '';
  autoSave();
  requestCompletion();
}

  // async function deleteNote() {
  //   if (!note) return;
  //   await invoke('delete_note', { id: note.id });
    
  //   // Update both stores
  //   notes.update(ns => ns.filter(n => n.id !== note.id));
  //   searchResults.update(sr => sr.filter(n => n.id !== note.id));
    
  //   selectedNote.set(null);
  // }

  // Auto-select a note when the component mounts
  onMount(async () => {
    // Wait a moment for the notes to load
    setTimeout(async () => {
      const allNotes = get(notes);
      if (allNotes.length > 0 && !get(selectedNote)) {
        // Select the first note
        selectedNote.set(allNotes[0]);
      } else if (allNotes.length === 0) {
        // If no notes exist, create a new one
        const newNote = await invoke('create_note', { title: 'New Note', content: '' });
        // Type assertion to ensure the note has the correct structure
        const typedNote = newNote as {id: string, title: string, content: string};
        notes.update(n => [...n, typedNote]);
        selectedNote.set(typedNote);
      }
    }, 100);
  });

  onDestroy(() => {
    if (saveTimeout) clearTimeout(saveTimeout);
  });

  // function handleSearch() {
  //   const query = prompt("Enter search term:");
  //   if (query) {
  //     const results = get(notes).filter(note => 
  //       note.title.toLowerCase().includes(query.toLowerCase()) || 
  //       note.content.toLowerCase().includes(query.toLowerCase())
  //     );
      
  //     if (results.length > 0) {
  //       const noteIds = results.map(note => note.id);
  //       dispatch("search-results", { noteIds });
  //     } else {
  //       window.alert("No results found");
  //     }
  //   }
  // }

  // async function handleSave() {
  //   if (note) {
  //     await invoke("save_note", { id: note.id, title: localTitle, content: localContent });
  //     window.alert("Note saved!");
  //   }
  // }

  // async function handleDelete() {
  //   if (note && confirm("Are you sure you want to delete this note?")) {
  //     await invoke("delete_note", { id: note.id });
  //     dispatch("note-deleted");
  //   }
  // }

  // async function handleSemanticSearch() {
  //   try {
  //     const query = prompt("Enter search query:");
  //     if (!query) return;

  //     const results = await invoke("semantic_search", { query });
  //     console.log("Semantic search results:", results);
      
  //     if (results && Array.isArray(results) && results.length > 0) {
  //       const noteIds = results.map((r: any) => r.id);
  //       dispatch("search-results", { noteIds });
  //     } else {
  //       window.alert("No results found");
  //     }
  //   } catch (error: any) {
  //     console.error("Error during semantic search:", error);
  //     window.alert(`Error: ${error.message || error}`);
  //   }
  // }

  // async function checkServerStatus() {
  //   try {
  //     const isAvailable = await invoke("check_server_status");
  //     if (isAvailable) {
  //       window.alert("gRPC server is available!");
  //     } else {
  //       window.alert("gRPC server is not available. Please start the server.");
  //     }
  //   } catch (error: any) {
  //     console.error("Error checking server status:", error);
  //     window.alert(`Error: ${error.message || error}`);
  //   }
  // }

  // Get completion from gRPC server
  // async function getCompletion() {
  //   try {
  //     if (!$selectedNote) return;
      
  //     // Get the current selection or use the last few words as context
  //     const selection = window.getSelection()?.toString() || "";
  //     let prompt = selection;
      
  //     if (!prompt || prompt.trim() === "") {
  //       // If no selection, use the last few words of the note content
  //       const words = localContent.split(/\s+/);
  //       const lastWords = words.slice(Math.max(0, words.length - 10)).join(" ");
  //       prompt = lastWords;
  //     }
      
  //     if (!prompt || prompt.trim() === "") {
  //       const userPrompt = window.prompt("Enter text for completion:");
  //       if (!userPrompt) return;
  //       prompt = userPrompt;
  //     }

  //     const completion = await invoke("get_completion", { 
  //       prompt, 
  //       maxTokens: 50, 
  //       temperature: 0.7 
  //     }) as string;
      
  //     // Insert the completion at the current cursor position
  //     const textarea = document.getElementById("note-content") as HTMLTextAreaElement;
  //     if (!textarea) return;
      
  //     const cursorPos = textarea.selectionEnd;
  //     const textBefore = localContent.substring(0, cursorPos);
  //     const textAfter = localContent.substring(cursorPos);
      
  //     // Update the local content with the completion
  //     localContent = textBefore + completion + textAfter;
      
  //     // Trigger auto-save
  //     autoSave();
      
  //     // Move cursor to end of inserted completion
  //     setTimeout(() => {
  //       if (textarea) {
  //         textarea.selectionStart = textarea.selectionEnd = cursorPos + completion.length;
  //         textarea.focus();
  //       }
  //     }, 0);
      
  //   } catch (error: any) {
  //     console.error("Error getting completion:", error);
  //     window.alert(`Error: ${error.message || error}`);
  //   }
  // }
</script>

{#if $selectedNote}
  <div class="note-editor">
    <input
      class="note-title"
      type="text"
      bind:value={localTitle}
      placeholder="Title"
      on:input={autoSave}
    />
    <div class="editor-container">
      {#if suggestion}
        <div bind:this={overlayEl} class="autocomplete-overlay"><span class="typed">{localContent}</span><span class="suggestion">{suggestion}</span></div>
      {/if}
      <textarea
        bind:this={textareaEl}
        id="note-content" class="note-content completable-textarea"
        bind:value={localContent}
        on:input={e => { handleContentInput(e); setTimeout(syncOverlayScroll,0); }}
        on:scroll={syncOverlayScroll}
        on:keydown={async e => {
           if (e.key === 'Tab') {
             e.preventDefault();
             const textarea = e.target as HTMLTextAreaElement;
             if (suggestion) {
               const selStart = textarea.selectionStart;
               const selEnd = textarea.selectionEnd;
               const before = localContent.slice(0, selStart);
               const after = localContent.slice(selEnd);
               localContent = before + suggestion + after;
               await tick(); // Wait for DOM update
               textarea.selectionStart = textarea.selectionEnd = selStart + suggestion.length;
               textarea.focus();
               suggestion = '';
               autoSave();
               requestCompletion(); // Generate a new suggestion after accepting
             } else {
               const selStart = textarea.selectionStart;
               const selEnd = textarea.selectionEnd;
               const before = localContent.slice(0, selStart);
               const after = localContent.slice(selEnd);
               localContent = before + '\t' + after;
               await tick();
               textarea.selectionStart = textarea.selectionEnd = selStart + 1;
               textarea.focus();
               autoSave();
             }
           }
         }}

        placeholder="Start writing your note here..."
        rows={15}
      ></textarea>

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
  
  .note-content {
    width: 100%;
    box-sizing: border-box;
    outline: none;
    resize: none;
    min-height: 200px;
    font-family: inherit;
    font-size: 1em;
    line-height: 1.5;
    border: 1px solid #c0d0e8;
    border-radius: 4px;
    padding: 12px;
    color: #333;
    background: #fff;
    box-shadow: 0 0 0 1px #f0f4f8;
    transition: border-color 0.2s, box-shadow 0.2s;
    vertical-align: top;
  }

  .note-title {
    border: 1px solid #c0d0e8;
    border-radius: 4px;
    padding: 0.5em;
    font-size: 1.2em;
    box-shadow: 0 0 0 1px #f0f4f8;
    outline: none;
  }

  p {
    color: #666;
    font-size: 1.1em;
    text-align: center;
    margin-top: 2em;
  }
.autocomplete-overlay {
  border: 1px solid transparent;
  border-radius: 4px; /* match .note-content */

  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  min-height: 200px;
  font-family: inherit;
  font-size: 1em;
  line-height: 1.5;
  box-sizing: border-box;
  padding: 12px;
  white-space: pre-wrap;
  pointer-events: none;
  overflow: hidden;
  vertical-align: top;
}
.autocomplete-overlay { pointer-events:none; overflow:hidden; }
.autocomplete-overlay .typed {
  color: transparent;
  /* color: #be2626; */
}
.autocomplete-overlay .suggestion {
  color: #aaa;
  font-family: inherit;
  font-size: inherit;
  font-weight: inherit;
  font-style: inherit;
  line-height: inherit;
  letter-spacing: inherit;
  vertical-align: inherit;
}
.completable-textarea {
  background: transparent;
  position: relative;
  z-index: 2;
}
.editor-container {
  position: relative;
}
</style>
