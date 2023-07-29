<script lang="ts">
	import {basicSetup} from "codemirror"
	import {EditorView, keymap} from "@codemirror/view"
	import {EditorState} from "@codemirror/state"
	import {indentWithTab} from "@codemirror/commands"

	import { python } from "@codemirror/lang-python"

	import { EditorMode } from "$lib";

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
							run: () => {
								dispatch('run');
								return true;
							},
						},
						{
							key: 'Ctrl-Enter',
							run: () => {
								dispatch('run');
								return true;
							},
							preventDefault: true,
						},
						{
							key: 'Escape',
							run: () => {
								dispatch('escape');
								return true;
							},
						},
					]),
					python(),
					EditorView.lineWrapping,
					EditorView.updateListener.of((update) => {
						if (update.focusChanged) {
							dispatch('mode', editor.hasFocus ? EditorMode.INSERT : EditorMode.NORMAL);
						}
						state.value = update.state.doc.toJSON().join('\n');
					}),
				],
			}),
			parent: editorParent,
		})
	});

	export function edit() {
		editor.focus();
	}
</script>

<div bind:this={editorParent} on:focus={() => { alert(); dispatch('focus') }}></div>

<style lang="postcss">
	:global(.cm-editor) {
		@apply text-sm border border-slate-100 bg-slate-50;
	}
	:global(.cm-editor .cm-gutters .cm-lineNumbers) {
		@apply px-1;
	}
	:global(.cm-editor .cm-gutters, .cm-editor .cm-gutterElement.cm-activeLineGutter) {
		@apply bg-slate-200;
	}
	:global(.cm-editor .cm-line.cm-activeLine) {
		@apply border-r-4 bg-slate-50 border-slate-300;
	}

	:global(.cm-editor.cm-focused) {
		@apply border-blue-100 bg-blue-50;
	}
	:global(.cm-editor.cm-focused .cm-gutters, .cm-editor.cm-focused .cm-gutterElement.cm-activeLineGutter) {
		@apply bg-blue-100;
	}
	:global(.cm-editor.cm-focused .cm-line.cm-activeLine) {
		@apply border-r-4 bg-blue-100 border-blue-300;
	}

	:global(.cm-editor .cm-content) {
		@apply p-2;
	}

</style>
