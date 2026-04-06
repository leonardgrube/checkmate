<script lang="ts">
	import { fade, scale } from 'svelte/transition';

	export interface SkippedItem {
		sectionIndex: number;
		itemIndex: number;
		title: string;
		reason: string;
	}

	let {
		items = $bindable(),
		onsave,
		oncancel
	}: {
		items: SkippedItem[];
		onsave: () => void;
		oncancel: () => void;
	} = $props();
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" transition:fade={{ duration: 150 }} onmousedown={oncancel}>
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="dialog"
		transition:scale={{ duration: 200, start: 0.95 }}
		onmousedown={(e) => e.stopPropagation()}
	>
		<h3 class="title">Skipped Items</h3>
		<p class="subtitle">Optionally add a reason for each skipped item.</p>

		<div class="items">
			{#each items as item, i}
				<div class="item">
					<span class="item-title">{item.title}</span>
					<input
						type="text"
						class="input"
						placeholder="Reason (optional)"
						bind:value={items[i].reason}
					/>
				</div>
			{/each}
		</div>

		<div class="actions">
			<button class="btn back" onclick={oncancel}>Back</button>
			<button class="btn save" onclick={onsave}>Save Run</button>
		</div>
	</div>
</div>

<style>
	.overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 900;
		padding: 20px;
	}
	.dialog {
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: 12px;
		padding: 20px;
		max-width: 420px;
		width: 100%;
		max-height: 80vh;
		overflow-y: auto;
	}
	.title {
		margin: 0 0 4px 0;
		font-size: 16px;
		font-weight: 700;
		color: var(--text-primary);
	}
	.subtitle {
		margin: 0 0 16px 0;
		font-size: 13px;
		color: var(--text-secondary);
	}

	.items {
		display: flex;
		flex-direction: column;
		gap: 12px;
		margin-bottom: 20px;
	}
	.item {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.item-title {
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
	}
	.input {
		width: 100%;
		padding: 8px 10px;
		border: 1px solid var(--border);
		border-radius: 8px;
		background: var(--bg-card);
		color: var(--text-primary);
		font-family: inherit;
		font-size: 13px;
		outline: none;
		transition: border-color 0.15s;
	}
	.input:focus {
		border-color: var(--accent);
	}
	.input::placeholder {
		color: var(--text-secondary);
	}

	.actions {
		display: flex;
		gap: 8px;
	}
	.btn {
		flex: 1;
		padding: 10px;
		border: none;
		border-radius: 8px;
		font-family: inherit;
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.15s;
	}
	.back {
		background: var(--bg-hover);
		color: var(--text-primary);
	}
	.back:hover {
		filter: brightness(1.2);
	}
	.save {
		background: var(--accent);
		color: #0f0f0f;
		font-weight: 700;
	}
	.save:hover {
		filter: brightness(1.1);
	}
</style>
