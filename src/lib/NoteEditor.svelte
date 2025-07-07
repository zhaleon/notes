<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { debounce } from '$lib/utils';
  
  export let content = '';
  let completion = '';
  let isLoading = false;
  
  // Debounce the completion request to avoid spamming the server
  const getCompletion = debounce(async (text: string) => {
    if (!text.trim()) {
      completion = '';
      return;
    }
    
    try {
      isLoading = true;
      const response = await fetch('http://localhost:3000/completion', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ prompt: text })
      });
      
      if (response.ok) {
        const data = await response.json();
        completion = data.completion || '';
      }
    } catch (error) {
      console.error('Error getting completion:', error);
    } finally {
      isLoading = false;
    }
  }, 300);
  
  function handleInput(e: Event) {
    const target = e.target as HTMLTextAreaElement;
    content = target.value;
    getCompletion(content);
  }
  
  // Clean up any pending debounced calls
  onDestroy(() => {
    getCompletion.cancel?.();
  });
</script>

<div class="editor-container">
  <textarea
    bind:value={content}
    on:input={handleInput}
    class="note-editor"
    placeholder="Start typing..."
    rows={10}
  ></textarea>
  
  {#if completion}
    <div class="completion" class:loading={isLoading}>
      {completion}
    </div>
  {/if}
</div>

<style>
  .editor-container {
    position: relative;
    width: 100%;
  }
  
  .note-editor {
    width: 100%;
    min-height: 200px;
    padding: 1rem;
    border: 1px solid #ccc;
    border-radius: 4px;
    font-family: inherit;
    font-size: 1rem;
    line-height: 1.5;
  }
  
  .completion {
    color: #666;
    margin-top: 0.5rem;
    padding: 0.5rem;
    border-left: 3px solid #4CAF50;
    background-color: #f9f9f9;
    opacity: 1;
    transition: opacity 0.3s ease;
  }
  
  .completion.loading {
    opacity: 0.7;
  }
</style>
