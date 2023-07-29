<script lang="ts">
	import { createEventDispatcher } from "svelte";

	import FaTimes from 'svelte-icons/fa/FaTimes.svelte'

	const dispatch = createEventDispatcher();

	export let name: string;
	export let isActive: boolean;
	export let isDeletable: boolean;

	let deleteBtn: HTMLElement;
</script>

<li class="border-2 {isActive ? 'border-blue-500' : 'clickable'} flex" on:click={(event) => { 
	if (event.target === deleteBtn) return;
	dispatch('focus') 
}}>
	{#if isActive}
		<span class="px-2" contenteditable bind:innerText={name}></span>
	{:else}
		<span class="px-2">{name}</span>
	{/if}
	{#if isDeletable}
		<span bind:this={deleteBtn} class="flex content-center items-center ml-2 hover:bg-slate-200 cursor-pointer w-6 h-6 p-[6px]" on:click={(_) => dispatch('delete')}>
			<FaTimes />
		</span>
	{/if}
</li>
