<script lang="ts">
	import * as Accordion from '$lib/components/ui/accordion';
	import * as Card from '$lib/components/ui/card/index';
	import { Button } from '$lib/components/ui/button/index';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index';
	import { Trash2, BugPlay } from 'lucide-svelte';
	import { onMount } from 'svelte';
	import { fetch_configs_list, tcc_list } from '$lib/stores/configs';
	import { toast } from 'svelte-sonner';
	import { invoke } from '@tauri-apps/api';
	import type { ConnectionConfig } from '$lib/schema';

	onMount(async () => {
		await fetch_configs_list();
	});

	const handleActive = async (e: any) => {
		let id = e.target.id;
		let res = (await invoke('active_config_by_id', { id })) as ConnectionConfig;
		if (!res) {
			toast.error('Data Fetching Failed', {
				description: 'Data Fetching Failed',
				position: 'top-right'
			});
			return;
		} else {
			toast.success('success', {
				description: `successful activate tcc`,
				position: 'top-right'
			});
		}
		await fetch_configs_list();
	};
	const handleDelete = async (e: any) => {
		let id = e.target.id;
		let res = (await invoke('delete_config_by_id', { id })) as number;
		if (!res) {
			toast.error('Data Fetching Failed', {
				description: 'Data delete failed',
				position: 'top-right'
			});
			return;
		} else {
			toast.success('success', {
				description: `operation done successfully`,
				position: 'top-right'
			});
		}
		await fetch_configs_list();
	};
</script>

{#each $tcc_list as data}
	<Card.Root
		class="h-[210px] w-[300px] p-0 text-sm {data.is_active ? 'border-2 border-green-500' : ''}"
	>
		<Card.Header class="flex flex-row items-center justify-between">
			<div class="flex flex-col gap-1">
				<Card.Title>{data.env}</Card.Title>
				<Card.Description>{data.id}</Card.Description>
			</div>
			<div class="flex">
				{#if !data.is_active}
					<AlertDialog.Root>
						<AlertDialog.Trigger asChild let:builder>
							<Button variant="outline" size="icon" builders={[builder]}>
								<BugPlay id={data.id} color="green" /></Button
							>
						</AlertDialog.Trigger>
						<AlertDialog.Content>
							<AlertDialog.Header>
								<AlertDialog.Title>Are you absolutely sure to activate it?</AlertDialog.Title>
								<AlertDialog.Description>
									This action will effective immediately and Will disable the currently activated
									version .
								</AlertDialog.Description>
							</AlertDialog.Header>
							<AlertDialog.Footer>
								<AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
								<AlertDialog.Action>
									<Button id={data.id} on:click={handleActive} class="w-full">Continue</Button>
								</AlertDialog.Action>
							</AlertDialog.Footer>
						</AlertDialog.Content>
					</AlertDialog.Root>
					<AlertDialog.Root>
						<AlertDialog.Trigger asChild let:builder>
							<Button variant="outline" size="icon" id={data.id} builders={[builder]}>
								<Trash2 id={data.id} color="gray" /></Button
							>
						</AlertDialog.Trigger>
						<AlertDialog.Content>
							<AlertDialog.Header>
								<AlertDialog.Title>Are you absolutely sure?</AlertDialog.Title>
								<AlertDialog.Description>
									This action cannot be undone. This will permanently delete this data from servers.
								</AlertDialog.Description>
							</AlertDialog.Header>
							<AlertDialog.Footer>
								<AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
								<AlertDialog.Action>
									<Button id={data.id} on:click={handleDelete} class="w-full">Continue</Button>
								</AlertDialog.Action>
							</AlertDialog.Footer>
						</AlertDialog.Content>
					</AlertDialog.Root>
				{/if}
			</div>
		</Card.Header>
		<Card.Content>
			<Accordion.Root>
				<Accordion.Item value="content">
					<Accordion.Trigger><span class="text-gray-400">{data.url}</span></Accordion.Trigger>
					<Accordion.Content>
						{data.username}/{data.password}
					</Accordion.Content>
				</Accordion.Item>
			</Accordion.Root>
		</Card.Content>
	</Card.Root>
{/each}
