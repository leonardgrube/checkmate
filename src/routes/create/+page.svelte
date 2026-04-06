<script lang="ts">
	import { goto } from '$app/navigation';
	import { fade, slide } from 'svelte/transition';
	import { flip } from 'svelte/animate';
	import {
		createChecklist,
		getConfig,
		getDefaultDataPath,
		type Section as ApiSection
	} from '$lib/api';
	import { formGuard } from '$lib/formGuard.svelte';
	import SnippetDialog from '$lib/components/SnippetDialog.svelte';
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

	let nextId = 1;
	function genId() {
		return nextId++;
	}

	let title = $state('');
	let description = $state('');
	let tagsInput = $state('');
	let sections = $state<EditSection[]>([
		{
			id: genId(),
			heading: '',
			items: [{ id: genId(), title: '', description: '', code: '', codeLang: '', showCode: false }]
		}
	]);
	let saving = $state(false);
	let showSnippetDialog = $state(false);
	let activeDataPath = $state('your checkmate data folder');
	let toast = $state<{ message: string; type: 'success' | 'error' } | null>(null);
	let aiSnippet = $derived(buildAiSnippet(activeDataPath));

	function autoResize(el: HTMLTextAreaElement) {
		el.style.height = 'auto';
		el.style.height = el.scrollHeight + 'px';
	}

	function markDirty() {
		if (title.trim() || sections.some((s) => s.items.some((i) => i.title.trim()))) {
			formGuard.dirty = true;
		}
	}

	$effect(() => {
		return () => {
			formGuard.dirty = false;
		};
	});

	$effect(() => {
		Promise.all([getConfig().catch(() => null), getDefaultDataPath().catch(() => '')])
			.then(([config, defaultPath]) => {
				activeDataPath =
					config?.data_repo_path?.trim() || defaultPath || 'your checkmate data folder';
			})
			.catch(() => {}); // non-critical: best-effort config/path lookup
	});

	function buildAiSnippet(dataPath: string) {
		return `Write a checklist for checkmate as a single markdown file.

Create the file in:
${dataPath}/checklists/

Follow these rules:
- Use UTF-8 markdown with YAML frontmatter.
- Include:
  - title (required)
  - description (optional)
  - tags (optional array of strings)
- Group checklist sections with ## headings only.
- Use unchecked checklist items with "- [ ]".
- Keep item titles concise and actionable.
- Each checklist item may have:
  - one optional single-line description
  - one optional code snippet
- If an item includes both, put the description first and the code snippet after it.
- If you include a code snippet, use a fenced code block indented 2 spaces under the checklist item.
- Only one code snippet is allowed per checklist item.
- Do not use nested lists, nested checkboxes, or heading levels other than ##.
- Do not add blank lines between an item and its description or code snippet.
- Keep all checklist items unchecked in the template.

Format example:
---
title: "Example Checklist"
description: "Short summary"
tags: [ops, example]
---

## Section Name
- [ ] Step title
  One-line description
  \`\`\`bash
  echo "example"
  \`\`\`
`;
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

			await createChecklist(title.trim(), description.trim() || undefined, tags, apiSections);
			formGuard.dirty = false;
			toast = { message: 'Checklist created!', type: 'success' };
			setTimeout(() => goto('/'), 800);
		} catch (e) {
			toast = { message: String(e), type: 'error' };
			saving = false;
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="page" in:fade={{ duration: 200 }} oninput={markDirty}>
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

	<div class="note-panel">
		<div class="note-copy">
			<span class="note-title">AI Drafting Snippet</span>
			<p class="note-text">
				Copy a prompt snippet for your AI to draft a checklist markdown file aligned with the
				required checklist markdown format.
			</p>
		</div>
		<button class="note-btn" type="button" onclick={() => (showSnippetDialog = true)}
			>Open Snippet</button
		>
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
			{saving ? 'Saving…' : 'Create Checklist'}
		</button>
	</div>
</div>

{#if toast}
	<Toast message={toast.message} type={toast.type} onclose={() => (toast = null)} />
{/if}

{#if showSnippetDialog}
	<SnippetDialog
		title="AI Prompt Snippet"
		subtitle="Copy this into your AI tool to generate a checklist markdown file for the current checkmate data path."
		snippet={aiSnippet}
		onclose={() => (showSnippetDialog = false)}
	/>
{/if}

<style>
	.page {
		padding-bottom: 0;
	}
	.form {
		display: flex;
		flex-direction: column;
		gap: 12px;
		margin-bottom: 20px;
	}
	.note-panel {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		margin-bottom: 20px;
		padding: 14px 16px;
		border: 1px solid var(--border);
		border-radius: 12px;
		background: linear-gradient(135deg, rgba(74, 222, 128, 0.08), rgba(255, 255, 255, 0.02));
	}
	.note-copy {
		min-width: 0;
	}
	.note-title {
		display: block;
		margin-bottom: 4px;
		font-size: 12px;
		font-weight: 700;
		color: var(--accent);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}
	.note-text {
		margin: 0;
		font-size: 13px;
		line-height: 1.5;
		color: var(--text-secondary);
	}
	.note-btn {
		flex-shrink: 0;
		padding: 10px 14px;
		border: none;
		border-radius: 8px;
		background: var(--accent);
		color: #0f0f0f;
		font-family: inherit;
		font-size: 13px;
		font-weight: 700;
		cursor: pointer;
		transition: all 0.15s;
	}
	.note-btn:hover {
		filter: brightness(1.08);
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

	@media (max-width: 560px) {
		.note-panel {
			flex-direction: column;
			align-items: stretch;
		}

		.note-btn {
			width: 100%;
		}
	}

	.action-bar {
		position: sticky;
		bottom: -16px;
		z-index: 10;
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
</style>
