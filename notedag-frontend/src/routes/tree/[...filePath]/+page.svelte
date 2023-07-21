<script>
    /** @type {import('./$types').PageData} */
	export let data;

	function getLoc(pathname, fname) {
		let n = pathname.length;
		if (n > 0 && pathname[n - 1] != '/') {
			pathname += '/'
		}
		return pathname + fname;
	}

	async function addNewND(pathname, event) {
		const newFile = getLoc(pathname, 'untitled.ind');

		const response = await fetch("/api/create", {
			method: "POST",
			body: JSON.stringify({ filePath: newFile }),
			headers: {
				"Content-Type": "application/json",
			},
		});

		const newPath = `/notedags/${newFile}`;

		window.location.pathname = newPath;
	}
</script>

<div class="max-w-2xl mx-auto p-4">
	<h1>NoteDag</h1>

	<h2>Browser</h2>

	<div class="border border-2 flex flex-col">
		<div class="flex">
			<input class="flex-1 px-4 py-2" value="."/>
			<input type="button" class="px-4 py-2 clickable" value="+" on:click={(event) => addNewND(data.root, event)}/>
		</div>
		<hr>
		<ul class="list-none flex flex-col">
			{#if getLoc(data.root, '') !== '' }
			<a 
				class="px-4 py-2 clickable"
				href={`/tree/${getLoc(data.root, '..')}`}
			>
				<li>..</li>
			</a>
			{/if}

			{#each data.files as {fileName, isDir, size, modified}}
				<a
					class="px-4 py-2 clickable"
					href={`/${isDir ? 'tree' : 'notedags'}/${getLoc(data.root, fileName)}`}
				>
					<li class="flex">
						<span class={isDir ? "" : "font-bold"}>{fileName}</span>
						<span class="flex-1"></span>
						{#if !isDir}
							<span class="pl-2 text-slate-500">{(size/1000).toFixed(2)} kB</span>
						{/if}
						<span class="pl-2 text-slate-500">{new Date(modified).toLocaleString()}</span>
					</li>
				</a>
			{/each}
		</ul>
	</div>
</div>
