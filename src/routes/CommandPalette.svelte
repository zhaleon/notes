<script lang="ts">
  import { createEventDispatcher, onMount, afterUpdate } from 'svelte';
  import { notes, selectedNote } from './notesStore';
  import { get } from 'svelte/store';

  export let open = false;
  const dispatch = createEventDispatcher();
  let inputEl: HTMLInputElement | null = null;
  let localQuery = '';
  let filteredNotes: {id: string, title: string, content: string}[] = [];
  let activeIndex = 0;
  let resultsEl: HTMLUListElement | null = null;

  $: filteredNotes = localQuery.trim() === ''
    ? get(notes)
    : get(notes).filter(n =>
        n.title.toLowerCase().includes(localQuery.toLowerCase()) ||
        n.content.toLowerCase().includes(localQuery.toLowerCase())
      );
  $: if (activeIndex >= filteredNotes.length) activeIndex = 0;

  function onInput(e: Event) {
    localQuery = (e.target as HTMLInputElement).value;
    activeIndex = 0;
  }

  function onSelect(note: {id: string, title: string, content: string}) {
    selectedNote.set(note);
    dispatch('close');
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      dispatch('close');
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      activeIndex = (activeIndex + 1) % filteredNotes.length;
      scrollToActive();
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      activeIndex = (activeIndex - 1 + filteredNotes.length) % filteredNotes.length;
      scrollToActive();
    } else if (e.key === 'Enter' && filteredNotes[activeIndex]) {
      onSelect(filteredNotes[activeIndex]);
    }
  }
  function onResultKeyDown(e: KeyboardEvent, idx: number) {
    if (e.key === 'Enter') {
      onSelect(filteredNotes[idx]);
    }
  }
  function scrollToActive() {
    if (!resultsEl) return;
    const active = resultsEl.querySelector('.active') as HTMLElement;
    if (active) active.scrollIntoView({ block: 'nearest' });
  }
  onMount(() => {
    if (open && inputEl) {
      inputEl.focus();
    }
  });
  afterUpdate(() => {
    if (open && inputEl) {
      inputEl.focus();
    }
  });
</script>

{#if open}
  <button class="palette-backdrop" aria-label="Close command palette" on:click={() => dispatch('close')} tabindex="0"></button>
  <div class="palette-modal" role="dialog" aria-modal="true" aria-label="Command palette">
    <div class="palette-search-wrapper">
      <span class="search-icon">🔍</span>
      <input
        class="palette-search"
        bind:this={inputEl}
        type="text"
        placeholder="Search notes..."
        bind:value={localQuery}
        on:input={onInput}
        on:keydown={onKeyDown}
        aria-label="Search notes"
      />
    </div>
    <ul class="palette-results" bind:this={resultsEl} role="listbox" aria-label="Notes search results">
      {#each filteredNotes as note, idx (note.id)}
        <li>
          <button
            type="button"
            class:active={idx === activeIndex}
            role="option"
            aria-selected={idx === activeIndex}
            tabindex="0"
            on:click={() => onSelect(note)}
            on:keydown={(e) => onResultKeyDown(e as KeyboardEvent, idx)}
          >{note.title || 'Untitled'}</button>
        </li>
      {/each}
      {#if filteredNotes.length === 0}
        <li class="no-results">No notes found.</li>
      {/if}
    </ul>
  </div>
{/if}

<style>
.palette-backdrop {
  position: fixed;
  top: 0; left: 0; right: 0; bottom: 0;
  background: rgba(0,0,0,0.32);
  z-index: 1000;
}
.palette-modal {
  position: fixed;
  top: 10vh;
  left: 50%;
  transform: translateX(-50%);
  background: #222;
  color: #f6f6f6;
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0,0,0,0.25);
  min-width: 420px;
  max-width: 90vw;
  padding: 2rem 2rem 1rem 2rem;
  z-index: 1001;
  display: flex;
  flex-direction: column;
  align-items: stretch;
}
.palette-search-wrapper {
  display: flex;
  align-items: center;
  background: #181818;
  border-radius: 8px;
  padding: 0.5em 1em;
  border: 2px solid #4a7dfc;
  margin-bottom: 1.5em;
}
.palette-search {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: #f6f6f6;
  font-size: 1.2em;
  padding: 0.4em 0.6em;
}
.search-icon {
  margin-right: 0.6em;
  font-size: 1.1em;
  color: #aaa;
}
.palette-results {
  list-style: none;
  padding: 0;
  margin: 0;
  max-height: 260px;
  overflow-y: auto;
  background: #181818;
  border-radius: 10px;
  box-shadow: 0 2px 16px rgba(0,0,0,0.10);
}
.palette-results li {
  margin: 0;
  padding: 0;
}
.palette-results button {
  width: 100%;
  display: flex;
  align-items: center;
  background: transparent;
  border: none;
  outline: none;
  color: #f6f6f6;
  font-size: 1.06em;
  padding: 0.85em 1.1em;
  border-radius: 8px;
  cursor: pointer;
  text-align: left;
  transition: background 0.13s, color 0.13s;
  position: relative;
}
.palette-results button.active,
.palette-results button:focus,
.palette-results button:hover {
  background: linear-gradient(90deg, #304ffe 0%, #1976d2 100%);
  color: #fff;
  box-shadow: 0 2px 8px rgba(48,79,254,0.08);
  z-index: 1;
}
.palette-results button:not(:last-child)::after {
  content: '';
  display: block;
  position: absolute;
  left: 16px;
  right: 16px;
  bottom: 0;
  height: 1px;
  background: rgba(255,255,255,0.06);
}
.no-results {
  color: #888;
  text-align: center;
  padding: 1em 0;
  font-size: 1em;
}
</style>
