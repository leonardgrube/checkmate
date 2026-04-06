<script lang="ts">
	import TagBadge from './TagBadge.svelte';

	let {
		title,
		description,
		tags = [],
		onclick,
		onedit
	}: {
		title: string;
		description?: string;
		tags?: string[];
		onclick: () => void;
		onedit: () => void;
	} = $props();
</script>

<div class="card-wrap">
	<button class="card" {onclick} type="button">
		<span class="title">{title}</span>
		{#if description}
			<span class="desc">{description}</span>
		{/if}
		{#if tags.length > 0}
			<div class="tags">
				{#each tags as tag}
					<TagBadge {tag} />
				{/each}
			</div>
		{/if}
	</button>
	<button
		class="edit-btn"
		onclick={(e) => {
			e.stopPropagation();
			onedit();
		}}
		type="button"
		title="Edit checklist"
	>
		<svg
			viewBox="0 0 24 24"
			width="16"
			height="16"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7" />
			<path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z" />
		</svg>
	</button>
</div>

<style>
	.card-wrap {
		display: flex;
		gap: 0;
		align-items: stretch;
	}

	.card {
		display: flex;
		flex-direction: column;
		gap: 6px;
		flex: 1;
		padding: 14px 16px;
		border: 1px solid var(--border);
		border-radius: 10px 0 0 10px;
		background: var(--bg-card);
		color: inherit;
		cursor: pointer;
		text-align: left;
		font-family: inherit;
		transition: all 0.2s;
	}
	.card:hover {
		background: var(--bg-hover);
		border-color: var(--border);
	}
	.card:active {
		background: var(--border);
	}

	.edit-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 44px;
		border: 1px solid var(--border);
		border-left: none;
		border-radius: 0 10px 10px 0;
		background: var(--bg-card);
		color: var(--text-secondary);
		cursor: pointer;
		transition: all 0.2s;
		flex-shrink: 0;
	}
	.edit-btn:hover {
		background: var(--bg-hover);
		color: var(--accent);
	}
	.edit-btn:active {
		background: var(--border);
	}

	.title {
		font-size: 15px;
		font-weight: 600;
		color: var(--text-primary);
	}
	.desc {
		font-size: 13px;
		color: var(--text-secondary);
		line-height: 1.4;
	}
	.tags {
		display: flex;
		gap: 4px;
		flex-wrap: wrap;
		margin-top: 2px;
	}
</style>
