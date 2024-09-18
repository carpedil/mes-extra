import type { ConnectionConfig } from '$lib/schema';
import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api';

export let tcc_list = writable<ConnectionConfig[]>([]);

export const fetch_configs_list = async () => {
	const res = (await invoke('get_all_configs')) as ConnectionConfig[];
	console.log('Received TCC list:', res);
	tcc_list.set(res);
};
