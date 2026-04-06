<script lang="ts">
	import { fly } from 'svelte/transition';

	let {
		message,
		type = 'success',
		onclose
	}: {
		message: string;
		type?: 'success' | 'error';
		onclose: () => void;
	} = $props();

	$effect(() => {
		const timer = setTimeout(onclose, 3000);
		return () => clearTimeout(timer);
	});
</script>

<div class="toast {type}" transition:fly={{ y: 40, duration: 300 }} role="alert">
	<span class="icon">{type === 'success' ? '✓' : '!'}</span>
	<span class="msg">{message}</span>
</div>

<style>
	.toast {
		position: fixed;
		bottom: 20px;
		left: 50%;
		transform: translateX(-50%);
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 16px;
		border-radius: 8px;
		font-size: 13px;
		font-weight: 500;
		z-index: 1000;
		box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
	}
	.toast.success {
		background: #1a2e1a;
		color: var(--accent);
		border: 1px solid var(--accent-dim);
	}
	.toast.error {
		background: #2e1a1a;
		color: var(--danger);
		border: 1px solid rgba(248, 113, 113, 0.2);
	}
	.icon {
		font-weight: 700;
		font-size: 14px;
	}
</style>
