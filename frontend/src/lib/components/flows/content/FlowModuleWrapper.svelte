<script lang="ts">
	import { Script, type FlowModule } from '$lib/gen'
	import { getContext } from 'svelte'

	import type { FlowEditorContext } from '../types'
	import FlowBranchesWrapper from './FlowBranchesWrapper.svelte'
	import FlowLoop from './FlowLoop.svelte'
	import FlowModuleComponent from './FlowModuleComponent.svelte'
	import FlowBranchAllWrapper from './FlowBranchAllWrapper.svelte'
	import FlowBranchOneWrapper from './FlowBranchOneWrapper.svelte'
	import {
		createInlineScriptModule,
		createLoop,
		createBranches,
		pickScript,
		createBranchAll
	} from '$lib/components/flows/flowStateUtils'
	import FlowInputs from './FlowInputs.svelte'
	import { flowStateStore, type FlowModuleState } from '../flowState'
	import Tooltip from '$lib/components/Tooltip.svelte'
	import { Alert } from '$lib/components/common'

	const { selectedId } = getContext<FlowEditorContext>('FlowEditorContext')

	export let flowModule: FlowModule

	// These pointers are used to easily access previewArgs of parent module, and previous module
	// Pointer to parent module, only defined within Branches or Loops.
	export let parentModule: FlowModule | undefined = undefined
	// Pointer to previous module, for easy access to testing results
	export let previousModuleId: string | undefined = undefined
</script>

{#if flowModule.id === $selectedId}
	{#if flowModule.value.type === 'forloopflow'}
		<FlowLoop bind:mod={flowModule} {parentModule} {previousModuleId} />
	{:else if flowModule.value.type === 'branchone'}
		<FlowBranchesWrapper
			type={flowModule.value.type}
			{previousModuleId}
			bind:flowModule
			{parentModule}
		/>
	{:else if flowModule.value.type === 'branchall'}
		<FlowBranchesWrapper
			type={flowModule.value.type}
			{previousModuleId}
			bind:flowModule
			{parentModule}
		/>
	{:else if flowModule.value.type === 'identity'}
		{#if $selectedId == 'failure'}
			<Alert type="info" title="Error handlers are triggered upon non recovered errors">
				If defined, the error handler will take as input, the result of the step that errored (which
				has its error in the 'error field').
				<br />
				<br />
				Steps are retried until they succeed, or until the maximum number of retries defined for that
				spec is reached, at which point the error handler is called.
			</Alert>
		{/if}
		<h1 class="p-4"
			>Select a step kind <Tooltip
				>Until being defined, this step acts as an identify function, returning as result its input
				and assigning it a key 'previous_result' if the input is not a json object</Tooltip
			></h1
		>
		<FlowInputs
			shouldDisableTriggerScripts={parentModule !== undefined || previousModuleId !== undefined}
			on:loop={async () => {
				const [module, state] = await createLoop(flowModule.id)
				flowModule = module
				$flowStateStore[module.id] = state
			}}
			on:branchone={async () => {
				const [module, state] = await createBranches(flowModule.id)
				flowModule = module
				$flowStateStore[module.id] = state
			}}
			on:branchall={async () => {
				const [module, state] = await createBranchAll(flowModule.id)
				flowModule = module
				$flowStateStore[module.id] = state
			}}
			on:pick={async ({ detail }) => {
				const { path, summary } = detail
				const [module, state] = await pickScript(path, summary, flowModule.id)
				flowModule = module
				$flowStateStore[module.id] = state
			}}
			on:new={async ({ detail }) => {
				const { language, kind, subkind } = detail

				const [module, state] = await createInlineScriptModule(
					language,
					kind,
					subkind,
					flowModule.id
				)

				if (kind == Script.kind.APPROVAL) {
					module.suspend = { required_events: 1, timeout: 1800 }
				}

				flowModule = module
				$flowStateStore[module.id] = state
			}}
			failureModule={$selectedId === 'failure'}
		/>
	{:else if flowModule.value.type === 'rawscript' || flowModule.value.type === 'script'}
		<FlowModuleComponent bind:flowModule {parentModule} {previousModuleId} />
	{/if}
{:else if flowModule.value.type === 'forloopflow'}
	{#each flowModule.value.modules as submodule, index (index)}
		<svelte:self
			bind:flowModule={submodule}
			bind:parentModule={flowModule}
			previousModuleId={flowModule.value.modules[index - 1]?.id}
		/>
	{/each}
{:else if flowModule.value.type === 'branchone'}
	{#if $selectedId === `${flowModule?.id}-branch-default`}
		<div class="p-4 text-sm">Default branch</div>
	{:else}
		{#each flowModule.value.default as submodule, index}
			<svelte:self
				bind:flowModule={submodule}
				bind:parentModule={flowModule}
				previousModuleId={flowModule.value.default[index - 1]?.id}
			/>
		{/each}
	{/if}
	{#each flowModule.value.branches as branch, branchIndex (branchIndex)}
		{#if $selectedId === `${flowModule?.id}-branch-${branchIndex}`}
			<FlowBranchOneWrapper bind:branch parentModule={flowModule} {previousModuleId} />
		{:else}
			{#each branch.modules as submodule, index}
				<svelte:self
					bind:flowModule={submodule}
					bind:parentModule={flowModule}
					previousModuleId={flowModule.value.branches[branchIndex].modules[index - 1]?.id}
				/>
			{/each}
		{/if}
	{/each}
{:else if flowModule.value.type === 'branchall'}
	{#each flowModule.value.branches as branch, branchIndex (branchIndex)}
		{#if $selectedId === `${flowModule?.id}-branch-${branchIndex}`}
			<FlowBranchAllWrapper bind:branch />
		{:else}
			{#each branch.modules as submodule, index}
				<svelte:self
					bind:flowModule={submodule}
					bind:parentModule={flowModule}
					previousModuleId={flowModule.value.branches[branchIndex].modules[index - 1]?.id}
				/>
			{/each}
		{/if}
	{/each}
{/if}
