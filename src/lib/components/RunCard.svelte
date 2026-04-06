<script lang="ts">
	let {
		title,
		templateLabel,
		date,
		checked,
		total,
		onclick
	}: {
		title: string;
		templateLabel: string;
		date: string;
		checked: number;
		total: number;
		onclick: () => void;
	} = $props();

	let allDone = $derived(checked === total);
</script>

<button class="card" {onclick} type="button">
	<div class="top">
		<span class="title">{title}</span>
		<span class="date">{date}</span>
	</div>
	<span class="template">{templateLabel}</span>
	<div class="progress">
		<div class="progress-bar">
			<div
				class="progress-fill"
				class:done={allDone}
				style="width: {total > 0 ? (checked / total) * 100 : 0}%"
			></div>
		</div>
		<span class="progress-text" class:done={allDone}>{checked}/{total}</span>
	</div>
</button>

<style>
	.card {
		display: flex;
		flex-direction: column;
		gap: 6px;
		width: 100%;
		padding: 14px 16px;
		border: 1px solid var(--border);
		border-radius: 10px;
		background: var(--bg-card);
		color: inherit;
		cursor: pointer;
		text-align: left;
		font-family: inherit;
		transition: all 0.2s;
	}
	.card:hover {
		background: var(--bg-hover);
		border-color: var(--border);
	}
	.card:active {
		background: var(--border);
	}

	.top {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		flex-wrap: wrap;
		gap: 8px;
	}

	.title {
		flex: 1;
		min-width: 0;
		font-size: 15px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.date {
		font-size: 12px;
		color: var(--text-secondary);
		flex-shrink: 0;
		font-variant-numeric: tabular-nums;
		white-space: nowrap;
	}

	.template {
		font-size: 12px;
		color: var(--text-secondary);
	}

	.progress {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-top: 2px;
	}

	.progress-bar {
		flex: 1;
		height: 4px;
		background: var(--border);
		border-radius: 2px;
		overflow: hidden;
	}

	.progress-fill {
		height: 100%;
		background: var(--text-secondary);
		border-radius: 2px;
		transition: width 0.3s;
	}
	.progress-fill.done {
		background: var(--accent);
	}

	.progress-text {
		font-size: 12px;
		color: var(--text-secondary);
		flex-shrink: 0;
		font-variant-numeric: tabular-nums;
	}
	.progress-text.done {
		color: var(--accent);
	}
</style>
