<script lang="ts">
	import { Tabs } from 'bits-ui';
	import { ListChecks } from 'lucide-svelte';
	import FlowExport from './FlowExport.svelte';
</script>

<div>
	<Tabs.Root value="Flow Export" class="h-fit">
		<Tabs.List class="flex gap-4 w-[100vw] text-xs border-b-2">
			<Tabs.Trigger value="Flow Export" class="flex items-center text-xs">
				<ListChecks />Flow Export</Tabs.Trigger
			>
			<Tabs.Trigger value="Rcipe Not In Use" class="flex items-center text-xs"
				><ListChecks />Rcipe Not In Use</Tabs.Trigger
			>
			<Tabs.Trigger value="With WPH Recipes" class="flex items-center text-xs"
				><ListChecks />With WPH Recipes</Tabs.Trigger
			>
			<Tabs.Trigger value="Without WPH Recipes" class="flex items-center text-xs"
				><ListChecks />Without WPH Recipes</Tabs.Trigger
			>
		</Tabs.List>
		<Tabs.Content value="Flow Export" class="p-1"><FlowExport /></Tabs.Content>
		<Tabs.Content value="Rcipe Not In Use" class="p-1">Rcipe Not In Use</Tabs.Content>
		<Tabs.Content value="With WPH Recipes" class="p-1">With WPH Recipes</Tabs.Content>
		<Tabs.Content value="Without WPH Recipes" class="p-1">Without WPH Recipes</Tabs.Content>
	</Tabs.Root>
</div>
