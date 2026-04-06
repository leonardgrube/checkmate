<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { runState } from '$lib/runState.svelte';
	import { formGuard } from '$lib/formGuard.svelte';
	import {
		setWindowSize,
		getConfig,
		updateConfig,
		type AppConfig,
		type WindowPresets
	} from '$lib/api';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';

	let { children } = $props();
	let showSizeMenu = $state(false);
	let alwaysOnTop = $state(true);
	let presets = $state<WindowPresets>({
		compact: [420, 900],
		sidebar_width: 420,
		sidebar_dock: 'right',
		large: [720, 1100]
	});

	$effect(() => {
		void refreshWindowPreferences();
	});

	let path = $derived($page.url.pathname as string);
	let inRun = $derived(path.startsWith('/run/'));
	let inCreate = $derived(path === '/create');
	let inEdit = $derived(path.startsWith('/edit/'));
	let inSettings = $derived(path === '/settings');
	let inRuns = $derived(path === '/runs');
	let inRunView = $derived(path.startsWith('/runs/') && path !== '/runs');
	let inSubpage = $derived(inRun || inCreate || inEdit || inSettings || inRuns || inRunView);

	let pendingNav = $state<string | null>(null);

	let confirmMessage = $derived(
		runState.active && runState.dirty && inRun
			? 'You have an active run in progress. Cancel this run and leave?'
			: 'You have unsaved changes. Discard changes and leave?'
	);

	function navTo(href: string) {
		if (runState.active && runState.dirty && inRun) {
			pendingNav = href;
		} else if (formGuard.dirty && (inCreate || inEdit)) {
			pendingNav = href;
		} else {
			goto(href);
		}
	}

	function confirmNav() {
		const dest = pendingNav;
		pendingNav = null;
		runState.active = false;
		runState.dirty = false;
		formGuard.dirty = false;
		if (dest) goto(dest);
	}

	function cancelNav() {
		pendingNav = null;
	}

	function toggleSizeMenu() {
		showSizeMenu = !showSizeMenu;
		if (showSizeMenu) {
			void refreshWindowPreferences();
		}
	}

	function pickSize(size: string) {
		showSizeMenu = false;
		setWindowSize(size);
	}

	function handleClickOutside(e: MouseEvent) {
		if (showSizeMenu && !(e.target as HTMLElement).closest('.size-menu-wrap')) {
			showSizeMenu = false;
		}
	}

	async function refreshWindowPreferences() {
		try {
			const config = await getConfig();
			presets = config.window_presets;
			alwaysOnTop = config.always_on_top;
		} catch {
			// Keep the current UI state if config lookup fails.
		}
	}

	async function toggleAlwaysOnTop() {
		const next = !alwaysOnTop;
		const window = getCurrentWindow();
		alwaysOnTop = next;

		try {
			await window.setAlwaysOnTop(next);
			const config = await getConfig();
			const updated: AppConfig = { ...config, always_on_top: next };
			await updateConfig(updated);
		} catch {
			alwaysOnTop = !next;
			try {
				await window.setAlwaysOnTop(!next);
			} catch {
				// Ignore rollback failures.
			}
		}
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="app" onclick={handleClickOutside}>
	<header>
		<div class="header-side left">
			{#if inSubpage}
				<button class="icon-btn" onclick={() => navTo('/')} title="Back to list">
					<svg
						viewBox="0 0 24 24"
						width="18"
						height="18"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<path d="M15 18l-6-6 6-6" />
					</svg>
				</button>
			{/if}
		</div>

		{#if inRun && runState.title}
			<span class="header-title">{runState.title}</span>
		{:else if inCreate}
			<span class="header-title">Create New Checklist</span>
		{:else if inEdit}
			<span class="header-title">Edit Checklist</span>
		{:else if inSettings}
			<span class="header-title">Settings</span>
		{:else if inRuns || inRunView}
			<span class="header-title">Run History</span>
		{/if}

		<div class="header-side right">
			<button
				class="icon-btn"
				class:active={path === '/create'}
				onclick={() => navTo('/create')}
				title="Create checklist"
			>
				<svg
					viewBox="0 0 24 24"
					width="18"
					height="18"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
				>
					<path d="M12 5v14M5 12h14" />
				</svg>
			</button>
			<button
				class="icon-btn"
				class:active={inRuns || inRunView}
				onclick={() => navTo('/runs')}
				title="Run history"
			>
				<svg
					viewBox="0 0 24 24"
					width="18"
					height="18"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<circle cx="12" cy="12" r="10" />
					<polyline points="12 6 12 12 16 14" />
				</svg>
			</button>
			<div class="size-menu-wrap">
				<button
					class="icon-btn"
					class:active={showSizeMenu}
					onclick={toggleSizeMenu}
					title="Window size"
				>
					<svg
						viewBox="0 0 24 24"
						width="18"
						height="18"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<rect x="2" y="3" width="20" height="14" rx="2" ry="2" />
						<line x1="8" y1="21" x2="16" y2="21" />
						<line x1="12" y1="17" x2="12" y2="21" />
					</svg>
				</button>
				{#if showSizeMenu}
					<div class="size-dropdown">
						<button class="size-option" onclick={() => pickSize('compact')}>
							<span class="size-label">Compact</span>
							<span class="size-desc">{presets.compact[0]} × {presets.compact[1]}</span>
						</button>
						<button class="size-option" onclick={() => pickSize('large')}>
							<span class="size-label">Large</span>
							<span class="size-desc">{presets.large[0]} × {presets.large[1]}</span>
						</button>
						<button class="size-option" onclick={() => pickSize('sidebar')}>
							<span class="size-label">Sidebar</span>
							<span class="size-desc"
								>{presets.sidebar_width} × full, docked {presets.sidebar_dock}</span
							>
						</button>
						<div class="size-divider"></div>
						<div class="menu-toggle-row">
							<div class="menu-toggle-copy">
								<span class="size-label">Stay in foreground</span>
								<span class="size-desc">Keep the app above other windows</span>
							</div>
							<button
								class="toggle"
								class:on={alwaysOnTop}
								onclick={(event) => {
									event.stopPropagation();
									void toggleAlwaysOnTop();
								}}
								type="button"
								aria-label="Toggle stay in foreground"
							>
								<span class="toggle-knob"></span>
							</button>
						</div>
					</div>
				{/if}
			</div>
			<button
				class="icon-btn"
				class:active={path === '/settings'}
				onclick={() => navTo('/settings')}
				title="Settings"
			>
				<svg
					viewBox="0 0 24 24"
					width="18"
					height="18"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<circle cx="12" cy="12" r="3" />
					<path
						d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 01-2.83 2.83l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"
					/>
				</svg>
			</button>
		</div>
	</header>
	<main class:no-scroll={pendingNav !== null}>
		{@render children()}
	</main>
</div>

{#if pendingNav !== null}
	<ConfirmDialog
		message={confirmMessage}
		confirmLabel="Discard"
		onconfirm={confirmNav}
		oncancel={cancelNav}
	/>
{/if}

<style>
	:global(*) {
		box-sizing: border-box;
		scrollbar-color: #3a3a3a transparent;
		scrollbar-width: thin;
	}
	:global(*::-webkit-scrollbar) {
		width: 8px;
	}
	:global(*::-webkit-scrollbar-track) {
		background: transparent;
	}
	:global(*::-webkit-scrollbar-thumb) {
		background: #3a3a3a;
		border-radius: 4px;
	}
	:global(*::-webkit-scrollbar-thumb:hover) {
		background: #555;
	}
	:global(body) {
		margin: 0;
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
		background: var(--bg-primary);
		color: var(--text-primary);
		--bg-primary: #0f0f0f;
		--bg-card: #1a1a1a;
		--bg-hover: #2a2a2a;
		--border: #3a3a3a;
		--text-primary: #ececec;
		--text-secondary: #999;
		--accent: #4ade80;
		--accent-dim: rgba(74, 222, 128, 0.2);
		--danger: #b91c1c;
	}

	.app {
		display: flex;
		flex-direction: column;
		height: 100vh;
	}

	header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 10px 16px;
		border-bottom: 1px solid var(--border);
		user-select: none;
		flex-shrink: 0;
	}

	.header-side {
		display: flex;
		align-items: center;
		gap: 4px;
		flex-shrink: 0;
	}
	.header-side.left {
		min-width: 32px;
	}
	.header-title {
		font-size: 14px;
		font-weight: 600;
		color: var(--text-primary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		flex: 1;
		text-align: center;
		min-width: 0;
	}

	.header-side.right {
		justify-content: flex-end;
	}

	.icon-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: 8px;
		border: none;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		transition: all 0.15s;
	}
	.icon-btn:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}
	.icon-btn.active {
		background: var(--bg-hover);
		color: var(--accent);
	}

	main {
		flex: 1;
		overflow-y: auto;
		padding: 16px;
	}
	main.no-scroll,
	main.no-scroll :global(*) {
		overflow: hidden !important;
	}

	.size-menu-wrap {
		position: relative;
	}

	.size-dropdown {
		position: absolute;
		top: 100%;
		right: 0;
		margin-top: 6px;
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: 10px;
		padding: 4px;
		z-index: 100;
		min-width: 150px;
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
	}

	.size-option {
		display: flex;
		flex-direction: column;
		gap: 1px;
		width: 100%;
		padding: 8px 12px;
		border: none;
		border-radius: 8px;
		background: transparent;
		color: var(--text-primary);
		cursor: pointer;
		text-align: left;
		font-family: inherit;
		transition: background 0.15s;
	}
	.size-option:hover {
		background: var(--bg-hover);
	}

	.size-divider {
		height: 1px;
		margin: 4px 0;
		background: var(--border);
	}

	.menu-toggle-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 10px 12px;
	}

	.menu-toggle-copy {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}

	.size-label {
		font-size: 13px;
		font-weight: 500;
	}

	.size-desc {
		font-size: 11px;
		color: var(--text-secondary);
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
</style>
