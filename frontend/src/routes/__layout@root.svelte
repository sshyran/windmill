<script lang="ts">
	import { faArrowLeft, faSpinner } from '@fortawesome/free-solid-svg-icons'
	import Icon from 'svelte-awesome'

	import UserMenu from '$lib/components/sidebar/UserMenu.svelte'
	import { OpenAPI } from '$lib/gen'
	import { classNames } from '$lib/utils'

	import WorkspaceMenu from '$lib/components/sidebar/WorkspaceMenu.svelte'
	import SidebarContent from '$lib/components/sidebar/SidebarContent.svelte'
	import '../app.css'
	import { userStore } from '$lib/stores'
	import CenteredModal from '$lib/components/CenteredModal.svelte'
	import { beforeNavigate } from '$app/navigation'

	OpenAPI.WITH_CREDENTIALS = true

	let menuOpen = false
	let isCollapsed = false

	beforeNavigate((newNavigationState) => {
		menuOpen = false
	})

	let innerWidth = window.innerWidth

	$: innerWidth < 1248 && innerWidth > 768 && (isCollapsed = true)
	$: (innerWidth >= 1248 || innerWidth < 768) && (isCollapsed = false)
</script>

<svelte:window bind:innerWidth />

{#if $userStore}
	<div>
		<div
			class={classNames('relative  md:hidden 	', menuOpen ? 'z-40' : 'pointer-events-none')}
			role="dialog"
			aria-modal="true"
		>
			<div
				class={classNames(
					'fixed inset-0 bg-gray-600 bg-opacity-75 transition-opacity ease-linear duration-300',
					menuOpen ? 'opacity-100' : 'opacity-0 '
				)}
			/>

			<div class="fixed inset-0 flex">
				<div
					class={classNames(
						'relative flex-1 flex flex-col max-w-xs w-full bg-white transition ease-in-out duration-300 transform',
						menuOpen ? 'translate-x-0' : '-translate-x-full'
					)}
				>
					<div
						class={classNames(
							'absolute top-0 right-0 -mr-12 pt-2 ease-in-out duration-300',
							menuOpen ? 'opacity-100' : 'opacity-0'
						)}
					>
						<button
							type="button"
							on:click={() => {
								menuOpen = !menuOpen
							}}
							class="ml-1 flex items-center justify-center h-10 w-10 rounded-full focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white"
						>
							<svg
								class="h-6 w-6 text-white"
								xmlns="http://www.w3.org/2000/svg"
								fill="none"
								viewBox="0 0 24 24"
								stroke-width="2"
								stroke="currentColor"
								aria-hidden="true"
							>
								<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
							</svg>
						</button>
					</div>
					<div class="bg-blue-500 h-full">
						<div class="flex items-center flex-shrink-0 p-4 font-extrabold text-white">
							Windmill
						</div>

						<div class="px-2 py-4 space-y-2 border-y border-blue-400">
							<WorkspaceMenu />
							<UserMenu />
						</div>

						<SidebarContent {isCollapsed} />
					</div>
				</div>
			</div>
		</div>

		<div
			class={classNames(
				'hidden md:flex md:flex-col md:fixed md:inset-y-0 transition-all ease-in-out duration-200 shadow-md z-10',
				isCollapsed ? 'md:w-12' : 'md:w-48'
			)}
		>
			<div class="flex-1 flex flex-col min-h-0 shadow-lg bg-blue-500">
				<div class="flex items-center flex-shrink-0 p-4 font-extrabold text-white">
					{#if isCollapsed}
						W
					{:else}
						Windmill
					{/if}
				</div>

				<div class="px-2 py-4 space-y-2 border-y border-blue-400">
					<WorkspaceMenu {isCollapsed} />
					<UserMenu {isCollapsed} />
				</div>
				<SidebarContent {isCollapsed} />

				<div class="flex-shrink-0 flex p-4 border-t border-blue-400">
					<button
						on:click={() => {
							isCollapsed = !isCollapsed
						}}
					>
						<Icon
							data={faArrowLeft}
							class={classNames(
								'flex-shrink-0 h-4 w-4 transition-all ease-in-out duration-200 text-white',
								isCollapsed ? 'rotate-180' : 'rotate-0'
							)}
						/>
					</button>
				</div>
			</div>
		</div>
		<div class={classNames('flex flex-col flex-1', isCollapsed ? 'md:pl-12' : 'md:pl-48')}>
			<main>
				<div class="w-full h-screen overflow-auto">
					<div
						class="py-2 px-2 sm:px-4 md:px-8 flex justify-between items-center shadow-sm max-w-6xl mx-auto md:hidden"
					>
						<button
							type="button"
							on:click={() => {
								menuOpen = true
							}}
							class="h-8 w-8 inline-flex items-center justify-center rounded-md text-gray-500 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-indigo-500"
						>
							<svg
								class="h-6 w-6"
								xmlns="http://www.w3.org/2000/svg"
								fill="none"
								viewBox="0 0 24 24"
								stroke-width="2"
								stroke="currentColor"
								aria-hidden="true"
							>
								<path stroke-linecap="round" stroke-linejoin="round" d="M4 6h16M4 12h16M4 18h16" />
							</svg>
						</button>
					</div>
					<slot />
				</div>
			</main>
		</div>
	</div>
{:else}
	<CenteredModal title="Loading user">
		<div class="mx-auto w-0">
			<Icon class="animate-spin" data={faSpinner} scale={2.0} />
		</div>
	</CenteredModal>
{/if}
