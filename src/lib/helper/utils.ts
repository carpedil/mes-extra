import { ColumnData, ExportSpecInput, SrvResult } from '$lib/schema';
import { onDestroy } from 'svelte';

export function onInterval(callback: () => void, milliseconds: number | undefined) {
	const interval = setInterval(callback, milliseconds);

	onDestroy(() => {
		clearInterval(interval);
	});
}

export const handleSrvResult = <T>(data: T, errors: string): SrvResult<T> => {
	console.log(data, errors);
	if (errors) {
		return new SrvResult(500, errors, null);
	}
	return new SrvResult(200, 'success', data);
};

export const serializeHeaders = (headers: ColumnData[]): string => {
	return headers
		.map((header) => {
			return `{columnName: "${header.column_name}",dataType:"${header.data_type}",dataLen:${header.data_len}}`;
		})
		.join(',');
};

export const serializeExportSpecInput = (spec: ExportSpecInput[]): string => {
	return spec
		.map((s) => {
			return `{ 
		  tableName: "${s.table_name}", 
		  headers: [${serializeHeaders(s.headers)}], 
		  querySql: "${s.query_sql.replaceAll('\n', ' ')}" 
		}`;
		})
		.join(', ');
};
