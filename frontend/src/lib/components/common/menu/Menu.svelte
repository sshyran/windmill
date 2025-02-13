<script lang="ts">
	import { classNames } from '$lib/utils'
	import { onMount } from 'svelte'
	import { scale } from 'svelte/transition'

	let show = false
	let menu: HTMLDivElement

	type Alignment = 'start' | 'end'
	type Side = 'top' | 'bottom'
	type Placement = `${Side}-${Alignment}`

	export let placement: Placement = 'bottom-start'

	function handleOutsideClick(event) {
		if (show && !menu.contains(event.target)) {
			show = false
		}
	}

	function handleEscape(event) {
		if (show && event.key === 'Escape') {
			show = false
		}
	}

	function close() {
		show = false
	}

	onMount(() => {
		document.addEventListener('click', handleOutsideClick, false)
		document.addEventListener('keyup', handleEscape, false)

		return () => {
			document.removeEventListener('click', handleOutsideClick, false)
			document.removeEventListener('keyup', handleEscape, false)
		}
	})

	const placementsClasses = {
		'bottom-start': 'origin-top-left left-0',
		'bottom-end': 'origin-top-right right-0',
		'top-start': 'origin-bottom-left left-0 bottom-0',
		'top-end': 'origin-bottom-right right-0 bottom-0'
	}
</script>

<div class="relative" bind:this={menu}>
	<div on:click={() => (show = !show)} on:click>
		<slot name="trigger" />
	</div>

	{#if show}
		<div
			class={classNames(
				'z-50 absolute mt-2 w-60 rounded-md shadow-lg bg-white ring-1 ring-black ring-opacity-5 focus:outline-none',
				placementsClasses[placement]
			)}
			role="menu"
			tabindex="-1"
		>
			<slot {close} />
		</div>
	{/if}
</div>
