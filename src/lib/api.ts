import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export interface ChecklistMeta {
	title: string;
	description?: string;
	tags: string[];
}

export interface ChecklistItem {
	title: string;
	description?: string;
	code?: string;
	codeLang?: string;
	note?: string;
	skipReason?: string;
	checked: boolean;
}

export interface Section {
	heading: string;
	items: ChecklistItem[];
}

export interface Checklist {
	slug: string;
	meta: ChecklistMeta;
	sections: Section[];
}

export interface ChecklistSummary {
	slug: string;
	title: string;
	description?: string;
	tags: string[];
}

export interface RunMetadata {
	template: string;
	title: string;
	date: string;
	notes?: string;
}

export interface RunSummary {
	template: string;
	filename: string;
	title: string;
	date: string;
	notes?: string;
	checked: number;
	total: number;
}

export interface Run {
	meta: RunMetadata;
	sections: Section[];
}

export interface WindowPresets {
	compact: [number, number];
	sidebar_width: number;
	sidebar_dock: string;
	large: [number, number];
}

export interface AppConfig {
	data_repo_path?: string;
	auto_commit: boolean;
	always_on_top: boolean;
	auto_scroll: boolean;
	window_size?: string;
	window_presets: WindowPresets;
}

export interface SettingsConfigUpdate {
	data_repo_path?: string;
	auto_commit: boolean;
	always_on_top: boolean;
	auto_scroll: boolean;
	window_presets: WindowPresets;
}

export type ChecklistChangeKind = 'created' | 'updated' | 'deleted';

export interface ChecklistChangeEvent {
	kind: ChecklistChangeKind;
	slug: string;
}

export function listChecklists(): Promise<ChecklistSummary[]> {
	return invoke('list_checklists');
}

export function getChecklist(slug: string): Promise<Checklist> {
	return invoke('get_checklist', { slug });
}

export function addSampleChecklist(): Promise<string> {
	return invoke('add_sample_checklist');
}

export function createChecklist(
	title: string,
	description: string | undefined,
	tags: string[],
	sections: Section[]
): Promise<string> {
	return invoke('create_checklist', { title, description, tags, sections });
}

export function updateChecklist(
	slug: string,
	title: string,
	description: string | undefined,
	tags: string[],
	sections: Section[]
): Promise<void> {
	return invoke('update_checklist', {
		slug,
		title,
		description,
		tags,
		sections
	});
}

export function deleteChecklist(slug: string): Promise<void> {
	return invoke('delete_checklist', { slug });
}

export function saveRun(
	templateSlug: string,
	title: string,
	notes: string | undefined,
	sections: Section[]
): Promise<string> {
	return invoke('save_run', { templateSlug, title, notes, sections });
}

export function listRuns(): Promise<RunSummary[]> {
	return invoke('list_runs');
}

export function getRun(template: string, filename: string): Promise<Run> {
	return invoke('get_run', { template, filename });
}

export function deleteRun(template: string, filename: string): Promise<void> {
	return invoke('delete_run', { template, filename });
}

export function getConfig(): Promise<AppConfig> {
	return invoke('get_config');
}

export function updateConfig(config: SettingsConfigUpdate): Promise<void> {
	return invoke('update_config', { config });
}

export function setWindowSize(size: string): Promise<void> {
	return invoke('set_window_size', { size });
}

export function getDefaultDataPath(): Promise<string> {
	return invoke('get_default_data_path');
}

export function checkIsGitRepo(path: string): Promise<boolean> {
	return invoke('check_is_git_repo', { path });
}

export function listenChecklistChanges(
	handler: (event: ChecklistChangeEvent) => void
): Promise<UnlistenFn> {
	return listen<ChecklistChangeEvent>('checklists://changed', ({ payload }) => handler(payload));
}
