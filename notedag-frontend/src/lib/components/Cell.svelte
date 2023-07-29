<script lang="ts">
	import CellInput from "./CellInput.svelte";
	import CellOutput from "./CellOutput.svelte";
	import { createEventDispatcher, onMount } from "svelte";
	import type { Cell } from "$lib/notedag";

	const dispatch = createEventDispatcher();

	export let cell: Cell;
	export let isFocused: boolean;
	export let isDeletable: boolean;

	let el: HTMLElement;

	onMount(() => {});
</script>

<li bind:this={el} class="flex py-2 border-2 {isFocused ? 'border-blue-500' : 'border-white hover:border-slate-200'}" on:click={(_event) => { dispatch('focus') }} tabindex="-1">
	<div class="flex flex-col w-14 items-end">
		<pre class="mx-2">[{cell.output.executionCount}]</pre>
	</div>
	<div class="flex-1 flex flex-col">
		<!--<pre class="p-2 bg-slate-100" contenteditable bind:innerText={cell.code.value}></pre>-->
		<CellInput
			bind:state={cell.code} 
			on:run={(_event) => { dispatch('run') }}
			on:escape={(_event) => { el.focus() }}
		/>
		<CellOutput bind:state={cell.output} />
	</div>
	<ul class="flex flex-col px-2">
		<a class="clickable" on:click={(_event) => dispatch('run')}>
			<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
				<path stroke-linecap="round" stroke-linejoin="round" d="M5.25 5.653c0-.856.917-1.398 1.667-.986l11.54 6.348a1.125 1.125 0 010 1.971l-11.54 6.347a1.125 1.125 0 01-1.667-.985V5.653z" />
			</svg>
		</a>
		{#if isDeletable}
			<a class="clickable mb-1" on:click={(_event) => dispatch('delete')}>
				<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
					<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
				</svg>
			</a>
		{/if}
	</ul>
</li>
