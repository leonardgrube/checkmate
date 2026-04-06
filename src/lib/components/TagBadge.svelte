<script lang="ts">
	let { tag }: { tag: string } = $props();
</script>

<span class="tag">{tag}</span>

<style>
	.tag {
		display: inline-block;
		padding: 2px 8px;
		border-radius: 10px;
		background: var(--accent-dim);
		color: var(--accent);
		font-size: 11px;
		font-weight: 500;
		letter-spacing: 0.02em;
	}
</style>
