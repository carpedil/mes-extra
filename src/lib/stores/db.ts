import { ExportSpecInput, TableColumnsInfo } from '$lib/schema';
import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api';

export let table_list = writable<TableColumnsInfo[]>([]);
export let table_selected = writable<ExportSpecInput>(new ExportSpecInput());

export const fetch_table_list = async () => {
	// const res = (await invoke('get_table_infos', {
	// 	sync_input:{
	// 		sync_no: '2024-09-21',
	// 		sync_version: 2
	// 	}
	// })) as any;

	const res = (await invoke('get_table_infos')) as any;
	console.log('get_table_infos:', res);
	table_list.set(res.data);
};

export const set_table_selected = (table: ExportSpecInput) => {
	table_selected.set(table);
};
