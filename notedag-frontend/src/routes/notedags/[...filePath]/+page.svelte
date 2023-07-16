<script lang="ts">
    /** @type {import('./$types').PageData} */
	export let data;

	let notedag = NoteDAGFromJSON(data.contents);

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
			code: { value: '', syntax: 'code', },
			meta: {},
			output: {},
		});

		notedag = notedag;
	}

	async function save(filePath) {
		const response = await fetch("/api/write", {
			method: "POST",
			body: JSON.stringify({ filePath, contents: JSON.stringify(notedag) }),
			headers: {
				"Content-Type": "application/json",
			},
		});
	}
</script>

<div class="max-w-2xl mx-auto p-4">
	<h1>NoteDag</h1>

	<h2>Browser</h2>

	<div class="border border-2 flex flex-col">
		<div>
			<input type="button" class="px-4 py-2 clickable" value="+" on:click={(event) => addNewCell(data.root, event)}/>
			<input type="button" class="px-4 py-2 clickable" value="Save" on:click={(event) => save(data.root)}/>
		</div>

		{#if notedag === null }
			<p>error in notedag</p>
		{:else}
			<ul class="flex flex-col">
				{#each notedag.cells as {code, meta, output}}
					<li class="flex">
						<div>
							<pre class="m-2">{JSON.stringify(meta)}</pre>
						</div>
						<div class="flex-1 flex flex-col">
							<pre class="m-2 p-2 bg-slate-100" contenteditable bind:innerText={code.value}></pre>
							<pre class="m-2">{JSON.stringify(output)}</pre>
						</div>
					</li>
				{/each}
			</ul>
		{/if}
	</div>
</div>
