<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { fade, slide } from 'svelte/transition';
	import { flip } from 'svelte/animate';
	import {
		getChecklist,
		updateChecklist,
		deleteChecklist,
		type Section as ApiSection
	} from '$lib/api';
	import { formGuard } from '$lib/formGuard.svelte';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
	import TopLoader from '$lib/components/TopLoader.svelte';
	import Toast from '$lib/components/Toast.svelte';

	interface EditItem {
		id: number;
		title: string;
		description: string;
		code: string;
		codeLang: string;
		showCode: boolean;
	}

	interface EditSection {
		id: number;
		heading: string;
		items: EditItem[];
	}

	let slug = $derived($page.params.slug as string);

	let nextId = 1;
	function genId() {
		return nextId++;
	}

	let title = $state('');
	let description = $state('');
	let tagsInput = $state('');
	let sections = $state<EditSection[]>([]);
	let loading = $state(true);
	let saving = $state(false);
	let error = $state('');
	let toast = $state<{ message: string; type: 'success' | 'error' } | null>(null);
	let showDeleteConfirm = $state(false);

	function autoResize(el: HTMLTextAreaElement) {
		el.style.height = 'auto';
		el.style.height = el.scrollHeight + 'px';
	}

	function markDirty() {
		formGuard.dirty = true;
	}

	$effect(() => {
		return () => {
			formGuard.dirty = false;
		};
	});

	$effect(() => {
		loadChecklist(slug);
	});

	async function loadChecklist(s: string) {
		try {
			const cl = await getChecklist(s);
			title = cl.meta.title;
			description = cl.meta.description ?? '';
			tagsInput = cl.meta.tags.join(', ');
			sections = cl.sections.map((sec) => ({
				id: genId(),
				heading: sec.heading,
				items: sec.items.map((item) => ({
					id: genId(),
					title: item.title,
					description: item.description ?? '',
					code: item.code ?? '',
					codeLang: item.codeLang ?? '',
					showCode: !!item.code
				}))
			}));
			loading = false;
		} catch (e) {
			error = String(e);
			loading = false;
		}
	}

	function addSection() {
		sections.push({
			id: genId(),
			heading: '',
			items: [{ id: genId(), title: '', description: '', code: '', codeLang: '', showCode: false }]
		});
	}

	function removeSection(si: number) {
		if (sections.length <= 1) return;
		sections.splice(si, 1);
	}

	function moveSection(si: number, dir: -1 | 1) {
		const ni = si + dir;
		if (ni < 0 || ni >= sections.length) return;
		[sections[si], sections[ni]] = [sections[ni], sections[si]];
	}

	function addItem(si: number) {
		sections[si].items.push({
			id: genId(),
			title: '',
			description: '',
			code: '',
			codeLang: '',
			showCode: false
		});
	}

	function removeItem(si: number, ii: number) {
		if (sections[si].items.length <= 1) return;
		sections[si].items.splice(ii, 1);
	}

	function moveItem(si: number, ii: number, dir: -1 | 1) {
		const ni = ii + dir;
		const items = sections[si].items;
		if (ni < 0 || ni >= items.length) return;
		[items[ii], items[ni]] = [items[ni], items[ii]];
	}

	async function handleSave() {
		if (!title.trim()) {
			toast = { message: 'Title is required', type: 'error' };
			return;
		}

		const hasItems = sections.some((s) => s.items.some((i) => i.title.trim()));
		if (!hasItems) {
			toast = { message: 'Add at least one item', type: 'error' };
			return;
		}

		saving = true;
		try {
			const tags = tagsInput
				.split(',')
				.map((t) => t.trim())
				.filter(Boolean);

			const apiSections: ApiSection[] = sections
				.filter((s) => s.items.some((i) => i.title.trim()))
				.map((s) => ({
					heading: s.heading.trim(),
					items: s.items
						.filter((i) => i.title.trim())
						.map((i) => ({
							title: i.title.trim(),
							description: i.description.trim() || undefined,
							code: i.code.trim() || undefined,
							codeLang: i.codeLang.trim() || undefined,
							checked: false
						}))
				}));

			await updateChecklist(slug, title.trim(), description.trim() || undefined, tags, apiSections);
			formGuard.dirty = false;
			toast = { message: 'Checklist updated!', type: 'success' };
			setTimeout(() => goto('/'), 800);
		} catch (e) {
			toast = { message: String(e), type: 'error' };
			saving = false;
		}
	}

	async function handleDelete() {
		showDeleteConfirm = false;
		try {
			await deleteChecklist(slug);
			formGuard.dirty = false;
			goto('/');
		} catch (e) {
			toast = { message: String(e), type: 'error' };
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="page" in:fade={{ duration: 200 }} oninput={markDirty}>
	{#if loading}
		<TopLoader />
	{:else if error}
		<p class="error">{error}</p>
	{:else}
		<div class="form">
			<label class="field">
				<span class="field-label">Title</span>
				<input type="text" class="input" placeholder="Checklist title" bind:value={title} />
			</label>
			<label class="field">
				<span class="field-label">Description</span>
				<input type="text" class="input" placeholder="Optional" bind:value={description} />
			</label>
			<label class="field">
				<span class="field-label">Tags</span>
				<input type="text" class="input" placeholder="Comma-separated" bind:value={tagsInput} />
			</label>
		</div>

		<div class="sections">
			{#each sections as section, si (section.id)}
				<div
					class="section-card"
					animate:flip={{ duration: 250 }}
					transition:slide={{ duration: 200 }}
				>
					<div class="section-header">
						<label class="section-label">
							<span class="field-label">Section heading</span>
							<input
								type="text"
								class="input section-input"
								placeholder="e.g. Prerequisites"
								bind:value={section.heading}
							/>
						</label>
						<div class="section-actions">
							<button
								class="icon-btn"
								type="button"
								onclick={() => moveSection(si, -1)}
								disabled={si === 0}
								title="Move up">↑</button
							>
							<button
								class="icon-btn"
								type="button"
								onclick={() => moveSection(si, 1)}
								disabled={si === sections.length - 1}
								title="Move down">↓</button
							>
							<button
								class="icon-btn danger"
								type="button"
								onclick={() => removeSection(si)}
								disabled={sections.length <= 1}
								title="Remove section">×</button
							>
						</div>
					</div>

					<div class="items">
						{#each section.items as item, ii (item.id)}
							<div class="item-card" animate:flip={{ duration: 200 }}>
								<div class="item-header">
									<span class="item-number">{ii + 1}</span>
									<div class="item-actions">
										<button
											class="icon-btn small"
											type="button"
											onclick={() => moveItem(si, ii, -1)}
											disabled={ii === 0}>↑</button
										>
										<button
											class="icon-btn small"
											type="button"
											onclick={() => moveItem(si, ii, 1)}
											disabled={ii === section.items.length - 1}>↓</button
										>
										<button
											class="icon-btn small danger"
											type="button"
											onclick={() => removeItem(si, ii)}
											disabled={section.items.length <= 1}>×</button
										>
									</div>
								</div>
								<div class="item-inputs">
									<label class="item-label"
										>Step title
										<input
											type="text"
											class="input item-input"
											placeholder="What needs to be done?"
											bind:value={item.title}
										/>
									</label>
									<label class="item-label"
										>Description
										<input
											type="text"
											class="input item-desc-input"
											placeholder="Optional details"
											bind:value={item.description}
										/>
									</label>
									{#if item.showCode}
										<div class="code-input-wrap">
											<label class="item-label"
												>Language
												<div class="code-input-header">
													<input
														type="text"
														class="input code-lang-input"
														placeholder="e.g. bash"
														bind:value={item.codeLang}
													/>
													<button
														class="code-toggle-btn remove"
														type="button"
														onclick={() => {
															item.showCode = false;
															item.code = '';
															item.codeLang = '';
														}}>Remove code</button
													>
												</div>
											</label>
											<label class="item-label"
												>Snippet
												<textarea
													class="input code-textarea"
													placeholder="Paste code here..."
													bind:value={item.code}
													rows="3"
													oninput={(e) => autoResize(e.currentTarget)}
													use:autoResize
												></textarea>
											</label>
										</div>
									{:else}
										<button
											class="code-toggle-btn"
											type="button"
											onclick={() => {
												item.showCode = true;
											}}>+ Code snippet</button
										>
									{/if}
								</div>
							</div>
						{/each}
						<button class="add-btn" type="button" onclick={() => addItem(si)}>+ Add item</button>
					</div>
				</div>
			{/each}
		</div>

		<button class="add-btn section-add" type="button" onclick={addSection}>+ Add section</button>

		<div class="action-bar">
			<button class="save-btn" onclick={handleSave} disabled={saving}>
				{saving ? 'Saving…' : 'Save Changes'}
			</button>

			<button class="delete-btn" onclick={() => (showDeleteConfirm = true)} type="button">
				Delete Checklist
			</button>
		</div>
	{/if}
</div>

{#if showDeleteConfirm}
	<ConfirmDialog
		message="Are you sure you want to delete this checklist? This cannot be undone."
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
		padding-bottom: 0;
	}
	.error {
		color: var(--danger);
		font-size: 14px;
	}
	.form {
		display: flex;
		flex-direction: column;
		gap: 12px;
		margin-bottom: 20px;
	}
	.field {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.field-label {
		font-size: 12px;
		font-weight: 600;
		color: var(--accent);
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

	.sections {
		display: flex;
		flex-direction: column;
		gap: 12px;
		margin-bottom: 12px;
	}

	.section-card {
		border: 1px solid var(--border);
		border-radius: 10px;
		background: var(--bg-card);
		padding: 12px;
	}
	.section-header {
		display: flex;
		gap: 8px;
		margin-bottom: 10px;
		align-items: flex-end;
	}
	.section-label {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.section-input {
		font-weight: 600;
	}
	.section-actions {
		display: flex;
		gap: 2px;
		flex-shrink: 0;
		padding-bottom: 2px;
	}

	.items {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	.item-card {
		border-left: 2px solid var(--accent-dim);
		padding: 10px 12px;
		border-radius: 0 8px 8px 0;
		background: rgba(255, 255, 255, 0.02);
	}
	.item-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 8px;
	}
	.item-number {
		font-size: 11px;
		font-weight: 700;
		color: var(--accent);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}
	.item-inputs {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.item-label {
		font-size: 11px;
		font-weight: 600;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.04em;
		margin-top: 4px;
	}
	.item-label:first-child {
		margin-top: 0;
	}
	.item-input {
		font-size: 13px;
		padding: 8px 10px;
	}
	.item-desc-input {
		font-size: 12px;
		padding: 6px 10px;
	}

	.code-input-wrap {
		display: flex;
		flex-direction: column;
		gap: 4px;
		margin-top: 8px;
		padding-top: 8px;
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

	.item-actions {
		display: flex;
		gap: 2px;
		flex-shrink: 0;
	}

	.icon-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: none;
		border-radius: 6px;
		background: transparent;
		color: var(--text-primary);
		cursor: pointer;
		font-size: 14px;
		font-family: inherit;
		transition: all 0.15s;
	}
	.icon-btn:hover:not(:disabled) {
		background: var(--bg-hover);
		color: #fff;
	}
	.icon-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
	.icon-btn.danger:hover:not(:disabled) {
		color: var(--danger);
	}
	.icon-btn.small {
		width: 24px;
		height: 24px;
		font-size: 12px;
	}

	.add-btn {
		padding: 8px 12px;
		border: 1px dashed var(--border);
		border-radius: 8px;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		font-family: inherit;
		font-size: 13px;
		transition: all 0.15s;
		width: 100%;
		text-align: center;
	}
	.add-btn:hover {
		border-color: var(--accent);
		color: var(--accent);
	}
	.section-add {
		margin-bottom: 16px;
	}

	.action-bar {
		position: sticky;
		bottom: -16px;
		z-index: 10;
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin: 0 -16px -16px -16px;
		padding: 14px 16px 18px;
		background: var(--bg-primary);
	}
	.save-btn {
		width: 100%;
		padding: 14px;
		border: none;
		border-radius: 10px;
		background: var(--accent);
		color: #0f0f0f;
		font-family: inherit;
		font-size: 15px;
		font-weight: 700;
		cursor: pointer;
		transition: all 0.15s;
	}
	.save-btn:hover:not(:disabled) {
		filter: brightness(1.1);
		transform: translateY(-1px);
	}
	.save-btn:active:not(:disabled) {
		transform: translateY(0);
	}
	.save-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
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
	}
	.delete-btn:hover {
		background: var(--danger);
		color: #fff;
	}
</style>
