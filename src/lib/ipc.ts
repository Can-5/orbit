/**
 * Thin wrapper around the Tauri IPC bridge.
 *
 * In a real Tauri window it calls `@tauri-apps/api` invoke().
 * In a plain browser (e.g. `pnpm dev` without the shell) every call returns
 * `null` so the UI can show an honest "Tauri host not detected" notice instead
 * of fabricating system data.
 */
import { invoke } from '@tauri-apps/api/core';

export const inTauri =
	typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export type CommandResult<T> = T | null;

export async function ipc<T>(command: string, args?: Record<string, unknown>): Promise<CommandResult<T>> {
	if (!inTauri) return null;
	try {
		return await invoke<T>(command, args);
	} catch (err) {
		console.error(`[orbit ipc] ${command} failed:`, err);
		return null;
	}
}

/** Matches the `ModuleDto` returned by the shell (serde camelCase). */
export interface ModuleDto {
	id: string;
	name: string;
	description: string;
	phase: number;
	enabled: boolean;
	implemented: boolean;
	requiredPermissions: string[];
}

/** Static mirror of the shell registry — only used with the Tauri host offline. */
const PREVIEW_MODULES: ModuleDto[] = [
	{ id: 'orbit_island', name: 'Orbit Island', description: 'Etkileşimli bildirim ve etkinlik adası.', phase: 1, enabled: true, implemented: true,  requiredPermissions: [] },
	{ id: 'ghostdrop',    name: 'GhostDrop',    description: 'Geçici dosya ve içerik alanı.',          phase: 2, enabled: true, implemented: false, requiredPermissions: [] },
	{ id: 'recall',       name: 'Recall',       description: 'Görsel pano geçmişi — arama, favoriler.',phase: 2, enabled: true, implemented: false, requiredPermissions: ['clipboard_read', 'clipboard_monitor'] },
	{ id: 'renamex',      name: 'RenameX',      description: 'Toplu dosya isimlendirme.',              phase: 2, enabled: true, implemented: false, requiredPermissions: ['file_system_watch'] },
	{ id: 'pixelpick',    name: 'PixelPick',    description: 'Ekran cetveli, renk seçici, paletler.', phase: 2, enabled: true, implemented: false, requiredPermissions: ['screen_capture'] },
	{ id: 'snapspace',    name: 'SnapSpace',    description: 'Çalışma alanı profilleri.',              phase: 3, enabled: true, implemented: false, requiredPermissions: ['window_management'] },
	{ id: 'cleanshot',    name: 'CleanShot',    description: 'Ekran görüntüsü al, işaretle, arşivle.', phase: 3, enabled: true, implemented: false, requiredPermissions: ['screen_capture'] },
	{ id: 'apppulse',     name: 'AppPulse',     description: 'CPU/RAM ve işlem izleme.',               phase: 3, enabled: true, implemented: false, requiredPermissions: ['process_metrics'] },
	{ id: 'linknest',     name: 'LinkNest',     description: 'Yerel bağlantı ve yer imi yöneticisi.',  phase: 3, enabled: true, implemented: false, requiredPermissions: ['network'] },
	{ id: 'keyglow',      name: 'KeyGlow',      description: 'Klavye görselleştirme ve ses.',          phase: 4, enabled: true, implemented: false, requiredPermissions: ['keyboard_listen'] },
	{ id: 'themeport',    name: 'ThemePort',    description: 'Tek tema, her uygulama.',                phase: 4, enabled: true, implemented: false, requiredPermissions: ['theme_control'] }
];

export async function listModules(): Promise<{ modules: ModuleDto[]; live: boolean }> {
	const live = await ipc<ModuleDto[]>('list_modules');
	if (live) return { modules: live, live: true };
	return { modules: PREVIEW_MODULES, live: false };
}

/* ---------- Island ambient snapshot (host-pushed; never invented) ---------- */

export interface NowPlaying {
	player: string;
	title: string;
	artist: string;
	position_secs: number;
	length_secs: number | null;
	playing: boolean;
}

export interface DownloadTask {
	id: string;
	label: string;
	received_bytes: number;
	total_bytes: number | null;
	done: boolean;
}

export interface DroppedFile {
	path: string;
	bytes: number;
}

/** serde snake_case keys, matching orbit_core::island::IslandSnapshot. */
export interface IslandSnapshot {
	view: 'collapsed' | 'expanded';
	active: unknown | null;
	queued: number;
	timer: unknown | null;
	now_playing: NowPlaying | null;
	downloads: DownloadTask[];
	downloads_active: number;
	dropped_files: DroppedFile[];
}

export async function islandSnapshot(): Promise<IslandSnapshot | null> {
	return ipc<IslandSnapshot>('island_snapshot');
}

export async function islandDropFiles(paths: string[]): Promise<IslandSnapshot | null> {
	return ipc<IslandSnapshot>('island_drop_files', { paths });
}

export async function islandClearDroppedFiles(): Promise<IslandSnapshot | null> {
	return ipc<IslandSnapshot>('island_clear_dropped_files');
}

/** Start a timer in the host island. Returns the snapshot, or null offline. */
export async function islandStartTimer(label: string, durationSecs: number): Promise<IslandSnapshot | null> {
	return ipc<IslandSnapshot>('island_start_timer', { label, duration_secs: durationSecs });
}
