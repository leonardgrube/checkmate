<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		heading,
		children
	}: {
		heading: string;
		children: Snippet;
	} = $props();
</script>

{#if heading}
	<div class="section">
		<h3 class="heading">{heading}</h3>
		<div class="items">
			{@render children()}
		</div>
	</div>
{:else}
	<div class="items">
		{@render children()}
	</div>
{/if}

<style>
	.section {
		margin-bottom: 20px;
	}
	.heading {
		font-size: 13px;
		font-weight: 600;
		color: var(--accent);
		text-transform: uppercase;
		letter-spacing: 0.05em;
		margin: 0 0 6px 0;
		padding: 0 12px;
	}
	.items {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
</style>
