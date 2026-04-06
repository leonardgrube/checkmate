<script lang="ts">
	import { tweened } from 'svelte/motion';
	import { cubicOut } from 'svelte/easing';

	let { checked, total }: { checked: number; total: number } = $props();

	const progress = tweened(0, { duration: 400, easing: cubicOut });

	$effect(() => {
		progress.set(total > 0 ? checked / total : 0);
	});

	let pct = $derived(Math.round($progress * 100));
	let allDone = $derived(checked === total && total > 0);
</script>

<div class="progress-wrap">
	<div class="progress-bar">
		<div class="progress-fill" class:done={allDone} style="width: {pct}%"></div>
	</div>
	<span class="progress-label" class:done={allDone}>
		{checked}/{total} complete
	</span>
</div>

<style>
	.progress-wrap {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.progress-bar {
		flex: 1;
		height: 6px;
		background: var(--bg-hover);
		border-radius: 3px;
		overflow: hidden;
	}
	.progress-fill {
		height: 100%;
		background: var(--accent);
		border-radius: 3px;
		transition: background 0.3s;
	}
	.progress-fill.done {
		background: var(--accent);
		box-shadow: 0 0 8px var(--accent-dim);
	}
	.progress-label {
		font-size: 12px;
		color: var(--text-secondary);
		white-space: nowrap;
		transition: color 0.3s;
	}
	.progress-label.done {
		color: var(--accent);
	}
</style>
