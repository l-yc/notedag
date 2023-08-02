<script>
    /** @type {import('./$types').PageData} */
	export let data;
	import Header from "$lib/components/Header.svelte";
	import { api } from '$lib';

	$: cwd = data.root === '' ? '.' : './' + data.root;

	function getLoc(pathname, fname) {
		let n = pathname.length;
		if (n > 0 && pathname[n - 1] != '/') {
			pathname += '/'
		}
		return pathname + fname;
	}

	async function addNewND(pathname, event) {
		const newFile = getLoc(pathname, 'untitled.ind');

		const _ = await api.post('notedag/create', { filePath: newFile });

		const newPath = `/notedags/${newFile}`;

		window.location.pathname = newPath;
	}
</script>

<div>
	<Header>
		<div slot="annotation" >
			<a href="#" class="ml-4 text-xl">tree</a>
		</div>
		<div slot="toolbar" class="flex constrained">
			<label class="px-3 py-1">CWD:</label>
			<input class="flex-1 px-2" disabled bind:value={cwd}/>
			<input type="button" class="px-3 py-1 clickable" value="New NoteDAG" on:click={(event) => addNewND(data.root, event)}/>
		</div>
	</Header>

	<ul class="list-none flex flex-col border-2 constrained">
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
