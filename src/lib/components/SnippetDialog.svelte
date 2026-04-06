<script lang="ts">
	import { fade, scale } from 'svelte/transition';

	let {
		title,
		subtitle,
		snippet,
		onclose
	}: {
		title: string;
		subtitle: string;
		snippet: string;
		onclose: () => void;
	} = $props();

	let copied = $state(false);

	async function copySnippet() {
		await navigator.clipboard.writeText(snippet);
		copied = true;
		setTimeout(() => (copied = false), 1500);
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" transition:fade={{ duration: 150 }} onmousedown={onclose}>
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="dialog"
		transition:scale={{ duration: 200, start: 0.95 }}
		onmousedown={(e) => e.stopPropagation()}
	>
		<h3 class="title">{title}</h3>
		<p class="subtitle">{subtitle}</p>

		<div class="code-block">
			<div class="code-header">
				<span class="code-label">Prompt Snippet</span>
				<button class="copy-btn" type="button" onclick={copySnippet}>
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
			<pre><code>{snippet}</code></pre>
		</div>

		<div class="actions">
			<button class="close-btn" type="button" onclick={onclose}>Close</button>
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
		max-width: 720px;
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
		line-height: 1.5;
	}

	.code-block {
		border: 1px solid var(--border);
		border-radius: 10px;
		overflow: hidden;
		background: #141414;
	}

	.code-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		padding: 10px 12px;
		border-bottom: 1px solid var(--border);
		background: rgba(255, 255, 255, 0.02);
	}

	.code-label {
		font-size: 12px;
		font-weight: 600;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	pre {
		margin: 0;
		padding: 14px;
		overflow-x: auto;
	}

	code {
		font-family: 'SF Mono', 'Fira Code', 'Cascadia Code', monospace;
		font-size: 12px;
		line-height: 1.55;
		color: var(--text-primary);
		white-space: pre-wrap;
		word-break: break-word;
	}

	.copy-btn,
	.close-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 8px 12px;
		border-radius: 8px;
		font-family: inherit;
		font-size: 12px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.15s;
	}

	.copy-btn {
		border: 1px solid var(--border);
		background: transparent;
		color: var(--text-primary);
	}

	.copy-btn:hover {
		background: var(--bg-hover);
	}

	.actions {
		display: flex;
		justify-content: flex-end;
		margin-top: 16px;
	}

	.close-btn {
		border: none;
		background: var(--accent);
		color: #0f0f0f;
	}

	.close-btn:hover {
		filter: brightness(1.1);
	}
</style>
