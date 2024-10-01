import { z } from 'zod';
import { format } from 'sql-formatter';

export const formSchema = z.object({
	env: z.enum(['', 'Test', 'Dev', 'Prod']).default('Test'),
	dbType: z.enum(['', 'Postgress', 'Oracle', 'MySQL']),
	url: z.string().max(20),
	username: z.string().max(15),
	password: z.string().max(15),
	isActive: z.boolean().default(false)
});

export type FormSchema = typeof formSchema;

export const validateFormData = (form: Object): string[] => {
	const emptyFields: string[] = [];
	// 检查对象中的每个属性
	for (const [key, value] of Object.entries(form)) {
		// 如果属性是字符串类型，并且值为空字符串，则添加到emptyFields数组中
		if (typeof value === 'string' && value.trim() === '') {
			emptyFields.push(key);
		}
	}
	return emptyFields;
};

export const envs = ['Test', 'Dev', 'Prod'];
export const dbTypes = ['Postgress', 'Oracle', 'MySQL'];

export class ConnectionConfig {
	public id: string = '';
	public db_type: string = '';
	public env: string = '';
	public url: string = '';
	public username: string = '';
	public password: string = '';
	public is_active: boolean = false;
	public abandonedTableList: string = '';
}

export class TableColumnsInfo {
	public sync_no: string = '';
	public sync_version: number = 0;
	public table_name: string = '';
	public column_infos: ColumnData[] = [];
}

export class ColumnData {
	public column_name: string = '';
	public data_type: string = '';
	public data_len: number = 0;
}

export class ExportSpecInput {
	public sync_no: string = '';
	public sync_version: number = 0;
	public table_name: string = '';
	public headers: ColumnData[] = [];
	public query_sql: string = '';

	constructor() {
		this.table_name = '';
		this.headers = [];
		this.query_sql = '';
	}

	set_sync_no(sync_no: string) {
		this.sync_no = sync_no;
	}

	set_sync_version(sync_version: number) {
		this.sync_version = sync_version;
	}

	set_table_name(table_name: string) {
		this.table_name = table_name;
	}

	set_headers(column_infos: ColumnData[]) {
		this.headers = column_infos;
	}

	set_query_sql(where_clause: string) {
		const select_fields = this.headers.map((header) => header.column_name).join(', ');
		where_clause === ''
			? (this.query_sql = `SELECT ${select_fields} FROM ${this.table_name} ORDER BY ${this.headers[1].column_name}`)
			: (this.query_sql = `SELECT ${select_fields} FROM ${this.table_name} ${where_clause.toUpperCase()} ORDER BY ${this.headers[1].column_name} `);

		this.query_sql = format(this.query_sql.trim(), { language: 'sql' });
	}
}

export class SrvResult<T> {
	public code: number = 200;
	public message: string = 'success';
	public data: T;

	constructor(code: number, message: string, data: any) {
		this.code = code;
		this.message = message;
		this.data = data;
	}
}

export class BannedTableInfo {
	public tableName: string = '';
	public tableList: string[] = [];
	constructor(tableName: string, tableList: string[]) {
		this.tableName = tableName;
		const notExists = !tableList.includes(tableName);
		this.tableList = notExists && tableName != '' ? [...tableList, tableName] : tableList;
	}
}

export class TableData {
	data: string[] = [];
}
