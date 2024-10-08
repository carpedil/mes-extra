<script lang="ts">
	import { page } from '$app/stores';
	import Button from '$lib/components/ui/button/button.svelte';
	import type { ProcessFlowVersions, ProductDef, ProductDefVersions } from '$lib/schema';
	import {
		flow_export_table_headers,
		flow_export_table_values,
		table_headers,
		table_values
	} from '$lib/stores/db';
	import { invoke } from '@tauri-apps/api';
	import * as Table from '$lib/components/ui/table/index';
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { writable } from 'svelte/store';
	import * as XLSX from 'xlsx';
	let selectedProductDefName = writable('');
	let selectedProductDefVersion = writable('');
	let selectedProcessFlowName = writable<string | undefined>('');
	let selectedProcessFlowVersion = writable('');

	let productDefNames = writable<ProductDef[]>([]);
	let productDefVersions = writable<ProductDefVersions[]>([]);
	let processFlowVersions = writable<ProcessFlowVersions[]>([]);

	onMount(async () => {
		let res = (await invoke('get_product_def_list')) as any;
		productDefNames.set(res.data);
		console.log('get_product_def_list:', productDefNames, res);
	});

	const handelProductChange = async (evt: any) => {
		const product_def_name = evt.target.value;
		selectedProductDefName.set(product_def_name);
		let product_vers_res = (await invoke('get_product_def_ver_list', {
			product_def_name: product_def_name
		})) as any;
		productDefVersions.update((vers) => (vers = product_vers_res.data as ProductDefVersions[]));
		console.log('get_product_def_ver_list', productDefVersions);
		let flowName = $productDefNames.filter((p) => p.product_def_name == $selectedProductDefName)[0]
			.process_flow_name;
		selectedProcessFlowName.set(flowName);

		let flow_vers_res = (await invoke('get_process_flow_ver_list', {
			process_flow_name: $selectedProcessFlowName
		})) as any;
		processFlowVersions.update((vers) => (vers = flow_vers_res.data as ProcessFlowVersions[]));
	};
	const handleProductDefVersionChange = async (evt: any) => {
		selectedProductDefVersion.set(evt.target.value);
	};

	const handleProcessFlowVersionChange = async (evt: any) => {
		selectedProcessFlowVersion.set(evt.target.value);
	};

	const handleQuery = async () => {
		if (
			$selectedProductDefName === '' ||
			$selectedProductDefVersion === '' ||
			$selectedProcessFlowName === '' ||
			$selectedProcessFlowVersion === ''
		) {
			toast.error('Parameter verification failed', {
				description: 'Please select specific product and process information',
				position: 'top-right'
			});
			return;
		}
		let res = (await invoke('get_flow_export', {
			input: {
				product_def_name: $selectedProductDefName,
				product_def_ver: $selectedProductDefVersion,
				process_flow_name: $selectedProcessFlowName,
				process_flow_ver: $selectedProcessFlowVersion
			}
		})) as any;
		console.log('get_flow_export:', res);
		flow_export_table_headers.set(res.data.headers);
		flow_export_table_values.set(res.data.values);
	};

	const handleDownload = () => {
		const ws: XLSX.WorkSheet = XLSX.utils.table_to_sheet(
			document.getElementById('flow-export-table')
		);
		const wb: XLSX.WorkBook = XLSX.utils.book_new();
		XLSX.utils.book_append_sheet(wb, ws, $selectedProductDefName);
		XLSX.writeFile(
			wb,
			`${$selectedProductDefName}.${$selectedProductDefVersion}.${$selectedProcessFlowName}.${$selectedProcessFlowVersion}.xlsx`
		);
	};
</script>

<div>
	<form action="" class="border grid grid-rows-2 grid-cols-2 gap-2 shrink p-2">
		{#if $productDefNames.length != 0}
			<label for="productName"
				>ProductDefName:
				<select name="products" id="products" on:change={handelProductChange}>
					<option value="---" disabled selected>---</option>
					{#each $productDefNames as product}
						<option value={product.product_def_name} class="border-2"
							>{product.product_def_name}</option
						>
					{/each}
				</select>
			</label>
		{/if}
		{#if $selectedProcessFlowName != ''}
			<label for="ProcessFlowName"
				>ProcessFlowName:
				<input type="text" id="ProcessFlowName" value={$selectedProcessFlowName} disabled />
			</label>
		{/if}
		{#if $productDefVersions.length != 0}
			<label for="productDefVersion"
				>ProductDefVersion:
				<select name="products" id="productDefVersion" on:change={handleProductDefVersionChange}>
					<option value="--" selected disabled>--</option>
					{#each $productDefVersions as version}
						<option value={version.product_def_ver}
							>{version.product_def_ver}({version.product_def_state})</option
						>
					{/each}
				</select>
			</label>
		{/if}
		{#if $processFlowVersions.length != 0}
			<label for="ProcessFlowVersions"
				>ProcessFlowVersion:
				<select
					name="ProcessFlowVersions"
					id="ProcessFlowVersions"
					on:change={handleProcessFlowVersionChange}
				>
					<option value="---" disabled selected>---</option>
					{#each $processFlowVersions as version}
						<option value={version.process_flow_ver}
							>{version.process_flow_ver}({version.process_flow_state})</option
						>
					{/each}
				</select>
			</label>
			<div></div>
			<div>
				<Button size="sm" variant="outline" type="submit" class="w-fit" on:click={handleQuery}
					>Query</Button
				>
				<Button
					size="sm"
					variant="outline"
					type="reset"
					class="w-fit"
					on:click={() => history.go(0)}>Reset</Button
				>
				<Button size="sm" variant="outline" class="w-fit" on:click={handleDownload}>Download</Button
				>
			</div>
		{/if}
	</form>
	<div class="w-[88.5vw] h-[80vh] overflow-auto text-nowrap">
		<Table.Root class="text-xs text-slate-500" id="flow-export-table">
			<Table.Header class="border">
				<Table.Row>
					{#each $flow_export_table_headers as header}
						<Table.Head class="border-2">{header}</Table.Head>
					{/each}
				</Table.Row>
			</Table.Header>
			<Table.Body>
				{#each $flow_export_table_values as item}
					<Table.Row>
						{#each item as value}
							<Table.Cell class="border font-medium">{value}</Table.Cell>
						{/each}
					</Table.Row>
				{/each}
			</Table.Body>
		</Table.Root>
	</div>
</div>
