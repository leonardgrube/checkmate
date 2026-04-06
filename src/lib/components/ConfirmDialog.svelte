<script lang="ts">
	import { fade, scale } from 'svelte/transition';

	let {
		message,
		confirmLabel = 'Cancel Run',
		cancelLabel = 'Continue',
		onconfirm,
		oncancel
	}: {
		message: string;
		confirmLabel?: string;
		cancelLabel?: string;
		onconfirm: () => void;
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
		<p class="message">{message}</p>
		<div class="actions">
			<button class="btn confirm" onclick={onconfirm}>{confirmLabel}</button>
			<button class="btn cancel" onclick={oncancel}>{cancelLabel}</button>
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
		max-width: 320px;
		width: 100%;
	}
	.message {
		margin: 0 0 16px 0;
		font-size: 14px;
		color: var(--text-primary);
		line-height: 1.5;
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
	.cancel {
		background: var(--accent);
		color: #0f0f0f;
		font-weight: 700;
	}
	.cancel:hover {
		filter: brightness(1.1);
	}
	.confirm {
		background: var(--danger);
		color: #fff;
	}
	.confirm:hover {
		filter: brightness(1.1);
	}
</style>
