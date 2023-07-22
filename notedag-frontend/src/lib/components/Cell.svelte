<script lang="ts">
	import { EditorView, basicSetup } from "codemirror"
	import { python } from "@codemirror/lang-python"

	import { onMount } from 'svelte';

	interface CellCode {
		value: string;
		syntax: string;
	}

	export let state: CellCode;

	let editorParent: HTMLDivElement;
	let editor: EditorView;

	onMount(() => {
		editor = new EditorView({
			extensions: [basicSetup, python()],
			parent: editorParent,
			doc: state.value,
		})
	});
</script>

<div bind:this={editorParent}></div>
