import { z } from 'zod';

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
	public query_sql: string = '';
	public column_infos: ColumnData[] = [];
}

export class ColumnData {
	public column_name: string = '';
	public data_type: string = '';
	public data_len: number = 0;
}

export class TableData {
	public headers: string[] = [];
	public values: string[][] = [];

	constructor(headers: string[], values: string[][]) {
		this.headers = headers;
		this.values = values;
	}
}

export type ProductDef = {
	product_def_name: string;
	process_flow_name: string;
};

export type ProductDefVersions = {
	product_def_ver: string;
	product_def_state: string;
};

export type ProcessFlow = {
	process_flow_name: string;
	process_flow_state: string;
};

export type ProcessFlowVersions = {
	process_flow_ver: string;
	process_flow_state: string;
};
