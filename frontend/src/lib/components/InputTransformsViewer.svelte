<script lang="ts">
	import type { InputTransform } from '$lib/gen'
	import { Highlight } from 'svelte-highlight'
	import ObjectViewer from './propertyPicker/ObjectViewer.svelte'
	import typescript from 'svelte-highlight/languages/typescript'

	export let inputTransforms: Record<string, InputTransform>
</script>

<ul class="mb-1">
	{#each Object.entries(inputTransforms) as [key, val]}
		<li>
			<span class="font-black text-gray-700">{key}</span>: {#if val.type == 'static'}<ObjectViewer
					json={val.value}
				/>{:else}
				<span class="text-xs">
					<Highlight offsetTop={0} language={typescript} code={val.expr} />
				</span>
			{/if}
		</li>
	{/each}
</ul>
