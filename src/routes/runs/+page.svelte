<script lang="ts">
	import { goto } from '$app/navigation';
	import { fly, fade } from 'svelte/transition';
	import { listRuns, listChecklists, type ChecklistSummary, type RunSummary } from '$lib/api';
	import RunCard from '$lib/components/RunCard.svelte';
	import TopLoader from '$lib/components/TopLoader.svelte';

	let runs = $state<RunSummary[]>([]);
	let checklistTitles = $state<Record<string, string>>({});
	let loading = $state(true);
	let error = $state('');
	let query = $state('');

	function formatRunDate(value: string): string {
		const [date, time] = value.split(' ');
		return date && time ? `${date} | ${time}` : value;
	}

	function buildChecklistTitleMap(checklists: ChecklistSummary[]): Record<string, string> {
		return Object.fromEntries(checklists.map((checklist) => [checklist.slug, checklist.title]));
	}

	let filtered = $derived(() => {
		if (!query.trim()) return runs;
		const q = query.toLowerCase();
		return runs.filter(
			(r) => r.title.toLowerCase().includes(q) || r.template.toLowerCase().includes(q)
		);
	});

	$effect(() => {
		Promise.all([listRuns(), listChecklists()])
			.then(([runData, checklistData]) => {
				runs = runData;
				checklistTitles = buildChecklistTitleMap(checklistData);
				loading = false;
			})
			.catch((e) => {
				error = String(e);
				loading = false;
			});
	});
</script>

<div class="page">
	{#if loading}
		<div in:fade><TopLoader /></div>
	{:else if error}
		<p class="error" in:fade>{error}</p>
	{:else if runs.length === 0}
		<div class="empty" in:fade>
			<p>No runs yet.</p>
			<p class="hint">Complete a checklist run to see it here.</p>
		</div>
	{:else}
		{#if runs.length > 1}
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
				<input type="text" class="search" placeholder="Search runs…" bind:value={query} />
			</div>
		{/if}
		<div class="list">
			{#each filtered() as run, i (run.template + '/' + run.filename)}
				<div in:fly={{ y: 20, duration: 300, delay: i * 60 }}>
					<RunCard
						title={run.title}
						templateLabel={checklistTitles[run.template] ?? run.template}
						date={formatRunDate(run.date)}
						checked={run.checked}
						total={run.total}
						onclick={() => goto(`/runs/${run.template}/${run.filename}`)}
					/>
				</div>
			{:else}
				<p class="no-results">No runs match "{query}"</p>
			{/each}
		</div>
	{/if}
</div>

<style>
	.page {
		min-height: 100%;
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
	.empty {
		text-align: center;
		padding-top: 40px;
		color: var(--text-secondary);
		font-size: 14px;
	}
	.hint {
		font-size: 13px;
		margin-top: 4px;
	}
</style>
