<script lang="ts">
	import { tick } from 'svelte';

	let {
		title,
		description,
		code,
		codeLang,
		note,
		skipReason,
		checked = $bindable(),
		disabled = false,
		editable = false,
		noteEditable = false,
		onchange,
		onedit,
		onnotesave
	}: {
		title: string;
		description?: string;
		code?: string;
		codeLang?: string;
		note?: string;
		skipReason?: string;
		checked: boolean;
		disabled?: boolean;
		editable?: boolean;
		noteEditable?: boolean;
		onchange?: (checked: boolean) => void;
		onedit?: () => void;
		onnotesave?: (note: string | undefined) => void;
	} = $props();

	let copied = $state(false);
	let showNoteEditor = $state(false);
	let noteDraft = $state('');
	let noteInputEl = $state<HTMLTextAreaElement | null>(null);

	$effect(() => {
		if (!showNoteEditor) {
			noteDraft = note ?? '';
		}
	});

	function toggle() {
		if (disabled) return;
		checked = !checked;
		onchange?.(checked);
	}

	async function copyCode(e: MouseEvent) {
		e.stopPropagation();
		if (!code) return;
		await navigator.clipboard.writeText(code);
		copied = true;
		setTimeout(() => (copied = false), 1500);
	}

	async function openNoteEditor(e: MouseEvent) {
		e.stopPropagation();
		if (!noteEditable) return;

		noteDraft = note ?? '';
		showNoteEditor = true;
		await tick();
		autoResize(noteInputEl);
		noteInputEl?.focus();
		noteInputEl?.select();
	}

	function saveNote(e?: MouseEvent | KeyboardEvent) {
		e?.stopPropagation();
		if (!noteEditable) return;

		const nextNote = noteDraft.trim() || undefined;
		onnotesave?.(nextNote);
		noteDraft = nextNote ?? '';
		showNoteEditor = false;
	}

	function cancelNoteEditor(e?: MouseEvent | KeyboardEvent) {
		e?.stopPropagation();
		noteDraft = note ?? '';
		showNoteEditor = false;
	}

	function stopPropagation(e: Event) {
		e.stopPropagation();
	}

	function autoResize(el: HTMLTextAreaElement | null) {
		if (!el) return;
		el.style.height = 'auto';
		el.style.height = `${el.scrollHeight}px`;
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="item"
	class:checked
	class:disabled
	class:hasActions={editable || noteEditable}
	onclick={toggle}
	onkeydown={(e) => {
		if ((e.target as HTMLElement).closest('button, input, textarea')) return;
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			toggle();
		}
	}}
	role="button"
	tabindex="0"
>
	<div class="checkbox-wrap">
		<div class="checkbox" class:checked>
			{#if checked}
				<svg viewBox="-4 0 243.5 217" width="14" height="14" class="checkmark">
					<path
						d="M-2,123 L19,103 L29,100 L37,102 C37,102 46,110 54,119.5 C62,129 72.500,142.500 72.500,142.500 C72.500,142.500 160.783,19.528 230.500,1.500 C237,5 221,17 221,17 C221,17 161,56 82,196 C80.711,199.736 77.500,204.500 77.500,204.500 C77.500,204.500 73.851,216.472 64.500,202.500 C55.149,188.528 24,143.5 15,137 C12,135 10.5,132 8,130 C3.5,126 -2,123 -2,123 Z"
					/>
				</svg>
			{/if}
		</div>
	</div>
	<div class="content">
		<span class="title" class:checked>{title}</span>
		{#if description}
			<span class="description">{description}</span>
		{/if}
		{#if showNoteEditor}
			<div
				class="note-editor"
				onclick={stopPropagation}
				onkeydown={(e) => {
					e.stopPropagation();
					if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
						e.preventDefault();
						saveNote(e);
					} else if (e.key === 'Escape') {
						e.preventDefault();
						cancelNoteEditor(e);
					}
				}}
			>
				<textarea
					class="note-input"
					placeholder="Add note"
					bind:value={noteDraft}
					bind:this={noteInputEl}
					rows="1"
					oninput={(e) => autoResize(e.currentTarget)}
					use:autoResize
				></textarea>
				<button class="note-cancel-btn" type="button" onclick={cancelNoteEditor}> Cancel </button>
				<button class="note-save-btn" type="button" onclick={saveNote}> Save </button>
			</div>
		{:else if note}
			<div class="note-text">Note: {note}</div>
		{/if}
		{#if skipReason}
			<span class="skip-reason">Skipped: {skipReason}</span>
		{/if}
		{#if code}
			<div class="code-block">
				<div class="code-header">
					{#if codeLang}<span class="code-lang">{codeLang}</span>{/if}
					<button class="copy-btn" onclick={copyCode} type="button">
						{#if copied}
							<svg
								viewBox="0 0 24 24"
								width="14"
								height="14"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
								stroke-linecap="round"
								stroke-linejoin="round"
							>
								<polyline points="20 6 9 17 4 12" />
							</svg>
							Copied
						{:else}
							<svg
								viewBox="0 0 24 24"
								width="14"
								height="14"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
								stroke-linecap="round"
								stroke-linejoin="round"
							>
								<rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
								<path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1" />
							</svg>
							Copy
						{/if}
					</button>
				</div>
				<pre><code>{code}</code></pre>
			</div>
		{/if}
	</div>
	{#if editable || noteEditable}
		<div class="actions">
			{#if editable}
				<button
					class="action-btn"
					onclick={(e) => {
						e.stopPropagation();
						onedit?.();
					}}
					type="button"
					title="Edit step"
					aria-label="Edit step"
				>
					<svg
						viewBox="0 0 24 24"
						width="14"
						height="14"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<path d="M12 20h9" />
						<path d="M16.5 3.5a2.121 2.121 0 113 3L7 19l-4 1 1-4 12.5-12.5z" />
					</svg>
				</button>
			{/if}
			{#if noteEditable}
				<button
					class="action-btn"
					type="button"
					title={note ? 'Edit note' : 'Add note'}
					aria-label={note ? 'Edit note' : 'Add note'}
					onclick={openNoteEditor}
				>
					<svg
						viewBox="0 0 24 24"
						width="14"
						height="14"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<path
							d="M4 5.5A2.5 2.5 0 016.5 3h11A2.5 2.5 0 0120 5.5v13a.5.5 0 01-.8.4L16 16.5H6.5A2.5 2.5 0 014 14z"
						/>
						<path d="M8 8h8" />
						<path d="M8 11h5" />
					</svg>
				</button>
			{/if}
		</div>
	{/if}
</div>

<style>
	.item {
		position: relative;
		display: flex;
		align-items: flex-start;
		gap: 12px;
		width: 100%;
		padding: 10px 12px;
		border: none;
		border-radius: 8px;
		background: transparent;
		color: inherit;
		cursor: pointer;
		text-align: left;
		font-family: inherit;
		transition: background 0.15s;
	}
	.item.hasActions {
		padding-right: 40px;
	}
	.item:not(.disabled):hover {
		background: var(--bg-hover);
	}
	.item:not(.disabled):active {
		background: var(--border);
	}
	.item.disabled {
		cursor: default;
	}

	.checkbox-wrap {
		position: relative;
		flex-shrink: 0;
		padding-top: 1px;
	}

	.checkbox {
		position: relative;
		width: 22px;
		height: 22px;
		border: 2px solid var(--border);
		border-radius: 6px;
		background: transparent;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
	}
	.checkbox.checked {
		background: var(--accent);
		border-color: var(--accent);
		transform: scale(1.1);
	}

	.checkmark {
		flex-shrink: 0;
		fill: #0f0f0f;
		animation: reveal 0.15s ease-out forwards;
	}

	@keyframes reveal {
		from {
			clip-path: inset(0 100% 0 0);
		}
		to {
			clip-path: inset(0 0 0 0);
		}
	}

	.content {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
		flex: 1;
	}

	.actions {
		position: absolute;
		top: 8px;
		right: 8px;
		display: flex;
		flex-direction: column;
		gap: 4px;
		opacity: 0;
		pointer-events: none;
		transition: opacity 0.15s;
	}
	.item.hasActions:hover .actions,
	.item.hasActions:focus-visible .actions,
	.actions:focus-within {
		opacity: 1;
		pointer-events: auto;
	}

	.action-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: none;
		border-radius: 6px;
		background: rgba(255, 255, 255, 0.02);
		color: var(--text-secondary);
		cursor: pointer;
		transition:
			color 0.15s,
			background 0.15s;
	}
	.action-btn:hover {
		background: var(--bg-hover);
		color: var(--accent);
	}

	.title {
		font-size: 14px;
		font-weight: 500;
		color: var(--text-primary);
		transition: all 0.25s;
		line-height: 1.4;
	}
	.title.checked {
		color: var(--text-secondary);
		text-decoration: line-through;
		text-decoration-color: var(--text-secondary);
	}

	.description {
		font-size: 12px;
		color: var(--text-secondary);
		line-height: 1.4;
	}

	.note-text {
		font-size: 12px;
		color: var(--accent);
		line-height: 1.4;
		font-weight: 500;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}

	.note-editor {
		display: flex;
		gap: 6px;
		align-items: flex-end;
		margin-top: 4px;
	}

	.note-input {
		flex: 1;
		min-width: 0;
		padding: 6px 8px;
		border: 1px solid var(--border);
		border-radius: 7px;
		background: var(--bg-card);
		color: var(--text-primary);
		font-family: inherit;
		font-size: 12px;
		outline: none;
		resize: none;
		overflow: hidden;
		line-height: 1.4;
		white-space: pre-wrap;
	}
	.note-input:focus {
		border-color: var(--accent);
	}

	.note-save-btn {
		padding: 6px 8px;
		border: none;
		border-radius: 7px;
		background: var(--accent);
		color: #0f0f0f;
		font-family: inherit;
		font-size: 11px;
		font-weight: 700;
		cursor: pointer;
		transition: filter 0.15s;
	}
	.note-save-btn:hover {
		filter: brightness(1.08);
	}

	.note-cancel-btn {
		padding: 6px 8px;
		border: 1px solid var(--border);
		border-radius: 7px;
		background: transparent;
		color: var(--text-primary);
		font-family: inherit;
		font-size: 11px;
		font-weight: 700;
		cursor: pointer;
		transition: all 0.15s;
	}
	.note-cancel-btn:hover {
		background: var(--bg-hover);
		border-color: var(--text-secondary);
	}

	.skip-reason {
		font-size: 12px;
		color: var(--danger);
		font-style: italic;
		line-height: 1.4;
	}

	.code-block {
		margin-top: 6px;
		border: 1px solid var(--border);
		border-radius: 8px;
		overflow: hidden;
		background: var(--bg-card);
	}

	.code-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 4px 8px;
		background: var(--bg-hover);
		border-bottom: 1px solid var(--border);
	}

	.code-lang {
		font-size: 11px;
		color: var(--text-secondary);
		font-family: inherit;
		text-transform: lowercase;
	}

	.copy-btn {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		font-family: inherit;
		font-size: 11px;
		transition: all 0.15s;
		margin-left: auto;
	}
	.copy-btn:hover {
		background: var(--bg-hover);
		color: var(--accent);
	}

	pre {
		margin: 0;
		padding: 10px 12px;
		overflow-x: auto;
	}

	code {
		font-family: 'SF Mono', 'Fira Code', 'Cascadia Code', monospace;
		font-size: 12px;
		line-height: 1.5;
		color: #e0e0e0;
		white-space: pre;
	}
</style>
