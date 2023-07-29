<script lang="ts">
    /** @type {import('./$types').PageData} */
	export let data : PageData;
	import { EditorMode } from "$lib";
	import type { Keybind } from "$lib/keybindings";
	import { registerDocumentKeybindings } from "$lib/keybindings";
	import Cell from "$lib/components/Cell.svelte";
	import Header from "$lib/components/Header.svelte";
	import type { UUID } from "$lib/notedag";
	import { NoteDAGState } from "$lib/notedag";
	//import { KernelManager } from "$lib/kernel";
	import { KernelManager } from "./kernel";

	import { onMount, SvelteComponent } from 'svelte';
	import ChildTab from '$lib/components/ChildTab.svelte';
	import FaPlus from 'svelte-icons/fa/FaPlus.svelte'

	let notedag: NoteDAGState = NoteDAGState.load(data.contents, () => { notedag = notedag; });
	let kernel: KernelManager = new KernelManager(() => { kernel = kernel; });

	/// handlers
	async function save(filePath: string) {
		const response = await fetch("/api/write", {
			method: "POST",
			body: JSON.stringify({ filePath, contents: JSON.stringify(notedag) }),
			headers: {
				"Content-Type": "application/json",
			},
		});
		const json = await response.json();
		alert(JSON.stringify(json));
	}

	/// lifecycle
	onMount(() => {
		kernel.connect();
		const kb: Keybind[] = [
		  {
			keys: ["a"],
			description: "Add cell above",
			run: () => notedag.addNewCellBefore(),
		  },
		  {
			keys: ["b"],
			description: "Add cell below",
			run: () => notedag.addNewCellAfter(),
		  },
		  {
			keys: ["x"],
			description: "Delete focused cell",
			run: () => notedag.deleteCell(notedag.focusedCell, notedag.focusedGroup),
		  },
		  {
			keys: ["k"],
			description: "Focus cell above",
			run: () => notedag.focusCellBefore(),
		  },
		  {
			keys: ["j"],
			description: "Focus cell after",
			run: () => notedag.focusCellAfter(),
		  },
		  {
			keys: ["i"],
			description: "Enter edit mode for a cell",
			run: () => { cellElements[notedag.focusedCell].edit() },
		  },
		  {
			keys: ["Shift-Enter"],
			description: "Run cell",
			run: () => runCell(notedag.focusedCell),
		  },
		];

		registerDocumentKeybindings(kb);
	});

	let cellElements: Record<string, SvelteComponent> = {};

	async function runCell(cellId: UUID) {
		//await connect();
		let cell = notedag.cells[cellId];
		console.log('sending', cell.code.value);

		notedag.clearCell(cell.id);
		cell.output.executionCount = '.';
		await kernel.submit(cell, (updated) => { 
			notedag = notedag;
			console.log('updated', updated);
		});
	}

	async function runGroup(groupId: UUID) {
		const group = notedag.groups[groupId];
		for (const id of group.cells) {
			await runCell(id);
		}
	}

	async function runActiveGroupChain() {
		for (const group of notedag.activeGroupChain) {
			await runGroup(group.id);
		}
	}

	let editorMode = EditorMode.NORMAL;
	let documentLocation = '';
</script>

<svelte:document on:scroll={(_event) => 
	documentLocation = `${Math.round(window.scrollY / (document.body.scrollHeight - window.innerHeight) * 100)}%`
}/>

<div>
	<Header>
		<p slot="annotation" class="ml-4 text-xl">{data.filename}</p>
		<div slot="toolbar" class="flex constrained">
			<span class="px-3 py-1">
				Status: {kernel.connection.status}
			</span>
			<span class="flex-1"></span>
			<input type="button" class="px-3 py-1 clickable" value="Connect" on:click={(_event) => kernel.connect()}/>
			<input type="button" class="px-3 py-1 clickable" value="Save" on:click={(_event) => save(data.root)}/>
			<input type="button" class="px-3 py-1 clickable" value="Add Group" on:click={(_event) => notedag.addNewGroup()}/>
			<input type="button" class="px-3 py-1 clickable" value="Run All" on:click={(_event) => runActiveGroupChain()}/>
			<input type="button" class="px-3 py-1 clickable" value="Clear All" on:click={(_event) => notedag.clearOutput()}/>
		</div>
	</Header>
		
	<!--<p>{JSON.stringify(notedag.groups)}</p>-->
	<!--<p>{notedag.focusedGroup}</p>-->
	<!--<p>{notedag.focusedCell}</p>-->

	<div class="h-screen flex flex-col constrained">
		<ul class="flex flex-col space-y-2">
			{#each notedag.activeGroupChain as group, idx (group.id)}
				<li>
					<ul class="flex">
						{#if idx === 0}
							<ChildTab
								bind:name={group.name}
								isActive={true}
								isDeletable={false}
								on:focus={() => notedag.setNextGroup(notedag.activeGroupChain[idx-1].id, group.id)}
								on:delete={() => notedag.deleteGroup(group.id, notedag.activeGroupChain[idx-1].id)}
							/>
						{:else}
							{#each notedag.activeGroupChain[idx-1].children as childId (childId)}
								<ChildTab
									bind:name={notedag.groups[childId].name}
									isActive={childId === group.id}
									isDeletable={true}
									on:focus={() => notedag.setNextGroup(notedag.activeGroupChain[idx-1].id, childId)}
									on:delete={() => notedag.deleteGroup(childId, notedag.activeGroupChain[idx-1].id)}
								/>
							{/each}
							<a class="flex content-center items-center clickable w-7 h-7 p-2" on:click={(_event) => notedag.addNewGroup(notedag.activeGroupChain[idx-1]?.id)}>
								<FaPlus />
							</a>
						{/if}
						<span class="flex-1"></span>
						<span class="px-3 clickable" on:click={(_) => notedag.addNewCell(group.id)}>Add Cell</span>
						<span class="px-3 clickable" on:click={(_) => runGroup(group.id)}>Run Group</span>
						<span class="px-3 clickable" on:click={(_) => notedag.clearGroup(group.id)}>Clear Group</span>
					</ul>

					<ul class="-mt-[2px] flex flex-col border-2" on:click={(_event) => notedag.focusGroup(group.id)}>
						<!--<div>{JSON.stringify(group)}</div>-->
						{#each group.cells.map(id => notedag.cells[id]) as cell (cell.id)}
							<!--<pre>{cell.id}</pre>-->
							<Cell 
								bind:this={cellElements[cell.id]}
								bind:cell
								isFocused={notedag.focusedCell === cell.id}
								isDeletable={group.cells.length > 1}
								on:focus={() => notedag.focusCell(group.id, cell.id)}
								on:delete={() => notedag.deleteCell(cell.id, group.id)}
								on:run={() => runCell(cell.id)}
								on:mode={(event) => { editorMode = event.detail }}
							/>
						{/each}
					</ul>
				</li>
			{/each}
		</ul>
	</div>

	<!-- modal editor status bar -->
	<div class="w-full fixed bottom-0 bg-white border-t-2 border-slate-500 mt-2 z-10">
		<div class="flex items-end constrained">
			<span class="px-2">-- {editorMode} --</span>
			<span class="flex-1"></span>
			<span class="px-2">{documentLocation}</span>
		</div>
	</div>
</div>
