<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { fade, fly } from 'svelte/transition';
	import { getChecklist, getRun, deleteRun, type Run } from '$lib/api';
	import CheckboxItem from '$lib/components/CheckboxItem.svelte';
	import SectionGroup from '$lib/components/SectionGroup.svelte';
	import ProgressBar from '$lib/components/ProgressBar.svelte';
	import TopLoader from '$lib/components/TopLoader.svelte';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
	import Toast from '$lib/components/Toast.svelte';

	let template = $derived($page.params.template as string);
	let filename = $derived($page.params.filename as string);

	let run = $state<Run | null>(null);
	let checklistTitle = $state('');
	let loading = $state(true);
	let error = $state('');
	let showDeleteConfirm = $state(false);
	let toast = $state<{ message: string; type: 'success' | 'error' } | null>(null);

	let totalItems = $derived(run ? run.sections.reduce((sum, s) => sum + s.items.length, 0) : 0);
	let checkedItems = $derived(
		run ? run.sections.reduce((sum, s) => sum + s.items.filter((i) => i.checked).length, 0) : 0
	);

	$effect(() => {
		loadRun(template, filename);
	});

	function formatRunDate(value: string): string {
		const [date, time] = value.split(' ');
		return date && time ? `${date} | ${time}` : value;
	}

	async function loadRun(t: string, f: string) {
		try {
			const loadedRun = await getRun(t, f);
			run = loadedRun;
			checklistTitle = loadedRun.meta.template;
			try {
				const checklist = await getChecklist(loadedRun.meta.template);
				checklistTitle = checklist.meta.title;
			} catch {
				// Keep the slug fallback if the source checklist was removed.
			}
			loading = false;
		} catch (e) {
			error = String(e);
			loading = false;
		}
	}

	async function handleDelete() {
		showDeleteConfirm = false;
		try {
			await deleteRun(template, filename);
			goto('/');
		} catch (e) {
			toast = { message: String(e), type: 'error' };
		}
	}
</script>

<div class="page">
	{#if loading}
		<div in:fade><TopLoader /></div>
	{:else if error}
		<p class="error" in:fade>{error}</p>
	{:else if run}
		<div in:fade={{ duration: 200 }}>
			<div class="run-info">
				<h2 class="run-title">{run.meta.title}</h2>
				<div class="run-meta">
					<span class="meta-item">Saved {formatRunDate(run.meta.date)}</span>
					<span class="meta-item">{checklistTitle}</span>
				</div>
				{#if run.meta.notes}
					<p class="run-notes">{run.meta.notes}</p>
				{/if}
			</div>

			<div class="progress-section">
				<ProgressBar checked={checkedItems} total={totalItems} />
			</div>

			<div class="sections">
				{#each run.sections as section, si}
					<SectionGroup heading={section.heading}>
						{#each section.items as item, ii}
							<div in:fly={{ y: 10, duration: 200, delay: (si * 3 + ii) * 30 }}>
								<CheckboxItem
									title={item.title}
									description={item.description}
									code={item.code}
									codeLang={item.codeLang}
									note={item.note}
									skipReason={item.skipReason}
									checked={item.checked}
									disabled
								/>
							</div>
						{/each}
					</SectionGroup>
				{/each}
			</div>

			<button class="delete-btn" onclick={() => (showDeleteConfirm = true)} type="button">
				Delete Run
			</button>
		</div>
	{/if}
</div>

{#if showDeleteConfirm}
	<ConfirmDialog
		message="Are you sure you want to delete this run? This cannot be undone."
		confirmLabel="Delete"
		cancelLabel="Keep"
		onconfirm={handleDelete}
		oncancel={() => (showDeleteConfirm = false)}
	/>
{/if}

{#if toast}
	<Toast message={toast.message} type={toast.type} onclose={() => (toast = null)} />
{/if}

<style>
	.page {
		padding-bottom: 20px;
	}
	.run-info {
		margin-bottom: 16px;
	}
	.run-title {
		margin: 0;
		font-size: 18px;
		font-weight: 700;
		color: var(--text-primary);
	}
	.run-meta {
		display: flex;
		gap: 12px;
		margin-top: 4px;
	}
	.meta-item {
		font-size: 13px;
		color: var(--text-secondary);
		font-variant-numeric: tabular-nums;
	}
	.run-notes {
		margin: 8px 0 0;
		font-size: 13px;
		color: var(--text-secondary);
		line-height: 1.4;
		font-style: italic;
	}
	.progress-section {
		margin-bottom: 20px;
	}
	.sections {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	.error {
		color: var(--danger);
		font-size: 14px;
	}

	.delete-btn {
		width: 100%;
		padding: 11px 12px;
		border: 1px solid var(--danger);
		border-radius: 10px;
		background: transparent;
		color: var(--danger);
		font-family: inherit;
		font-size: 14px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.15s;
		margin-top: 24px;
	}
	.delete-btn:hover {
		background: var(--danger);
		color: #fff;
	}
</style>
