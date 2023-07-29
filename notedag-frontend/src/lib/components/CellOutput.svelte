<script lang="ts">
	export interface CellOutputs {
		value: string;
		error: string;
		result: string;
	}

	export let state: CellOutputs;
</script>

<!-- FIXME: this is vulnerable to XSS. Ok if we're just running local (trusted) notebooks but we should really fix it -->
<div class="p-2">
	<pre>{@html state.value}</pre>
	<pre>{@html state.error}</pre>
	<div>{@html state.result}</div>
</div>
