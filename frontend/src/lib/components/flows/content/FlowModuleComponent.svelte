<script lang="ts">
	import { Pane, Splitpanes } from 'svelte-splitpanes'
	import Tab from '$lib/components/common/tabs/Tab.svelte'
	import Tabs from '$lib/components/common/tabs/Tabs.svelte'
	import Editor from '$lib/components/Editor.svelte'
	import EditorBar, { EDITOR_BAR_WIDTH_THRESHOLD } from '$lib/components/EditorBar.svelte'
	import ModulePreview from '$lib/components/ModulePreview.svelte'
	import { createScriptFromInlineScript, fork } from '$lib/components/flows/flowStateUtils'
	import { flowStore } from '$lib/components/flows/flowStore'
	import SchemaForm from '$lib/components/SchemaForm.svelte'
	import { RawScript, type FlowModule } from '$lib/gen'
	import FlowCard from '../common/FlowCard.svelte'
	import FlowModuleHeader from './FlowModuleHeader.svelte'
	import { flowStateStore } from '../flowState'
	import { scriptLangToEditorLang } from '$lib/utils'
	import PropPickerWrapper from '../propPicker/PropPickerWrapper.svelte'
	import { afterUpdate, getContext, setContext } from 'svelte'
	import type { FlowEditorContext } from '../types'
	import { loadSchemaFromModule } from '../utils'
	import FlowModuleScript from './FlowModuleScript.svelte'
	import FlowModuleEarlyStop from './FlowModuleEarlyStop.svelte'
	import FlowModuleSuspend from './FlowModuleSuspend.svelte'
	import FlowRetries from './FlowRetries.svelte'
	import { getStepPropPicker } from '../previousResults'
	import Tooltip from '$lib/components/Tooltip.svelte'
	import { Kbd } from '$lib/components/common'

	const { selectedId, previewArgs } = getContext<FlowEditorContext>('FlowEditorContext')

	export let flowModule: FlowModule
	export let failureModule: boolean = false

	export let parentModule: FlowModule | undefined = undefined
	export let previousModuleId: string | undefined = undefined

	let editor: Editor
	let modulePreview: ModulePreview
	let websocketAlive = { pyright: false, black: false, deno: false, go: false }
	let selected = 'inputs'
	let wrapper: HTMLDivElement
	let panes: HTMLElement
	let totalTopGap = 0

	let width = 1200

	let inputTransforms: Record<string, any> =
		flowModule.value.type === 'rawscript' || flowModule.value.type === 'script'
			? flowModule.value.input_transforms
			: {}

	$: if (flowModule.value.type === 'rawscript' || flowModule.value.type === 'script') {
		flowModule.value.input_transforms = inputTransforms
	}

	$: stepPropPicker = failureModule
		? { pickableProperties: { previous_result: { error: 'the error message' } }, extraLib: '' }
		: getStepPropPicker($flowStateStore, parentModule, previousModuleId, $flowStore, previewArgs)

	function onKeyDown(event: KeyboardEvent) {
		if ((event.ctrlKey || event.metaKey) && event.key == 'Enter') {
			event.preventDefault()
			selected = 'test'
			modulePreview?.runTestWithStepArgs()
		}
	}

	async function reload(flowModule: FlowModule) {
		const { input_transforms, schema } = await loadSchemaFromModule(flowModule)

		setTimeout(() => {
			if (
				(flowModule.value.type == 'script' || flowModule.value.type == 'rawscript') &&
				JSON.stringify(flowModule.value.input_transforms) !== JSON.stringify(input_transforms)
			) {
				inputTransforms = input_transforms
			}
		})

		if (JSON.stringify(schema) !== JSON.stringify($flowStateStore[flowModule.id].schema)) {
			$flowStateStore[flowModule.id].schema = schema
		}
	}

	afterUpdate(() => {
		totalTopGap = 0
		if (!(wrapper && panes)) return

		const wrapperTop = wrapper.getBoundingClientRect().top
		const panesTop = panes.getBoundingClientRect().top
		totalTopGap = panesTop - wrapperTop
	})
</script>

<svelte:window on:keydown={onKeyDown} />

{#if flowModule.value.type === 'rawscript' || flowModule.value.type === 'script'}
	<div class="h-full" bind:this={wrapper} bind:clientWidth={width}>
		<FlowCard bind:flowModule>
			<svelte:fragment slot="header">
				<FlowModuleHeader
					bind:module={flowModule}
					on:toggleSuspend={() => (selected = 'suspend')}
					on:toggleRetry={() => (selected = 'retries')}
					on:toggleStopAfterIf={() => (selected = 'early-stop')}
					on:fork={async () => {
						const [module, state] = await fork(flowModule)
						flowModule = module
						$flowStateStore[module.id] = state
					}}
					on:createScriptFromInlineScript={async () => {
						const [module, state] = await createScriptFromInlineScript(
							flowModule,
							$selectedId,
							$flowStateStore[flowModule.id].schema
						)
						flowModule = module
						$flowStateStore[module.id] = state
					}}
				/>
			</svelte:fragment>

			{#if flowModule.value.type === 'rawscript'}
				<div class="border-b-2 shadow-sm p-1 mb-1">
					<EditorBar
						{editor}
						lang={flowModule.value['language'] ?? 'deno'}
						{websocketAlive}
						iconOnly={width < 768}
					/>
				</div>
			{/if}

			<div
				bind:this={panes}
				class="h-full"
				style="max-height: calc(100% - {totalTopGap}px) !important;"
			>
				<Splitpanes horizontal>
					<Pane size={50} minSize={20}>
						{#if flowModule.value.type === 'rawscript'}
							<div on:mouseleave={() => reload(flowModule)} class="h-full">
								<Editor
									bind:websocketAlive
									bind:this={editor}
									class="h-full px-2"
									bind:code={flowModule.value.content}
									deno={flowModule.value.language === RawScript.language.DENO}
									lang={scriptLangToEditorLang(flowModule.value.language)}
									automaticLayout={true}
									cmdEnterAction={async () => {
										selected = 'test'
										await reload(flowModule)
										modulePreview?.runTestWithStepArgs()
									}}
									formatAction={() => reload(flowModule)}
								/>
							</div>
						{:else if flowModule.value.type === 'script'}
							<FlowModuleScript path={flowModule.value.path} />
						{/if}
					</Pane>
					<Pane size={50} minSize={20}>
						<Tabs bind:selected>
							<Tab value="inputs"
								><Tooltip>
									Move the focus outside of the text editor to recompute the inputs or press
									<Kbd>Ctrl/Cmd</Kbd> + <Kbd>S</Kbd>
								</Tooltip>Inputs</Tab
							>
							<Tab value="test">Test</Tab>
							<Tab value="retries">Retries</Tab>
							{#if !$selectedId.includes('failure')}
								<Tab value="early-stop">Early Stop</Tab>
								<Tab value="suspend">Sleep/Suspend</Tab>
							{/if}
						</Tabs>
						<div class="h-[calc(100%-32px)]">
							{#if selected === 'inputs'}
								<div class="h-full overflow-auto">
									<PropPickerWrapper
										priorId={previousModuleId}
										pickableProperties={stepPropPicker.pickableProperties}
									>
										<SchemaForm
											schema={$flowStateStore[$selectedId].schema}
											inputTransform={true}
											importPath={$selectedId}
											bind:args={flowModule.value.input_transforms}
											bind:extraLib={stepPropPicker.extraLib}
										/>
									</PropPickerWrapper>
								</div>
							{:else if selected === 'test'}
								<ModulePreview
									bind:this={modulePreview}
									mod={flowModule}
									schema={$flowStateStore[$selectedId].schema}
								/>
							{:else if selected === 'retries'}
								<FlowRetries bind:flowModule class="px-4 pb-4 h-full overflow-auto" />
							{:else if selected === 'early-stop'}
								<FlowModuleEarlyStop
									{previousModuleId}
									bind:flowModule
									class="px-4 pb-4 h-full overflow-auto"
									{parentModule}
								/>
							{:else if selected === 'suspend'}
								<div class="px-4 pb-4 h-full overflow-auto">
									<FlowModuleSuspend {previousModuleId} bind:flowModule />
								</div>
							{/if}
						</div>
					</Pane>
				</Splitpanes>
			</div>
		</FlowCard>
	</div>
{:else}
	Incorrect flow module type
{/if}
