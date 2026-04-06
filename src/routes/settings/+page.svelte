<script lang="ts">
	import { fade } from 'svelte/transition';
	import { open } from '@tauri-apps/plugin-dialog';
	import {
		getConfig,
		updateConfig,
		setWindowSize,
		getDefaultDataPath,
		checkIsGitRepo,
		type SettingsConfigUpdate,
		type WindowPresets
	} from '$lib/api';
	import TopLoader from '$lib/components/TopLoader.svelte';
	import Toast from '$lib/components/Toast.svelte';

	let path = $state('');
	let autoCommit = $state(true);
	let alwaysOnTop = $state(true);
	let autoScroll = $state(false);
	let presets = $state<WindowPresets>({
		compact: [420, 900],
		sidebar_width: 420,
		sidebar_dock: 'right',
		large: [720, 1100]
	});
	let defaultPath = $state('');
	let isGitRepo = $state<boolean | null>(null);
	let loading = $state(true);
	let saving = $state(false);
	let toast = $state<{ message: string; type: 'success' | 'error' } | null>(null);

	$effect(() => {
		Promise.all([getConfig(), getDefaultDataPath().catch(() => '')])
			.then(([config, defPath]) => {
				path = config.data_repo_path ?? '';
				autoCommit = config.auto_commit;
				alwaysOnTop = config.always_on_top;
				autoScroll = config.auto_scroll;
				presets = config.window_presets;
				defaultPath = defPath;
				loading = false;
			})
			.catch(() => {
				loading = false;
			});
	});

	$effect(() => {
		const checkPath = path.trim() || defaultPath;
		if (!checkPath) return;
		checkIsGitRepo(checkPath)
			.then((result) => {
				isGitRepo = result;
				if (!result) autoCommit = false;
			})
			.catch(() => {
				isGitRepo = false;
				autoCommit = false;
			});
	});

	async function handleSave() {
		saving = true;
		try {
			const config: SettingsConfigUpdate = {
				data_repo_path: path.trim() || undefined,
				auto_commit: autoCommit,
				always_on_top: alwaysOnTop,
				auto_scroll: autoScroll,
				window_presets: presets
			};
			await updateConfig(config);
			const savedConfig = await getConfig();
			if (savedConfig.window_size) {
				await setWindowSize(savedConfig.window_size);
			}
			toast = { message: 'Settings saved!', type: 'success' };
		} catch (e) {
			toast = { message: String(e), type: 'error' };
		}
		saving = false;
	}

	async function handleBrowse() {
		const selected = await open({ directory: true, title: 'Select data repository' });
		if (selected) path = selected;
	}
</script>

<div class="page" in:fade={{ duration: 200 }}>
	{#if loading}
		<TopLoader />
	{:else}
		<div class="form">
			<label class="label">
				Data repository path
				<div class="path-row">
					<input type="text" class="input" placeholder="Select data folder..." bind:value={path} />
					<button type="button" class="browse-btn" onclick={() => void handleBrowse()}>
						Browse
					</button>
				</div>
				<span class="hint">
					Directory containing checklists/ and runs/ folders.
					{#if defaultPath}
						Default: {defaultPath}
					{:else}
						Leave empty to use the default location.
					{/if}
				</span>
				{#if isGitRepo === false}
					<span class="warning">
						This folder is not a git repository. Auto-commit has been disabled.
					</span>
				{/if}
			</label>

			<div class="toggle-group">
				<div class="toggle-row">
					<div class="toggle-info">
						<span class="toggle-title">Auto-commit</span>
						<span class="hint">Automatically git-commit after saving checklists and runs</span>
					</div>
					<button
						class="toggle"
						class:on={autoCommit}
						onclick={() => (autoCommit = !autoCommit)}
						type="button"
						aria-label="Toggle auto-commit"
						disabled={isGitRepo === false}
					>
						<span class="toggle-knob"></span>
					</button>
				</div>

				<div class="toggle-row">
					<div class="toggle-info">
						<span class="toggle-title">Auto-scroll on completion</span>
						<span class="hint">Scroll to the next unchecked item when completing a step</span>
					</div>
					<button
						class="toggle"
						class:on={autoScroll}
						onclick={() => (autoScroll = !autoScroll)}
						type="button"
						aria-label="Toggle auto-scroll on completion"
					>
						<span class="toggle-knob"></span>
					</button>
				</div>
			</div>

			<div class="presets-section">
				<span class="section-title">Window Presets</span>
				<span class="hint">Customize pixel sizes for each window preset</span>

				<div class="preset-row">
					<span class="preset-name">Compact</span>
					<div class="preset-inputs">
						<input
							type="number"
							class="num-input"
							bind:value={presets.compact[0]}
							min="200"
							max="3000"
						/>
						<span class="dim-sep">&times;</span>
						<input
							type="number"
							class="num-input"
							bind:value={presets.compact[1]}
							min="200"
							max="3000"
						/>
					</div>
				</div>

				<div class="preset-row">
					<span class="preset-name">Large</span>
					<div class="preset-inputs">
						<input
							type="number"
							class="num-input"
							bind:value={presets.large[0]}
							min="200"
							max="3000"
						/>
						<span class="dim-sep">&times;</span>
						<input
							type="number"
							class="num-input"
							bind:value={presets.large[1]}
							min="200"
							max="3000"
						/>
					</div>
				</div>

				<div class="preset-row">
					<span class="preset-name">Sidebar</span>
					<div class="preset-inputs">
						<div class="dock-toggle">
							<button
								class="dock-btn"
								class:active={presets.sidebar_dock === 'left'}
								onclick={() => (presets.sidebar_dock = 'left')}
								type="button">Left</button
							>
							<button
								class="dock-btn"
								class:active={presets.sidebar_dock === 'right'}
								onclick={() => (presets.sidebar_dock = 'right')}
								type="button">Right</button
							>
						</div>
						<input
							type="number"
							class="num-input"
							bind:value={presets.sidebar_width}
							min="200"
							max="3000"
						/>
						<span class="dim-sep">&times;</span>
						<input type="text" class="num-input disabled" value="100%" disabled />
					</div>
				</div>
			</div>

			<button class="save-btn" onclick={handleSave} disabled={saving}>
				{saving ? 'Saving…' : 'Save Settings'}
			</button>
		</div>
	{/if}
</div>

{#if toast}
	<Toast message={toast.message} type={toast.type} onclose={() => (toast = null)} />
{/if}

<style>
	.page {
		padding-bottom: 20px;
	}
	.form {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	.label {
		display: flex;
		flex-direction: column;
		gap: 6px;
		font-size: 13px;
		font-weight: 600;
		color: var(--text-secondary);
	}
	.path-row {
		display: flex;
		gap: 6px;
	}
	.path-row .input {
		flex: 1;
		min-width: 0;
	}
	.browse-btn {
		padding: 10px 14px;
		border: 1px solid var(--border);
		border-radius: 8px;
		background: var(--bg-card);
		color: var(--text-primary);
		font-family: inherit;
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
		white-space: nowrap;
		transition:
			border-color 0.15s,
			background 0.15s;
	}
	.browse-btn:hover {
		border-color: var(--accent);
		background: var(--bg-hover, var(--bg-card));
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
	.hint {
		font-size: 12px;
		font-weight: 400;
		color: var(--text-secondary);
		line-height: 1.4;
	}
	.warning {
		font-size: 12px;
		font-weight: 500;
		color: #e8a735;
		line-height: 1.4;
	}

	.toggle-group {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 12px 0;
		border-top: 1px solid var(--border);
		border-bottom: 1px solid var(--border);
	}

	.toggle-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}

	.toggle-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
		flex: 1;
	}

	.toggle-title {
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.toggle {
		position: relative;
		width: 44px;
		height: 24px;
		border-radius: 12px;
		border: none;
		background: var(--border);
		cursor: pointer;
		transition: background 0.2s;
		flex-shrink: 0;
		padding: 0;
	}
	.toggle.on {
		background: var(--accent);
	}
	.toggle:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.toggle-knob {
		position: absolute;
		top: 2px;
		left: 2px;
		width: 20px;
		height: 20px;
		border-radius: 50%;
		background: #fff;
		transition: transform 0.2s;
	}
	.toggle.on .toggle-knob {
		transform: translateX(20px);
	}

	.presets-section {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 12px 0;
		border-top: 1px solid var(--border);
		border-bottom: 1px solid var(--border);
	}

	.section-title {
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.preset-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}

	.preset-name {
		font-size: 13px;
		font-weight: 500;
		color: var(--text-secondary);
		min-width: 60px;
	}

	.preset-inputs {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.num-input {
		width: 64px;
		padding: 6px 8px;
		border: 1px solid var(--border);
		border-radius: 6px;
		background: var(--bg-card);
		color: var(--text-primary);
		font-family: inherit;
		font-size: 13px;
		text-align: center;
		outline: none;
		transition: border-color 0.15s;
		appearance: textfield;
		-moz-appearance: textfield;
	}
	.num-input::-webkit-inner-spin-button,
	.num-input::-webkit-outer-spin-button {
		-webkit-appearance: none;
		margin: 0;
	}
	.num-input:focus {
		border-color: var(--accent);
	}
	.num-input.disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.dim-sep {
		font-size: 13px;
		color: var(--text-secondary);
	}

	.dock-toggle {
		display: flex;
		margin-right: 6px;
		border: 1px solid var(--border);
		border-radius: 6px;
		overflow: hidden;
	}

	.dock-btn {
		padding: 6px 10px 5.5px;
		border: none;
		background: transparent;
		color: var(--text-secondary);
		font-family: inherit;
		font-size: 13px;
		cursor: pointer;
		transition: all 0.15s;
	}
	.dock-btn.active {
		background: var(--accent);
		color: #0f0f0f;
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
