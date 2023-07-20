<script lang="ts">
    /** @type {import('./$types').PageData} */
	export let data : PageData;
	import { kernel } from '$lib';
	import { v4 as uuidv4 } from 'uuid';
	import Convert from 'ansi-to-html';

	type UUID = string;

	interface CellCode {
		value: string;
		syntax: string;
	}

	interface CellOutput {
		value: string;
		error: string;
		result: string;
		executionCount: string;
	}

	interface Cell {
		id: UUID;
		code: CellCode;
		meta: object;
		output: CellOutput;
	}

	interface Group {
		id: UUID;
		name: string;
		cells: UUID[];
		dependentGroups: UUID[];
		nextGroup: UUID | null;
	}

	interface NoteDAG {
		/// Smallest unit of code
		cells: Record<UUID, Cell>;

		/// Groups cells together with some metadata
		groups: Record<UUID, Group>;

		/// Entry point for execution
		root: UUID;
	}

	let notedag: NoteDAG = NoteDAGFromJSON(data.contents);
	let focusedGroup: UUID = notedag.root;
	let focusedCell: UUID | null = notedag.groups[notedag.root].cells.at(0)?.id || null;

	function defaultCellOutput(): CellOutput {
		return {
			value: '',
			error: '',
			result: '',
			executionCount: ' ',
		}
	}

	function defaultCell(): Cell {
		return {
			id: uuidv4() as UUID,
			code: { value: '', syntax: 'code', },
			meta: {},
			output: defaultCellOutput(),
		}
	}

	function defaultGroup(): Group {
		return {
			id: uuidv4() as UUID,
			name: 'untitled group',
			cells: [],
			dependentGroups: [],
			nextGroup: null,
		}
	}

	function defaultNoteDAG(): NoteDAG {
		const firstGroup = defaultGroup();
		const root = firstGroup.id;
		let ret: NoteDAG = {
			cells: {},
			groups: {},
			root,
		};
		ret.groups[root] = firstGroup;
		return ret;
	}

	function NoteDAGFromJSON(contents: string): NoteDAG {
		try {
			return JSON.parse(contents) as NoteDAG;
		} catch (e) {
			//console.error('failed to parse NoteDag from JSON:', e);
			console.log('failed to parse NoteDAG from JSON'); 
			return defaultNoteDAG();
		}
	}

	/// views
	function getActiveGroupChain(notedag: NoteDAG): Group[] {
		let ret = [];
		let id: string | null = notedag.root;
		console.log('starting from', id);
		while (id !== null) {
			ret.push(notedag.groups[id]);
			id = notedag.groups[id].nextGroup;
		}
		return ret;
	}

	$: activeGroupChain = getActiveGroupChain(notedag);

	/// handlers
	function addNewCell(groupId: UUID, idx?: number) {
		const newCell = defaultCell();
		notedag.cells[newCell.id] = newCell;
		if (idx === undefined) notedag.groups[groupId].cells.push(newCell.id);
		else notedag.groups[groupId].cells.splice(idx, 0, newCell.id);
		notedag = notedag;
	}

	function addNewGroup(groupId?: UUID) {
		const newGroup = defaultGroup();
		notedag.groups[newGroup.id] = newGroup;
		
		if (groupId === undefined) {
			var parent = notedag.groups[focusedGroup];
		} else {
			var parent = notedag.groups[groupId];
		}
		parent.dependentGroups.push(newGroup.id);
		parent.nextGroup = newGroup.id;
		notedag = notedag;
	}

	async function deleteCell(cellId: UUID, groupId: UUID) {
		delete notedag.cells[cellId];
		if (focusedCell === cellId) focusedCell = null;

		let group = notedag.groups[groupId];
		const idx = group.cells.indexOf(cellId);
		group.cells.splice(idx, 1);
		notedag = notedag;
	}

	async function deleteGroup(groupId: UUID, parentGroupId: UUID) {
		if (groupId == notedag.root) {
			alert('cannot delete root group');
			return;
		}

		const group = notedag.groups[groupId];
		if (group.dependentGroups.length > 0) {
			alert('cannot delete group with dependent groups');
			return;
		}

		delete notedag.groups[groupId];
		if (focusedGroup === groupId) focusedGroup = notedag.root;

		let parent = notedag.groups[parentGroupId];
		const idx = parent.dependentGroups.indexOf(groupId);
		parent.dependentGroups.splice(idx, 1);
		if (parent.nextGroup === groupId) parent.nextGroup = parent.dependentGroups[Math.min(idx, parent.dependentGroups.length-1)] || null;

		notedag = notedag;
	}

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

	function clearOutput() {
		for (let id in notedag.cells) {
			notedag.cells[id].output = defaultCellOutput();
		}
		notedag = notedag;
	}

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
							case 'data':
								let json: Record<string, string> = JSON.parse(value);
								const contentTypeHandler: Record<string, ((s: string) => string)> = {
									'text/plain': (s: string) => {
										let pre = document.createElement('pre');
										pre.innerText = s;
										return pre.outerHTML;
									},
									'image/png': (s: string) => {
										let img = document.createElement('img');
										img.src = 'data:image/png;base64,' + s;
										return img.outerHTML;
									}
								};
								cell.output.result = '';
								for (let [k, v] of Object.entries(json)) {
									cell.output.result += contentTypeHandler[k](v);
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

	async function runCell(cell: Cell) {
		await connect();

		console.log('sending', cell.code.value);
		notedag = notedag;

		if (connection.ws !== null) {
			cell.output = defaultCellOutput();
			cell.output.executionCount = '.';

			const ws = connection.ws;
			ws.send(JSON.stringify({
				id: cell.id,
				value: cell.code.value,
			}));
		}
	}

	async function runGroup(group: Group) {
		for (let id of group.cells) {
			await runCell(notedag.cells[id]);
		}
	}

	async function runActiveGroupChain() {
		for (let group of activeGroupChain) {
			await runGroup(group);
		}
	}
</script>

<div class="4xl: max-w-4xl mx-auto p-4">
	<h1>NoteDag</h1>

	<h2>Viewer</h2>

	<!--<p>{JSON.stringify(notedag)}</p>-->
	<!--<p>{focusedGroup}</p>-->
	<!--<p>{JSON.stringify(activeGroupChain)}</p>-->

	<div class="border border-2 flex flex-col">
		<div class="flex">
			<span class="px-4 py-2">Status: {connection.status}</span>
			<span class="flex-1"></span>
			<input type="button" class="px-4 py-2 clickable" value="Connect" on:click={(_event) => connect()}/>
			<input type="button" class="px-4 py-2 clickable" value="Add Group" on:click={(_event) => addNewGroup()}/>
			<input type="button" class="px-4 py-2 clickable" value="Save" on:click={(_event) => save(data.root)}/>
			<input type="button" class="px-4 py-2 clickable" value="Run All" on:click={(_event) => runActiveGroupChain()}/>
			<input type="button" class="px-4 py-2 clickable" value="Clear All" on:click={(_event) => clearOutput()}/>
		</div>

		{#if notedag === null }
			<p>error in notedag</p>
		{:else}
			<ul class="flex flex-col">
				{#each activeGroupChain as group, idx}
					<li class="m-2" on:click={(_event) => {focusedGroup = group.id}}>
						<ul class="flex">
							{#if idx === 0}
								<span class="border border-2 px-2 border-red-500" contenteditable bind:innerText={group.name}></span>
							{:else}
								{#each activeGroupChain[idx-1].dependentGroups as dependentId}
									{#if dependentId === group.id}
										<span class="border border-2 px-2 border-red-500" contenteditable bind:innerText={group.name}></span>
									{:else}
										<span class="border border-2 px-2 clickable" on:click={(_event) => {activeGroupChain[idx-1].nextGroup = dependentId; notedag = notedag}}>
											{notedag.groups[dependentId].name}
										</span>
									{/if}
								{/each}
								<input type="button" class="px-2 clickable" value="+" on:click={(_event) => addNewGroup(activeGroupChain[idx-1]?.id)}/>
							{/if}
							<span class="flex-1"></span>
							<span class="border border-2 px-2 clickable" on:click={(_) => addNewCell(group.id)}>Add Cell</span>
						</ul>

						<ul class="border border-2 flex flex-col p-2 {focusedGroup === group.id ? 'border-red-500' : ''}">
							<div class="mb-4 flex">
								<pre>{group.id}</pre>
								<span class="flex-1"></span>
								<span class="ml-2 px-2 clickable" on:click={(_) => runGroup(group)}>Run All</span>
								{#if idx > 0}
									<span class="ml-2 px-2 clickable" on:click={(_) => deleteGroup(group.id, activeGroupChain[idx-1].id)}>Delete</span>
								{/if}
							</div>

							{#each group.cells.map(id => notedag.cells[id]) as cell}
								<li class="flex">
									<div class="flex flex-col mx-2">
										<pre class="mx-2">[{cell.output.executionCount}]</pre>
									</div>
									<div class="flex-1 flex flex-col mx-2">
										<pre class="p-2 bg-slate-100" contenteditable bind:innerText={cell.code.value}></pre>
										<!-- FIXME: this is vulnerable to XSS. Ok if we're just running local (trusted) notebooks but we should really fix it -->
										<div class="p-2">
											<pre>{@html cell.output.value}</pre>
											<pre>{@html cell.output.error}</pre>
											<div>{@html cell.output.result}</div>
										</div>
									</div>
									<ul class="flex flex-col mx-2">
										<input type="button" class="clickable" value="X" on:click={(_event) => deleteCell(cell.id, group.id)}/>
										<input type="button" class="clickable" value="Run" on:click={(_event) => runCell(cell)}/>
									</ul>
								</li>
							{/each}
						</ul>
					</li>
				{/each}
			</ul>
		{/if}
	</div>
</div>
