import type { Flow, FlowModule, ForloopFlow, InputTransform } from '$lib/gen'
import { get, writable, derived } from 'svelte/store'
import { flowStateStore, initFlowState } from './flowState'
import { numberToChars } from './utils'

export type FlowMode = 'push' | 'pull'

export const flowStore = writable<Flow>({
	summary: '',
	value: { modules: [] },
	path: '',
	edited_at: '',
	edited_by: '',
	archived: false,
	extra_perms: {}
})

export function dfs(modules: FlowModule[], previewOrder: boolean = false): string[] {
	let result: string[] = []
	for (const module of modules) {
		if (module.value.type == 'forloopflow') {
			if (previewOrder) {
				result = result.concat(module.id)
			}
			result = result.concat(dfs(module.value.modules))
			if (!previewOrder) {
				result = result.concat(module.id)
			}
		} else if (module.value.type == 'branchone') {
			if (previewOrder) {
				result = result.concat(module.id)
			}
			result = result.concat(dfs(module.value.branches.map((b) => b.modules).flat().concat(module.value.default)))
			if (!previewOrder) {
				result = result.concat(module.id)
			}
		} else if (module.value.type == 'branchall') {
			if (previewOrder) {
				result = result.concat(module.id)
			}
			result = result.concat(dfs(module.value.branches.map((b) => b.modules).flat()))
			if (!previewOrder) {
				result = result.concat(module.id)
			}
		} else {
			result.push(module.id)
		}
	}
	return result
}

export const flowIds = derived(flowStore, flow => dfs(flow.value.modules))

export async function initFlow(flow: Flow) {

	let counter = 40
	for (const mod of flow.value.modules) {
		migrateFlowModule(mod)
		let val = mod.value
		if (val.type == 'forloopflow') {
			let flowVal = val as ForloopFlow & { value?: { modules?: FlowModule[] } }
			if (flowVal.value && flowVal.value.modules) {
				flowVal.modules = flowVal.value.modules
				flowVal.value = undefined
			}
			flowVal.modules.forEach(migrateFlowModule)

		}
	}

	await initFlowState(flow)
	flowStore.set(flow)

	function migrateFlowModule(mod: FlowModule) {
		if (mod.id == undefined) {
			mod.id = numberToChars(counter++)
		}
		let modVal = mod as FlowModule & {
			input_transform?: Record<string, InputTransform>
			stop_after_if_expr?: string
			skip_if_stopped?: boolean
		}
		if (modVal.input_transform) {
			modVal.input_transforms = modVal.input_transform
			delete modVal.input_transform
		}
		if (
			(modVal.input_transforms && modVal.value.type == 'script') ||
			modVal.value.type == 'rawscript'
		) {
			if (modVal.input_transforms && Object.keys(modVal.input_transforms).length > 0) {
				modVal.value.input_transforms = modVal.input_transforms
				delete modVal.input_transforms
			}
		}
		if (modVal.stop_after_if_expr) {
			modVal.stop_after_if = {
				expr: modVal.stop_after_if_expr,
				skip_if_stopped: modVal.skip_if_stopped
			}
			delete modVal.stop_after_if_expr
			delete modVal.skip_if_stopped
		}
	}
}

export async function copyFirstStepSchema() {
	const flowState = get(flowStateStore)
	flowStore.update((flow) => {
		const firstModuleId = flow.value.modules[0].id

		if (flowState[firstModuleId]) {
			flow.schema = flowState[firstModuleId].schema
		}
		return flow
	})
}
