<script lang="ts">
	import CellInput from "./CellInput.svelte";
	import CellOutput from "./CellOutput.svelte";
	import { createEventDispatcher, onMount, SvelteComponent } from "svelte";
	import type { Cell } from "$lib/notedag";

	import FaPlay from 'svelte-icons/fa/FaPlay.svelte'
	import FaTrash from 'svelte-icons/fa/FaTrash.svelte'

	const dispatch = createEventDispatcher();

	export let cell: Cell;
	export let isFocused: boolean;
	export let isDeletable: boolean;

	let el: HTMLElement;
	let inputElement: SvelteComponent;

	onMount(() => {});

	export function edit() {
		inputElement.edit();
	}
</script>

<li
	bind:this={el}
	class="-m-[2px] flex py-2 border-2 {isFocused ? 'border-blue-500' : 'border-transparent hover:border-blue-200'}"
	on:click={(_event) => { dispatch('focus') }}
	tabindex="-1"
>
	<div class="flex flex-col w-14 items-end">
		<pre class="mx-2">[{cell.output.executionCount || cell.output.status}]</pre>
	</div>
	<div class="flex-1 flex flex-col">
		<!--<pre class="p-2 bg-slate-100" contenteditable bind:innerText={cell.code.value}></pre>-->
		<CellInput
			bind:this={inputElement}
			bind:state={cell.code} 
			on:run={(_event) => { dispatch('run') }}
			on:escape={(_event) => { el.focus() }}
			on:mode={(event) => { dispatch('mode', event.detail) }}
		/>
		<CellOutput bind:state={cell.output} />
	</div>
	<ul class="flex flex-col px-2">
		<a class="clickable w-6 h-6 p-1 mb-1 text-blue-500" on:click={(_event) => dispatch('run')}><FaPlay /></a>
		{#if isDeletable}
			<a class="clickable w-6 h-6 p-1 mb-1 text-blue-500" on:click={(_event) => dispatch('delete')}><FaTrash /></a>
		{/if}
	</ul>
</li>
