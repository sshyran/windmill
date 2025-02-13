<script lang="ts">
	import { faInfoCircle } from '@fortawesome/free-solid-svg-icons'
	import Icon from 'svelte-awesome'
	import { createPopperActions } from 'svelte-popperjs'
	const [popperRef, popperContent] = createPopperActions({
		placement: 'auto'
	})
	const betterPreventOverflow = (options) => ({
		name: 'preventOverflow',
		options,
		effect: ({ state }) => {
			const { padding = 0 } = options

			state.elements.popper.style.maxWidth = `calc(100vw - ${padding * 2}px)`
		}
	})
	const extraOpts = {
		modifiers: [
			betterPreventOverflow({ padding: 10 }),
			{ name: 'offset', options: { offset: [8, 8] } },
			{
				name: 'arrow',
				options: {
					padding: 10 // 5px from the edges of the popper
				}
			}
		]
	}

	let showTooltip = false
	let timeout: NodeJS.Timeout

	function open() {
		clearTimeout(timeout)
		showTooltip = true
	}
	function close() {
		timeout = setTimeout(() => (showTooltip = false), 100)
	}
</script>

<button use:popperRef on:mouseenter={open} on:mouseleave={close}>
	<Icon
		class="text-gray-500 font-thin inline-block align-middle w-4"
		data={faInfoCircle}
		scale={0.8}
	/>
</button>
{#if showTooltip}
	<div
		id="tooltip"
		use:popperContent={extraOpts}
		on:mouseenter={open}
		on:mouseleave={close}
		class="w-96 text-left"
	>
		<slot />
	</div>
{/if}

<style>
	#tooltip {
		@apply z-50 text-base font-normal text-gray-300 bg-gray-800 p-4 rounded-xl whitespace-normal;
	}
</style>
