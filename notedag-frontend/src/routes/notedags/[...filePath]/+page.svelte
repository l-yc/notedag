<script lang="ts">
    /** @type {import('./$types').PageData} */
	export let data : PageData;
	import { kernel } from '$lib';
	import type { Keybind } from "$lib/keybindings";
	import { registerDocumentKeybindings } from "$lib/keybindings";
	import Convert from 'ansi-to-html';
	import Cell from "$lib/components/Cell.svelte";
	import Header from "$lib/components/Header.svelte";
	import type { UUID } from "$lib/notedag";
	import { NoteDAG, Group, CellOutput } from "$lib/notedag";

	import { onMount } from 'svelte';
	import ChildTab from '$lib/components/ChildTab.svelte';

	let notedag: NoteDAG = NoteDAG.from_file_data(data.contents);

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
			keys: ["Shift-Enter"],
			description: "Add cell after",
			run: () => runCell(notedag.focusedCell),
		  },
		];

		registerDocumentKeybindings(kb);
	});

	/// kernel
	let connection = {
		ws: null as WebSocket | null,
		status: 'disconnected',
	};

	async function connect() {
		await new Promise<void>((resolve, reject) => {
			if (connection.ws !== null) {
				resolve();
				return;
			} 

			const uri = kernel.uri;
			let ws = new WebSocket(uri);
			connection = {
				ws,
				status: 'connecting',
			}

			ws.onopen = function() {
				connection.status = 'connected';
				console.log('connected');
				resolve();
			};

			ws.onmessage = function(msg) {
				console.log('received', msg.data);

				try {
					const { id, name, value, status } = JSON.parse(msg.data);

					const contentTypeHandler: Record<string, ((s: string) => string)> = {
						'text/plain': (s: string) => {
							let pre = document.createElement('pre');
							pre.innerText = s;
							return pre.outerHTML;
						},
						'text/html': (s: string) => {
							let div = document.createElement('div');
							div.innerHTML = s;
							return div.outerHTML;
						},
						'image/png': (s: string) => {
							let img = document.createElement('img');
							img.src = 'data:image/png;base64,' + s;
							return img.outerHTML;
						}
					};

					if (id !== undefined) {
						let cell = notedag.cells[id];
						switch (name) {
							case 'output':
							case 'error':
								console.log(value);
								let escaped = new Option(value).innerHTML;
								let convert = new Convert();
								let html = convert.toHtml(escaped);
								if (name == 'output') cell.output.value = html;
								else cell.output.error = html;
								break;
							case 'result':
								{
									let json: Record<string, string> = JSON.parse(value);
									cell.output.result = '';
									for (const k of ['text/html', 'text/plain']) {
										if (k in json) {
											const v = json[k];	
											cell.output.result += contentTypeHandler[k](v);
											break;
										}
									}
								}
								break;
							case 'data':
								{
									let json: Record<string, string> = JSON.parse(value);
									for (let [k, v] of Object.entries(json)) {
										cell.output.result += contentTypeHandler[k](v);
									}
								}
								break;
							case 'queued':
							case 'running':
							case 'complete':
								cell.output.executionCount = value;
								break;
						}
						notedag = notedag;
					} else {
						connection.status = status;
					}
				} catch (e) {
					console.error('failed to parse ws message');
				}
			};

			ws.onclose = function() {
				connection = {
					ws: null,
					status: 'disconnected'
				};
				console.log('disconnected');
			};
		});
	}

	async function runCell(cellId: UUID) {
		await connect();

		let cell = notedag.cells[cellId];
		console.log('sending', cell.code.value);
		notedag = notedag;

		if (connection.ws !== null) {
			cell.output = CellOutput.default();
			cell.output.executionCount = '.';

			const ws = connection.ws;
			ws.send(JSON.stringify({
				id: cell.id,
				value: cell.code.value,
			}));
		}
	}

	async function runGroup(groupId: UUID) {
		let group = notedag.groups[groupId];
		for (let id of group.cells) {
			await runCell(id);
		}
	}

	async function runActiveGroupChain() {
		for (let group of notedag.activeGroupChain) {
			await runGroup(group.id);
		}
	}

</script>

<div>
	<Header>
		<p slot="annotation" class="ml-4 text-xl">{data.filename}</p>
		<div slot="toolbar" class="flex constrained">
			<span class="px-3 py-1">
				Status: {connection.status}
			</span>
			<span class="flex-1"></span>
			<input type="button" class="px-3 py-1 clickable" value="Connect" on:click={(_event) => connect()}/>
			<input type="button" class="px-3 py-1 clickable" value="Save" on:click={(_event) => save(data.root)}/>
			<input type="button" class="px-3 py-1 clickable" value="Add Group" on:click={(_event) => notedag.addNewGroup()}/>
			<input type="button" class="px-3 py-1 clickable" value="Run All" on:click={(_event) => runActiveGroupChain()}/>
			<input type="button" class="px-3 py-1 clickable" value="Clear All" on:click={(_event) => notedag.clearOutput()}/>
		</div>
	</Header>
		
	<!--<p>{JSON.stringify(notedag)}</p>-->
	<p>{notedag.focusedGroup}</p>
	<p>{notedag.focusedCell}</p>
	<!--<p>{...notedag}</p>-->
	<!--<p>{JSON.stringify(activeGroupChain)}</p>-->

	<div class="flex flex-col constrained">
		<ul class="flex flex-col space-y-2">
			{#each notedag.activeGroupChain as group, idx (group.id)}
				<!--<li on:click={(_event) => notedag.focusGroup(group.id)}>-->
				<li on:click={(_event) => notedag.focusGroup(group.id)}>
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
							{#each notedag.activeGroupChain[idx-1].children as childId}
								<ChildTab
									bind:name={notedag.groups[childId].name}
									isActive={childId === group.id}
									isDeletable={true}
									on:focus={() => notedag.setNextGroup(notedag.activeGroupChain[idx-1].id, childId)}
									on:delete={() => notedag.deleteGroup(childId, notedag.activeGroupChain[idx-1].id)}
								/>
							{/each}
							<a class="flex content-center items-center px-1 clickable" on:click={(_event) => notedag.addNewGroup(notedag.activeGroupChain[idx-1]?.id)}>
								<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
									<path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
								</svg>
							</a>
						{/if}
						<span class="flex-1"></span>
						<span class="px-3 clickable" on:click={(_) => notedag.addNewCell(group.id)}>Add Cell</span>
						<span class="px-3 clickable" on:click={(_) => runGroup(group.id)}>Run Group</span>
						<span class="px-3 clickable" on:click={(_) => notedag.clearGroup(group.id)}>Clear Group</span>
					</ul>

					<ul class="flex flex-col border-2">
						<pre>{group.id}</pre>
						{#each group.cells.map(id => notedag.cells[id]) as cell}
							<!--{cell.id}-->
							<Cell 
								bind:cell
								isFocused={notedag.focusedCell === cell.id}
								isDeletable={group.cells.length > 1}
								on:focus={() => notedag.focusCell(group.id, cell.id)}
								on:delete={() => notedag.deleteCell(cell.id, group.id)}
								on:run={() => runCell(cell.id)}
							/>
						{/each}
					</ul>
				</li>
			{/each}
		</ul>
	</div>
</div>
