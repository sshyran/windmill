<script lang="ts">
	import { goto } from '$app/navigation'
	import type { Script } from '$lib/gen'

	import { truncateHash } from '$lib/utils'
	import { faPencil, faPlay } from '@fortawesome/free-solid-svg-icons'
	import { Button, Badge, ActionRow } from '$lib/components/common'

	export let script: Script
</script>

<a
	class="border p-4 rounded-sm shadow-sm space-y-2 hover:border-blue-600 text-gray-800 flex flex-col justify-between"
	href="/scripts/get/{script.hash}"
>
	<div class="font-bold">{script.summary || script.path}</div>

	<div class="inline-flex justify-between w-full">
		<div class="text-xs">{script.path}</div>

		<Badge color="gray">
			{truncateHash(script.hash)}
		</Badge>
	</div>
	<div class="inline-flex space-x-1 w-full">
		<Badge color="blue" capitalize>
			{script.language}
		</Badge>
		{#if script.kind !== 'script'}
			<Badge color="green" capitalize>
				{script.kind}
			</Badge>
		{/if}
	</div>
	<div class="flex flex-row-reverse gap-x-2">
		<Button
			on:click={() => goto(`/scripts/edit/${script.hash}?step=2`)}
			color="dark"
			size="xs"
			variant="border"
			startIcon={{ icon: faPencil }}
		>
			Edit
		</Button>

		<Button
			on:click={() => goto(`/scripts/run/${script.hash}`)}
			color="dark"
			size="xs"
			variant="border"
			startIcon={{ icon: faPlay }}
		>
			Run
		</Button>
	</div>
</a>
