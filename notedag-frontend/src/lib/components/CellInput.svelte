<script lang="ts">
	import {basicSetup} from "codemirror"
	import {EditorView, keymap} from "@codemirror/view"
	import {EditorState} from "@codemirror/state"
	import {indentWithTab} from "@codemirror/commands"

	import { python } from "@codemirror/lang-python"

	import { createEventDispatcher, onMount } from 'svelte';
	const dispatch = createEventDispatcher();

	interface CellCode {
		value: string;
		syntax: string;
	}

	export let state: CellCode;

	let editorParent: HTMLDivElement;
	let editor: EditorView;

	onMount(() => {
		editor = new EditorView({
			state: EditorState.create({
				doc: state.value,
				extensions: [
					basicSetup,
					keymap.of([
						indentWithTab,
						{
							key: 'Shift-Enter',
							run: () => dispatch('run'),
						}
					]),
					python(),
					EditorView.lineWrapping,
					EditorView.updateListener.of((update) => {
						state.value = update.state.doc.toJSON().join('\n');
					}),
				],
			}),
			parent: editorParent,
		})
	});
</script>

<div bind:this={editorParent}></div>
