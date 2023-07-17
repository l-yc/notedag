<script lang="ts">
    /** @type {import('./$types').PageData} */
	export let data;
	import { kernel } from '$lib';
	import { v4 as uuidv4 } from 'uuid';
	import Convert from 'ansi-to-html';

	interface CellCode {
		value: string;
		syntax: string;
	}

	interface CellOutput {
		value: string;
		error: string;
		executionCount: string;
	}

	interface Cell {
		id: string;
		code: CellCode;
		meta: object;
		output: CellOutput;
	}

	interface NoteDAG {
		cells: Cell[];
	}

	let notedag: NoteDAG = NoteDAGFromJSON(data.contents);

	function NoteDAGFromJSON(contents: string) {
		try {
			return JSON.parse(contents);
		} catch (e) {
			console.error('failed to parse NoteDag from JSON:', e);
			return { cells: [] };
		}
	}

	function addNewCell() {
		notedag.cells.push({
			id: uuidv4(),
			code: { value: '', syntax: 'code', },
			meta: {},
			output: { value: '', error: '',  executionCount: ' ' },
		});

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
	}

	function defaultOutput() {
return {
				value: '',
				error: '',
				executionCount: ' ',
			}
	}

	function clearOutput() {
		for (let cell of notedag.cells) {
			cell.output = defaultOutput();
		}
		notedag = notedag;
	}

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
						for (let cell of notedag.cells) {
							if (cell.id === id) {
								switch (name) {
									case 'output':
									case 'error':
										let escaped = new Option(value).innerHTML;
										var convert = new Convert();
										cell.output.value = convert.toHtml(escaped);
										break;
									case 'running':
									case 'complete':
										cell.output.executionCount = value;
										break;
								}
							}
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

	async function run(cell: Cell) {
		await connect();

		console.log('sending', cell.code.value);
		notedag = notedag;

		if (connection.ws !== null) {
			cell.output = defaultOutput();
			cell.output.executionCount = '.';

			const ws = connection.ws;
			ws.send(JSON.stringify({
				id: cell.id,
				value: cell.code.value,
			}));
		}
	}
</script>

<div class="4xl: max-w-4xl mx-auto p-4">
	<h1>NoteDag</h1>

	<h2>Viewer</h2>

	<div class="border border-2 flex flex-col">
		<div class="flex">
			<span class="px-4 py-2">Status: {connection.status}</span>
			<span class="flex-1"></span>
			<input type="button" class="px-4 py-2 clickable" value="Connect" on:click={(event) => connect()}/>
			<input type="button" class="px-4 py-2 clickable" value="Add Cell" on:click={(event) => addNewCell()}/>
			<input type="button" class="px-4 py-2 clickable" value="Clear All" on:click={(event) => clearOutput()}/>
			<input type="button" class="px-4 py-2 clickable" value="Save" on:click={(event) => save(data.root)}/>
		</div>

		{#if notedag === null }
			<p>error in notedag</p>
		{:else}
			<ul class="flex flex-col">
				{#each notedag.cells as cell}
					<li class="flex">
						<div class="flex flex-col">
							<pre class="m-2">[{cell.output.executionCount}]</pre>
							<input type="button" class="px-4 py-2 clickable" value="Run" on:click={(event) => run(cell)}/>
						</div>
						<div class="flex-1 flex flex-col">
							<pre class="m-2 p-2 bg-slate-100" contenteditable bind:innerText={cell.code.value}></pre>
							<!-- FIXME: this is vulnerable to XSS. Ok if we're just running local (trusted) notebooks but we should really fix it -->
							<pre class="m-2">{@html cell.output.value}</pre>
						</div>
					</li>
				{/each}
			</ul>
		{/if}
	</div>
</div>
