<script lang="ts">
	import Required from './Required.svelte'

	export let label: string
	export let format: string = ''
	export let contentEncoding = ''
	export let type: string | undefined = undefined
	export let required = false
	export let itemsType:
		| { type?: 'string' | 'number' | 'bytes'; contentEncoding?: 'base64' }
		| undefined = undefined
</script>

<div>
	<span class="font-semibold">
		{label}
	</span>
	<span class="text-sm italic text-indigo-800">
		({type ?? 'any'}{contentEncoding && contentEncoding != ''
			? `, encoding: ${contentEncoding}`
			: ''}{format && format != '' ? `, format: ${format}` : ''}{itemsType?.type
			? ` of ${itemsType?.type}s`
			: ''})</span
	>
	<Required {required} class="!ml-0" />
</div>
