<script lang="ts">
	import Button from '$lib/components/common/button/Button.svelte'
	import type { IconDefinition } from '@fortawesome/free-brands-svg-icons'
	import Icon from 'svelte-awesome'

	export let disabled: boolean = false
	export let icon: IconDefinition
	export let label: string
	export let iconColor: string
</script>

<Button
	{disabled}
	on:click
	size="sm"
	spacingSize="md"
	variant="border"
	color="light"
	startIcon={{
		icon,
		classes: iconColor
	}}
>
	{label}
</Button>
