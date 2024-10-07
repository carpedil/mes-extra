<script lang="ts">
	import { page } from '$app/stores';
	import Button from '$lib/components/ui/button/button.svelte';
	import type { ProcessFlowVersions, ProductDef, ProductDefVersions } from '$lib/schema';
	import { invoke } from '@tauri-apps/api';
	import { onMount } from 'svelte';
	import { writable } from 'svelte/store';
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
	$: {
		console.log(
			'selectedProductDefName',
			$selectedProductDefName,
			'selectedProductDefVersion',
			$selectedProductDefVersion,
			'selectedProcessFlowName',
			$selectedProcessFlowName,
			'selectedProcessFlowVersion',
			$selectedProcessFlowVersion
		);
	}
</script>

<div>
	<form action="" class="border grid grid-rows-2 grid-cols-2 gap-2 shrink p-2">
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
				<Button size="sm" variant="outline" type="submit" class="w-fit">Query</Button>
				<Button
					size="sm"
					variant="outline"
					type="reset"
					class="w-fit"
					on:click={() => history.go(0)}>Reset</Button
				>
			</div>
		{/if}
	</form>
	<div>data list</div>
</div>
