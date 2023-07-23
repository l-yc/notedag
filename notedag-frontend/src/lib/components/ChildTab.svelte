<script lang="ts">
	import CellInput from "./CellInput.svelte";
	import { createEventDispatcher } from "svelte";
	import type { Cell } from "$lib/notedag";

	const dispatch = createEventDispatcher();

	export let name: string;
	export let isActive: boolean;
	export let isDeletable: boolean;
</script>

<li class="border-2 {isActive ? 'border-blue-500' : 'clickable'} flex" on:click={(_event) => { dispatch('focus') }}>
	{#if isActive}
		<span class="px-2" contenteditable bind:innerText={name}></span>
	{:else}
		<span class="px-2">{name}</span>
	{/if}
	{#if isDeletable}
		<span class="flex content-center items-center ml-2 px-1 hover:bg-slate-200 cursor-pointer" on:click={(_) => dispatch('delete')}>
			<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
				<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
			</svg>
		</span>
	{/if}
</li>
