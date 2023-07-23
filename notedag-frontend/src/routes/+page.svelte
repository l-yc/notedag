<script>
    /** @type {import('./$types').PageData} */    export let data;
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';

	onMount(() => goto('tree'));
</script>

<div class="max-w-2xl mx-auto p-4">
<h1>NoteDag</h1>

<p>Loading...</p>
<a href="/tree">tree</a>
</div>
