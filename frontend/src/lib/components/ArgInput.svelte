<script lang="ts">
	import { faChevronDown, faChevronUp, faMinus, faPlus } from '@fortawesome/free-solid-svg-icons'

	import { setInputCat as computeInputCat, type InputCat } from '$lib/utils'
	import { Button } from './common'
	import { createEventDispatcher } from 'svelte'
	import Icon from 'svelte-awesome'
	import FieldHeader from './FieldHeader.svelte'
	import ObjectResourceInput from './ObjectResourceInput.svelte'
	import ObjectTypeNarrowing from './ObjectTypeNarrowing.svelte'
	import ResourcePicker from './ResourcePicker.svelte'
	import StringTypeNarrowing from './StringTypeNarrowing.svelte'
	import SchemaForm from './SchemaForm.svelte'
	import type { SchemaProperty } from '$lib/common'
	import SimpleEditor from './SimpleEditor.svelte'

	export let label: string = ''
	export let value: any

	export let defaultValue: any = undefined

	export let description: string = ''
	export let format: string = ''
	export let contentEncoding: 'base64' | 'binary' | undefined = undefined
	export let type: string | undefined = undefined
	export let required = false
	export let pattern: undefined | string = undefined
	export let valid = required ? false : true
	export let maxRows = 10
	export let enum_: string[] | undefined = undefined
	export let disabled = false
	export let editableSchema = false
	export let itemsType:
		| { type?: 'string' | 'number' | 'bytes'; contentEncoding?: 'base64' }
		| undefined = undefined
	export let displayHeader = true
	export let properties: { [name: string]: SchemaProperty } | undefined = undefined

	let seeEditable: boolean = enum_ != undefined || pattern != undefined
	const dispatch = createEventDispatcher()

	$: maxHeight = maxRows ? `${1 + maxRows * 1.2}em` : `auto`

	$: validateInput(pattern, value)

	let error: string = ''

	let el: HTMLElement | undefined = undefined

	export let editor: SimpleEditor | undefined = undefined

	let rawValue: string | undefined = undefined

	$: {
		if (rawValue) {
			try {
				value = JSON.parse(rawValue)
			} catch (err) {
				error = err.toString()
			}
		}
	}

	$: {
		if (inputCat === 'object') {
			evalValueToRaw()
			validateInput(pattern, value)
		}
	}

	export function evalValueToRaw() {
		if (value) {
			rawValue = JSON.stringify(value, null, 4)
		}
	}

	function fileChanged(e: any, cb: (v: string | undefined) => void) {
		let t = e.target
		if (t && 'files' in t && t.files.length > 0) {
			let reader = new FileReader()
			reader.onload = (e: any) => {
				cb(e.target.result.split('base64,')[1])
			}
			reader.readAsDataURL(t.files[0])
		} else {
			cb(undefined)
		}
	}

	export function focus() {
		el?.focus()
	}

	export async function recomputeSize() {
		if (el) {
			el.style.height = '30px'
			el.style.height = el.scrollHeight + 'px'
		}
	}

	function validateInput(pattern: string | undefined, v: any): void {
		if (required && (v == undefined || v == null || v === '')) {
			error = 'This field is required'
			valid = false
		} else {
			if (pattern && !testRegex(pattern, v)) {
				error = `Should match ${pattern}`
				valid = false
			} else {
				error = ''
				valid = true
			}
		}
	}

	function testRegex(pattern: string, value: any): boolean {
		try {
			const regex = new RegExp(pattern)
			return regex.test(value)
		} catch (err) {
			return false
		}
	}

	$: {
		if (value == undefined || value == null) {
			value = defaultValue
			if ((defaultValue === undefined || defaultValue === null) && inputCat === 'string') {
				value = ''
			}
		}
	}

	export let inputCat: InputCat = 'string'
	$: inputCat = computeInputCat(type, format, itemsType?.type, enum_, contentEncoding)
</script>

<div class="flex flex-col w-full mb-2">
	<div>
		{#if displayHeader}
			<FieldHeader {label} {required} {type} {contentEncoding} {format} {itemsType} />
		{/if}
		{#if editableSchema}
			<div class="my-1 text-xs border-solid border border-gray-400 rounded p-2">
				<span
					class="underline"
					on:click={() => {
						seeEditable = !seeEditable
					}}
				>
					Customize argument
					<Icon class="ml-2" data={seeEditable ? faChevronUp : faChevronDown} scale={0.7} />
				</span>

				{#if seeEditable}
					<div class="mt-2">
						<label class="text-gray-700">
							Description
							<textarea rows="1" bind:value={description} placeholder="Edit description" />
							{#if type == 'string' && !contentEncoding && format != 'date-time'}
								<StringTypeNarrowing bind:format bind:pattern bind:enum_ bind:contentEncoding />
							{:else if type == 'object'}
								<ObjectTypeNarrowing bind:format />
							{:else if type == 'array'}
								<select bind:value={itemsType}>
									<option value={undefined}>No specific item type</option>
									<option value={{ type: 'string' }}> Items are strings</option>
									<option value={{ type: 'number' }}>Items are numbers</option>
									<option value={{ type: 'string', contentEncoding: 'base64' }}
										>Items are bytes</option
									>
								</select>
							{/if}
						</label>
					</div>
				{/if}
			</div>
			<span class="text-2xs">Input preview:</span>
		{/if}

		{#if description}
			<div class="text-sm italic pb-1">
				{description}
			</div>
		{/if}

		<div class="flex space-x-1">
			{#if inputCat == 'number'}
				<input
					on:focus
					{disabled}
					type="number"
					class={valid
						? ''
						: 'border border-red-700 border-opacity-30 focus:border-red-700 focus:border-opacity-30 bg-red-100'}
					placeholder={defaultValue ?? ''}
					bind:value
					on:input={() => dispatch('input', { value, isRaw: true })}
				/>
			{:else if inputCat == 'boolean'}
				<input
					{disabled}
					type="checkbox"
					class={valid
						? ''
						: 'border border-red-700 border-opacity-30 focus:border-red-700 focus:border-opacity-30 bg-red-100'}
					bind:checked={value}
				/>
				{#if type == 'boolean' && value == undefined}
					<span>&nbsp; Not set</span>
				{/if}
			{:else if inputCat == 'list'}
				<div>
					<div>
						{#each value ?? [] as v, i}
							<div class="flex flex-row max-w-md mt-1">
								{#if itemsType?.type == 'number'}
									<input type="number" bind:value={v} />
								{:else if itemsType?.type == 'string' && itemsType?.contentEncoding == 'base64'}
									<input
										type="file"
										class="my-6"
										on:change={(x) => fileChanged(x, (val) => (value[i] = val))}
										multiple={false}
									/>
								{:else}
									<input type="text" bind:value={v} />
								{/if}
								<Button
									variant="border"
									color="red"
									size="sm"
									btnClasses="mx-6"
									on:click={() => {
										value = value.filter((el) => el != v)
										if (value.length == 0) {
											value = undefined
										}
									}}
								>
									<Icon data={faMinus} />
								</Button>
							</div>
						{/each}
					</div>
					<Button
						variant="border"
						color="blue"
						size="sm"
						btnClasses="mt-1"
						on:click={() => {
							if (value == undefined) {
								value = []
							}
							value = value.concat('')
						}}
					>
						<Icon data={faPlus} class="mr-2" />
						Add item
					</Button>
					<span class="ml-2">
						{(value ?? []).length} item{(value ?? []).length > 1 ? 's' : ''}
					</span>
				</div>
			{:else if inputCat == 'resource-object'}
				<ObjectResourceInput {format} bind:value />
			{:else if inputCat == 'object'}
				{#if properties && Object.keys(properties).length > 0}
					<div class="p-4 pl-8 border rounded w-full">
						<SchemaForm
							schema={{ properties, $schema: '', required: [], type: 'object' }}
							bind:args={value}
						/>
					</div>
				{:else}
					<textarea
						bind:this={el}
						on:focus
						{disabled}
						style="max-height: {maxHeight}"
						on:input={() => {
							recomputeSize()
							dispatch('input', { rawValue: value, isRaw: false })
						}}
						class="col-span-10 {valid
							? ''
							: 'border border-red-700 border-opacity-30 focus:border-red-700 focus:border-opacity-30 bg-red-100'}"
						placeholder={defaultValue ? JSON.stringify(defaultValue, null, 4) : ''}
						bind:value={rawValue}
					/>
				{/if}
			{:else if inputCat == 'enum'}
				<select {disabled} class="px-6" bind:value>
					{#each enum_ ?? [] as e}
						<option>{e}</option>
					{/each}
				</select>
			{:else if inputCat == 'date'}
				<input class="inline-block" type="datetime-local" bind:value />
			{:else if inputCat == 'sql'}
				<div class="border rounded mb-4 w-full border-gray-700">
					<SimpleEditor
						on:focus={() => dispatch('focus')}
						on:blur={() => dispatch('blur')}
						bind:this={editor}
						lang="sql"
						bind:code={value}
						class="few-lines-editor"
						on:change={async () => {
							dispatch('input', { rawValue: value, isRaw: false })
						}}
					/>
				</div>
			{:else if inputCat == 'base64'}
				<input
					type="file"
					class="my-6"
					on:change={(x) => fileChanged(x, (val) => (value = val))}
					multiple={false}
				/>
			{:else if inputCat == 'resource-string'}
				<ResourcePicker
					bind:value
					resourceType={format.split('-').length > 1
						? format.substring('resource-'.length)
						: undefined}
				/>
			{:else if inputCat == 'string'}
				<textarea
					bind:this={el}
					on:focus={() => dispatch('focus')}
					on:blur={() => dispatch('blur')}
					{disabled}
					style="height: 30px; max-height: {maxHeight}"
					class="col-span-10 {valid
						? ''
						: 'border border-red-700 border-opacity-30 focus:border-red-700 focus:border-opacity-30 bg-red-100'}"
					placeholder={defaultValue ?? ''}
					bind:value
					on:input={() => {
						recomputeSize()
						dispatch('input', { rawValue: value, isRaw: false })
					}}
				/>
			{/if}
			{#if !required && inputCat != 'resource-object'}
				<!-- <Tooltip placement="bottom" content="Reset to default value">
					<Button
						on:click={() => (value = undefined)}
						{disabled}
						color="alternative"
						size="sm"
						class="h-8"
					>
						<Icon data={faArrowRotateLeft} />
					</Button>
				</Tooltip> -->
			{/if}
			<slot name="actions" />
		</div>
		<div class="text-right text-xs {error === '' ? 'text-white' : 'font-bold text-red-600'}">
			{error === '' ? '...' : error}
		</div>
	</div>
</div>
