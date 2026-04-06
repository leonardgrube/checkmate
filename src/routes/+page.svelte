<script lang="ts">
	import { goto } from '$app/navigation';
	import { fly, fade } from 'svelte/transition';
	import {
		addSampleChecklist,
		listChecklists,
		listenChecklistChanges,
		type ChecklistSummary
	} from '$lib/api';
	import ChecklistCard from '$lib/components/ChecklistCard.svelte';
	import TopLoader from '$lib/components/TopLoader.svelte';

	let checklists = $state<ChecklistSummary[]>([]);
	let loading = $state(true);
	let addingSample = $state(false);
	let error = $state('');
	let query = $state('');

	let filtered = $derived(() => {
		if (!query.trim()) return checklists;
		const q = query.toLowerCase();
		return checklists.filter(
			(cl) =>
				cl.title.toLowerCase().includes(q) ||
				cl.description?.toLowerCase().includes(q) ||
				cl.tags.some((t) => t.toLowerCase().includes(q))
		);
	});

	async function loadChecklists(showLoading = false) {
		if (showLoading) {
			loading = true;
		}

		try {
			checklists = await listChecklists();
			error = '';
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}

	async function handleAddSampleChecklist() {
		addingSample = true;
		try {
			await addSampleChecklist();
			await loadChecklists(true);
			error = '';
		} catch (e) {
			error = String(e);
		} finally {
			addingSample = false;
		}
	}

	$effect(() => {
		void loadChecklists(true);

		let cancelled = false;
		let unlisten: (() => void) | undefined;

		listenChecklistChanges(() => {
			if (!cancelled) {
				void loadChecklists();
			}
		})
			.then((dispose) => {
				if (cancelled) {
					dispose();
					return;
				}
				unlisten = dispose;
			})
			.catch(() => {}); // expected: effect cleanup may race with listener setup

		return () => {
			cancelled = true;
			unlisten?.();
		};
	});
</script>

<div class="page">
	{#if loading}
		<div in:fade><TopLoader /></div>
	{:else if error}
		<div class="error" in:fade>
			<p>{error}</p>
			<a href="/settings">Configure data repo →</a>
		</div>
	{:else if checklists.length === 0}
		<div class="empty" in:fade>
			<p class="empty-text">You don't have any checklists yet.</p>
			<button class="empty-btn" onclick={() => goto('/create')}>Create your first checklist</button>
			<button
				class="empty-btn outline"
				onclick={() => void handleAddSampleChecklist()}
				disabled={addingSample}
			>
				{addingSample ? 'Adding starter checklist…' : 'Add starter checklist'}
			</button>
		</div>
	{:else}
		{#if checklists.length > 1}
			<div class="search-wrap">
				<svg
					class="search-icon"
					viewBox="0 0 24 24"
					width="16"
					height="16"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<circle cx="11" cy="11" r="8" />
					<path d="M21 21l-4.35-4.35" />
				</svg>
				<input type="text" class="search" placeholder="Search checklists…" bind:value={query} />
			</div>
		{/if}
		<div class="list">
			{#each filtered() as cl, i (cl.slug)}
				<div in:fly={{ y: 20, duration: 300, delay: i * 60 }}>
					<ChecklistCard
						title={cl.title}
						description={cl.description}
						tags={cl.tags}
						onclick={() => goto(`/run/${cl.slug}`)}
						onedit={() => goto(`/edit/${cl.slug}`)}
					/>
				</div>
			{:else}
				<p class="no-results">No checklists match "{query}"</p>
			{/each}
		</div>
	{/if}
</div>

<style>
	.page {
		min-height: 100%;
		position: relative;
	}
	.page::after {
		content: '';
		position: fixed;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		width: 280px;
		height: 340px;
		background: url('/logo-background.png') no-repeat center;
		background-size: contain;
		opacity: 1;
		pointer-events: none;
		z-index: -1;
	}
	.search-wrap {
		position: relative;
		margin-bottom: 10px;
	}
	.search-icon {
		position: absolute;
		left: 12px;
		top: 50%;
		transform: translateY(-50%);
		color: var(--text-secondary);
		pointer-events: none;
	}
	.search {
		width: 100%;
		padding: 10px 12px 10px 36px;
		border: 1px solid var(--border);
		border-radius: 8px;
		background: var(--bg-card);
		color: var(--text-primary);
		font-family: inherit;
		font-size: 14px;
		outline: none;
		transition: border-color 0.15s;
	}
	.search:focus {
		border-color: var(--accent);
	}
	.search::placeholder {
		color: var(--text-secondary);
	}
	.list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.no-results {
		color: var(--text-secondary);
		font-size: 13px;
		text-align: center;
		padding: 20px 0;
	}
	.error {
		color: var(--danger);
		font-size: 14px;
	}
	.error a {
		color: var(--accent);
		text-decoration: none;
	}
	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 14px;
		padding-top: 40px;
	}
	.empty-text {
		color: var(--text-secondary);
		font-size: 14px;
		margin: 0;
	}
	.empty-btn {
		width: 240px;
		padding: 10px 24px;
		border: none;
		border-radius: 8px;
		background: var(--accent);
		color: #0f0f0f;
		font-family: inherit;
		font-size: 14px;
		font-weight: 700;
		cursor: pointer;
		transition: all 0.15s;
	}
	.empty-btn:hover {
		filter: brightness(1.1);
		transform: translateY(-1px);
	}
	.empty-btn:active {
		transform: translateY(0);
	}
	.empty-btn.outline {
		border: 1px solid rgba(255, 255, 255, 0.75);
		background: transparent;
		color: #f5f5f5;
	}
	.empty-btn.outline:hover {
		background: rgba(255, 255, 255, 0.06);
		filter: none;
	}
	.empty-btn:disabled {
		opacity: 0.7;
		cursor: default;
		transform: none;
	}
</style>
