<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { onDestroy } from 'svelte';
	import { untrack } from 'svelte';
	import { fade, fly } from 'svelte/transition';
	import {
		getChecklist,
		getConfig,
		listenChecklistChanges,
		saveRun,
		updateChecklist,
		type ChecklistMeta,
		type Section
	} from '$lib/api';
	import { runState } from '$lib/runState.svelte';
	import CheckboxItem from '$lib/components/CheckboxItem.svelte';
	import SectionGroup from '$lib/components/SectionGroup.svelte';
	import ProgressBar from '$lib/components/ProgressBar.svelte';
	import TopLoader from '$lib/components/TopLoader.svelte';
	import Toast from '$lib/components/Toast.svelte';
	import SkipReasonsDialog, { type SkippedItem } from '$lib/components/SkipReasonsDialog.svelte';

	type ProgressItem = {
		checked: boolean;
		note?: string;
		skipReason?: string;
	};

	type ProgressSection = {
		items: ProgressItem[];
	};

	type InlineEditTarget = {
		sectionIndex: number;
		itemIndex: number;
	};

	type InlineEditDraft = {
		title: string;
		description: string;
		code: string;
		codeLang: string;
		showCode: boolean;
	};

	let slug = $derived($page.params.slug as string);

	let checklistMeta = $state<ChecklistMeta | null>(null);
	let templateSections = $state<Section[]>([]);
	let progressSections = $state<ProgressSection[]>([]);
	let runTitle = $state('');
	let runNotes = $state('');
	let loading = $state(true);
	let saving = $state(false);
	let savingInline = $state(false);
	let error = $state('');
	let toast = $state<{ message: string; type: 'success' | 'error' } | null>(null);
	let skippedItems = $state<SkippedItem[]>([]);
	let showSkipDialog = $state(false);
	let autoScroll = $state(false);
	let editingItem = $state<InlineEditTarget | null>(null);
	let editDraft = $state<InlineEditDraft | null>(null);
	let pendingChecklistRefresh = $state(false);
	let showJumpToSave = $state(false);
	let loadVersion = 0;
	let saveFormEl = $state<HTMLElement | null>(null);
	let runTitleInputEl = $state<HTMLInputElement | null>(null);

	let mergedSections = $derived(buildMergedSections(templateSections, progressSections));
	let totalItems = $derived(mergedSections.reduce((sum, section) => sum + section.items.length, 0));
	let checkedItems = $derived(
		mergedSections.reduce(
			(sum, section) => sum + section.items.filter((item) => item.checked).length,
			0
		)
	);

	$effect(() => {
		const currentSlug = slug;
		loading = true;
		error = '';
		checklistMeta = null;
		templateSections = [];
		progressSections = [];
		editingItem = null;
		editDraft = null;
		pendingChecklistRefresh = false;
		skippedItems = [];
		showSkipDialog = false;
		runTitle = '';
		runNotes = '';
		runState.active = false;
		runState.dirty = false;
		untrack(() => {
			void loadChecklist(currentSlug);
		});
	});

	$effect(() => {
		getConfig()
			.then((config) => {
				autoScroll = config.auto_scroll;
			})
			.catch(() => {}); // non-critical: auto-scroll is a preference fallback
	});

	$effect(() => {
		const currentSlug = slug;
		let cancelled = false;
		let unlisten: (() => void) | undefined;

		listenChecklistChanges((event) => {
			if (cancelled || event.slug !== currentSlug) return;

			if (editingItem) {
				pendingChecklistRefresh = true;
				return;
			}

			void loadChecklist(currentSlug, { preserveProgress: true, silentFailure: true });
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

	$effect(() => {
		const saveForm = saveFormEl;
		if (!saveForm) return;

		const main = saveForm.closest('main');
		if (!main) return;

		const updateJumpState = () => {
			const mainRect = main.getBoundingClientRect();
			const formRect = saveForm.getBoundingClientRect();
			showJumpToSave = formRect.top > mainRect.bottom - 24;
		};

		updateJumpState();

		main.addEventListener('scroll', updateJumpState, { passive: true });
		window.addEventListener('resize', updateJumpState);

		const observer = new ResizeObserver(() => updateJumpState());
		observer.observe(saveForm);
		observer.observe(main);

		return () => {
			main.removeEventListener('scroll', updateJumpState);
			window.removeEventListener('resize', updateJumpState);
			observer.disconnect();
		};
	});

	onDestroy(() => {
		runState.active = false;
		runState.dirty = false;
	});

	function buildMergedSections(template: Section[], progress: ProgressSection[]): Section[] {
		return template.map((section, sectionIndex) => ({
			heading: section.heading,
			items: section.items.map((item, itemIndex) => ({
				...item,
				checked: progress[sectionIndex]?.items[itemIndex]?.checked ?? false,
				note: progress[sectionIndex]?.items[itemIndex]?.note,
				skipReason: progress[sectionIndex]?.items[itemIndex]?.skipReason
			}))
		}));
	}

	function cloneSections(sections: Section[]): Section[] {
		return sections.map((section) => ({
			heading: section.heading,
			items: section.items.map((item) => ({ ...item }))
		}));
	}

	function buildProgressSections(
		sections: Section[],
		previous: ProgressSection[] = []
	): ProgressSection[] {
		return sections.map((section, sectionIndex) => ({
			items: section.items.map((_, itemIndex) => ({
				checked: previous[sectionIndex]?.items[itemIndex]?.checked ?? false,
				note: previous[sectionIndex]?.items[itemIndex]?.note,
				skipReason: previous[sectionIndex]?.items[itemIndex]?.skipReason
			}))
		}));
	}

	function getRenderableSections(): Section[] {
		return buildMergedSections(templateSections, progressSections);
	}

	function getChecklistPayload(sections: Section[]): Section[] {
		return sections.map((section) => ({
			heading: section.heading,
			items: section.items.map((item) => ({
				...item,
				checked: false,
				note: undefined,
				skipReason: undefined
			}))
		}));
	}

	async function loadChecklist(
		currentSlug: string,
		options: { preserveProgress?: boolean; silentFailure?: boolean } = {}
	) {
		const requestId = ++loadVersion;
		const preserveProgress = options.preserveProgress ?? false;
		const previousProgress = preserveProgress ? progressSections : [];
		const hadRenderableContent = !!checklistMeta || templateSections.length > 0;

		try {
			const checklist = await getChecklist(currentSlug);
			if (requestId !== loadVersion) return;

			checklistMeta = checklist.meta;
			templateSections = cloneSections(checklist.sections);
			progressSections = buildProgressSections(checklist.sections, previousProgress);
			runState.title = checklist.meta.title;
			runState.active = true;
			error = '';
		} catch (e) {
			if (requestId !== loadVersion) return;

			const message = String(e);
			if (hadRenderableContent || options.silentFailure) {
				toast = { message: `Checklist reload failed: ${message}`, type: 'error' };
			} else {
				error = message;
				runState.active = false;
			}
		} finally {
			if (requestId === loadVersion) {
				loading = false;
			}
		}
	}

	function updateProgress(sectionIndex: number, itemIndex: number, checked: boolean) {
		runState.dirty = true;
		progressSections[sectionIndex].items[itemIndex].checked = checked;
		if (checked) {
			progressSections[sectionIndex].items[itemIndex].skipReason = undefined;
			scrollToNextUnchecked(sectionIndex, itemIndex);
		}
	}

	function updateItemNote(sectionIndex: number, itemIndex: number, note: string | undefined) {
		runState.dirty = true;
		progressSections[sectionIndex].items[itemIndex].note = note;
	}

	function scrollToNextUnchecked(sectionIndex: number, itemIndex: number) {
		if (!autoScroll) return;

		setTimeout(() => {
			for (let si = sectionIndex; si < progressSections.length; si += 1) {
				const startIndex = si === sectionIndex ? itemIndex + 1 : 0;
				for (let ii = startIndex; ii < progressSections[si].items.length; ii += 1) {
					if (!progressSections[si].items[ii].checked) {
						const element = document.querySelector(`[data-item="${si}-${ii}"]`);
						if (!element) return;

						const main = element.closest('main');
						if (!main) return;

						const top =
							element.getBoundingClientRect().top -
							main.getBoundingClientRect().top +
							main.scrollTop -
							50;
						main.scrollTo({ top, behavior: 'smooth' });
						return;
					}
				}
			}
		}, 150);
	}

	function handleSave() {
		if (editingItem) {
			toast = { message: 'Save or cancel the step edit first', type: 'error' };
			return;
		}

		if (!runTitle.trim()) {
			toast = { message: 'Please enter a run title', type: 'error' };
			return;
		}

		const unchecked: SkippedItem[] = [];
		getRenderableSections().forEach((section, sectionIndex) => {
			section.items.forEach((item, itemIndex) => {
				if (!item.checked) {
					unchecked.push({
						sectionIndex,
						itemIndex,
						title: item.title,
						reason: progressSections[sectionIndex]?.items[itemIndex]?.skipReason ?? ''
					});
				}
			});
		});

		if (unchecked.length > 0) {
			skippedItems = unchecked;
			showSkipDialog = true;
			return;
		}

		void doSave();
	}

	function jumpToSave() {
		const saveForm = saveFormEl;
		if (!saveForm) return;

		const main = saveForm.closest('main');
		if (!main) return;

		const top =
			saveForm.getBoundingClientRect().top - main.getBoundingClientRect().top + main.scrollTop - 12;
		main.scrollTo({ top, behavior: 'smooth' });

		if (!runTitle.trim()) {
			setTimeout(() => {
				runTitleInputEl?.focus();
			}, 250);
		}
	}

	async function doSave() {
		showSkipDialog = false;
		saving = true;

		for (const item of skippedItems) {
			progressSections[item.sectionIndex].items[item.itemIndex].skipReason =
				item.reason.trim() || undefined;
		}

		try {
			await saveRun(slug, runTitle.trim(), runNotes.trim() || undefined, getRenderableSections());
			runState.active = false;
			toast = { message: 'Run saved!', type: 'success' };
			setTimeout(() => goto('/'), 800);
		} catch (e) {
			toast = { message: String(e), type: 'error' };
			saving = false;
		}
	}

	function openInlineEditor(sectionIndex: number, itemIndex: number) {
		const item = templateSections[sectionIndex]?.items[itemIndex];
		if (!item) return;

		editingItem = { sectionIndex, itemIndex };
		editDraft = {
			title: item.title,
			description: item.description ?? '',
			code: item.code ?? '',
			codeLang: item.codeLang ?? '',
			showCode: !!item.code
		};
	}

	async function cancelInlineEditor() {
		editingItem = null;
		editDraft = null;

		if (pendingChecklistRefresh) {
			pendingChecklistRefresh = false;
			await loadChecklist(slug, { preserveProgress: true, silentFailure: true });
		}
	}

	async function saveInlineEditor() {
		if (!editingItem || !editDraft || !checklistMeta) return;

		if (!editDraft.title.trim()) {
			toast = { message: 'Step title is required', type: 'error' };
			return;
		}

		const previousSections = cloneSections(templateSections);
		const nextSections = cloneSections(templateSections);
		const item = nextSections[editingItem.sectionIndex]?.items[editingItem.itemIndex];
		if (!item) return;

		item.title = editDraft.title.trim();
		item.description = editDraft.description.trim() || undefined;
		item.code = editDraft.showCode && editDraft.code.trim() ? editDraft.code : undefined;
		item.codeLang =
			editDraft.showCode && editDraft.code.trim() && editDraft.codeLang.trim()
				? editDraft.codeLang.trim()
				: undefined;

		templateSections = nextSections;
		savingInline = true;

		try {
			await updateChecklist(
				slug,
				checklistMeta.title,
				checklistMeta.description,
				checklistMeta.tags,
				getChecklistPayload(nextSections)
			);
			await cancelInlineEditor();
		} catch (e) {
			templateSections = previousSections;
			toast = { message: String(e), type: 'error' };
		} finally {
			savingInline = false;
		}
	}

	function autoResize(el: HTMLTextAreaElement) {
		el.style.height = 'auto';
		el.style.height = `${el.scrollHeight}px`;
	}

	function removeInlineCode() {
		if (!editDraft) return;
		editDraft.showCode = false;
		editDraft.code = '';
		editDraft.codeLang = '';
	}

	function enableInlineCode() {
		if (!editDraft) return;
		editDraft.showCode = true;
	}
</script>

<div class="page">
	{#if loading}
		<div in:fade><TopLoader /></div>
	{:else if error}
		<p class="error" in:fade>{error}</p>
	{:else}
		<div in:fade={{ duration: 200 }}>
			<div class="progress-section">
				<ProgressBar checked={checkedItems} total={totalItems} />
			</div>

			<div class="sections">
				{#each mergedSections as section, sectionIndex}
					<SectionGroup heading={section.heading}>
						{#each section.items as item, itemIndex}
							<div
								data-item="{sectionIndex}-{itemIndex}"
								in:fly={{ y: 10, duration: 200, delay: (sectionIndex * 3 + itemIndex) * 30 }}
							>
								{#if editingItem?.sectionIndex === sectionIndex && editingItem?.itemIndex === itemIndex && editDraft}
									<div class="inline-editor-card">
										<label class="editor-label">
											<span>Step title</span>
											<input
												type="text"
												class="input"
												placeholder="What needs to be done?"
												bind:value={editDraft.title}
											/>
										</label>
										<label class="editor-label">
											<span>Description</span>
											<input
												type="text"
												class="input"
												placeholder="Optional details"
												bind:value={editDraft.description}
											/>
										</label>

										{#if editDraft.showCode}
											<div class="code-input-wrap">
												<label class="editor-label">
													<span>Language</span>
													<div class="code-input-header">
														<input
															type="text"
															class="input code-lang-input"
															placeholder="e.g. bash"
															bind:value={editDraft.codeLang}
														/>
														<button
															class="code-toggle-btn remove"
															type="button"
															onclick={removeInlineCode}
														>
															Remove code
														</button>
													</div>
												</label>
												<label class="editor-label">
													<span>Snippet</span>
													<textarea
														class="input code-textarea"
														placeholder="Paste code here..."
														bind:value={editDraft.code}
														rows="3"
														oninput={(event) => autoResize(event.currentTarget)}
														use:autoResize
													></textarea>
												</label>
											</div>
										{:else}
											<button class="code-toggle-btn" type="button" onclick={enableInlineCode}>
												+ Code snippet
											</button>
										{/if}

										<div class="editor-actions">
											<button
												class="secondary-btn"
												type="button"
												onclick={() => void cancelInlineEditor()}
											>
												Cancel
											</button>
											<button
												class="primary-btn"
												type="button"
												onclick={() => void saveInlineEditor()}
												disabled={savingInline}
											>
												{savingInline ? 'Saving…' : 'Save Step'}
											</button>
										</div>
									</div>
								{:else}
									<CheckboxItem
										title={item.title}
										description={item.description}
										code={item.code}
										codeLang={item.codeLang}
										note={item.note}
										skipReason={item.skipReason}
										checked={item.checked}
										editable={true}
										noteEditable={true}
										onchange={(checked) => updateProgress(sectionIndex, itemIndex, checked)}
										onnotesave={(note) => updateItemNote(sectionIndex, itemIndex, note)}
										onedit={() => openInlineEditor(sectionIndex, itemIndex)}
									/>
								{/if}
							</div>
						{/each}
					</SectionGroup>
				{/each}
			</div>

			<div class="run-form" bind:this={saveFormEl}>
				<input
					type="text"
					class="input"
					placeholder="Run title"
					bind:value={runTitle}
					bind:this={runTitleInputEl}
				/>
				<textarea
					class="input textarea"
					placeholder="Notes (optional)"
					rows="3"
					bind:value={runNotes}
				></textarea>
				<button
					class="save-btn"
					class:saving
					onclick={handleSave}
					disabled={saving || savingInline}
				>
					{#if saving}
						Saving…
					{:else}
						Save Run
					{/if}
				</button>
			</div>

			{#if showJumpToSave}
				<div class="jump-save-bar" in:fade={{ duration: 150 }}>
					<button
						class="jump-save-btn"
						type="button"
						onclick={jumpToSave}
						aria-label="Jump to save"
						title="Jump to save"
					>
						<svg
							viewBox="0 0 24 24"
							width="18"
							height="18"
							fill="none"
							stroke="currentColor"
							stroke-width="2.5"
							stroke-linecap="round"
							stroke-linejoin="round"
						>
							<path d="M6 9l6 6 6-6" />
						</svg>
					</button>
				</div>
			{/if}
		</div>
	{/if}
</div>

{#if showSkipDialog}
	<SkipReasonsDialog
		bind:items={skippedItems}
		onsave={doSave}
		oncancel={() => {
			showSkipDialog = false;
		}}
	/>
{/if}

{#if toast}
	<Toast message={toast.message} type={toast.type} onclose={() => (toast = null)} />
{/if}

<style>
	.page {
		padding-bottom: 0;
	}

	.page :global([data-item]) {
		scroll-margin-top: 50px;
	}

	.progress-section {
		position: sticky;
		top: -16px;
		z-index: 10;
		background: var(--bg-primary);
		margin: -16px -16px 20px -16px;
		padding: 16px 16px 10px 16px;
	}

	.sections {
		display: flex;
		flex-direction: column;
		gap: 16px;
		margin-bottom: 24px;
	}

	.error {
		color: var(--danger);
		font-size: 14px;
	}

	.inline-editor-card {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 12px;
		border: 1px solid var(--border);
		border-radius: 10px;
		background: var(--bg-card);
	}

	.editor-label {
		display: flex;
		flex-direction: column;
		gap: 4px;
		font-size: 11px;
		font-weight: 600;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.input {
		width: 100%;
		padding: 10px 12px;
		border: 1px solid var(--border);
		border-radius: 8px;
		background: var(--bg-card);
		color: var(--text-primary);
		font-family: inherit;
		font-size: 14px;
		outline: none;
		transition: border-color 0.15s;
	}
	.input:focus {
		border-color: var(--accent);
	}
	.input::placeholder {
		color: var(--text-secondary);
	}

	.code-input-wrap {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding-top: 4px;
		border-top: 1px solid var(--border);
	}

	.code-input-header {
		display: flex;
		gap: 6px;
		align-items: center;
	}

	.code-lang-input {
		flex: 1;
		font-size: 12px;
		padding: 4px 8px;
	}

	.code-textarea {
		font-family: 'SF Mono', 'Fira Code', 'Cascadia Code', monospace;
		font-size: 12px;
		padding: 8px 10px;
		resize: none;
		overflow: hidden;
		line-height: 1.5;
		white-space: pre;
		tab-size: 2;
	}

	.code-toggle-btn {
		padding: 4px 8px;
		border: 1px dashed var(--border);
		border-radius: 6px;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		font-family: inherit;
		font-size: 11px;
		transition: all 0.15s;
		align-self: flex-start;
	}
	.code-toggle-btn:hover {
		border-color: var(--accent);
		color: var(--accent);
	}
	.code-toggle-btn.remove {
		border-style: solid;
	}
	.code-toggle-btn.remove:hover {
		border-color: var(--danger);
		color: var(--danger);
	}

	.editor-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		padding-top: 4px;
	}

	.secondary-btn,
	.primary-btn {
		padding: 8px 12px;
		border-radius: 8px;
		font-family: inherit;
		font-size: 12px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.15s;
	}

	.secondary-btn {
		border: 1px solid var(--border);
		background: transparent;
		color: var(--text-primary);
	}
	.secondary-btn:hover {
		background: var(--bg-hover);
		border-color: var(--text-secondary);
	}

	.primary-btn {
		border: none;
		background: var(--accent);
		color: #0f0f0f;
	}
	.primary-btn:hover:not(:disabled) {
		filter: brightness(1.1);
	}
	.primary-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.run-form {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding-top: 16px;
		border-top: 1px solid var(--border);
	}

	.jump-save-bar {
		position: fixed;
		right: 16px;
		bottom: 16px;
		z-index: 20;
		pointer-events: none;
	}

	.jump-save-btn {
		pointer-events: auto;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 42px;
		height: 42px;
		border: 1px solid rgba(74, 222, 128, 0.45);
		border-radius: 999px;
		background: var(--accent);
		color: #0f0f0f;
		cursor: pointer;
		box-shadow: 0 14px 34px rgba(0, 0, 0, 0.68);
		backdrop-filter: blur(10px);
		transition: all 0.15s;
	}
	.jump-save-btn:hover {
		filter: brightness(1.08);
		transform: translateY(-1px);
	}

	.textarea {
		resize: vertical;
		min-height: 60px;
	}

	.save-btn {
		padding: 12px 16px;
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
	.save-btn:hover:not(:disabled) {
		filter: brightness(1.1);
		transform: translateY(-1px);
	}
	.save-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}
</style>
