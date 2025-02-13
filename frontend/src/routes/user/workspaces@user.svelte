<script context="module">
	export function load() {
		return {
			stuff: { title: 'Workspace Selection' }
		}
	}
</script>

<script lang="ts">
	import { goto } from '$app/navigation'
	import { page } from '$app/stores'
	import { sendUserToast } from '$lib/utils'
	import { logout, logoutWithRedirect } from '$lib/logout'
	import { UserService, type WorkspaceInvite, WorkspaceService } from '$lib/gen'
	import { superadmin, usersWorkspaceStore, workspaceStore } from '$lib/stores'
	import Switch from '$lib/components/Switch.svelte'
	import { faCrown, faUserCog } from '@fortawesome/free-solid-svg-icons'
	import Icon from 'svelte-awesome'
	import { Button } from '$lib/components/common'

	let invites: WorkspaceInvite[] = []
	let list_all_as_super_admin: boolean = false
	let workspaces: { id: string; name: string; username: string }[] = []
	const rd = $page.url.searchParams.get('rd')

	async function loadInvites() {
		try {
			invites = await UserService.listWorkspaceInvites()
		} catch {}
	}

	async function loadWorkspaces() {
		if (!$usersWorkspaceStore) {
			try {
				usersWorkspaceStore.set(await WorkspaceService.listUserWorkspaces())
			} catch {}
		}
		if ($usersWorkspaceStore) {
			if (!$workspaceStore) {
				workspaceStore.set(localStorage.getItem('workspace')?.toString())
			}
		} else {
			logoutWithRedirect($page.url.pathname + $page.url.search)
		}
	}

	async function loadWorkspacesAsAdmin() {
		workspaces = (await WorkspaceService.listWorkspacesAsSuperAdmin({ perPage: 1000 })).map((x) => {
			return { ...x, username: 'superadmin' }
		})
	}

	$: {
		if (list_all_as_super_admin) {
			loadWorkspacesAsAdmin()
		} else {
			workspaces = $usersWorkspaceStore?.workspaces ?? []
		}
	}

	loadInvites()
	loadWorkspaces()
</script>

<div class="center-center min-h-screen p-4">
	<div
		class="border rounded-md shadow-md bg-white w-full max-w-[640px] p-4 sm:py-8 sm:px-10 mb-6 md:mb-20"
	>
		<h1 class="text-center mb-2">Select a workspace</h1>
		<p class="text-center font-medium text-gray-600 text-xs mb-10">
			Logged in as {$usersWorkspaceStore?.email}
		</p>
		<h2 class="mb-4">Workspaces</h2>
		{#if $superadmin}
			<Switch
				textFormat="text-xs"
				label={'List all as superadmin'}
				bind:checked={list_all_as_super_admin}
			/>
			<div class="my-4" />
		{/if}
		{#if workspaces.length == 0}
			<p class="text-sm text-gray-600 mt-2">
				You are not a member of any workspace yet. Accept an invitation or create your own
				workspace.
			</p>
		{/if}
		{#each workspaces as workspace}
			<label class="block pb-2">
				<button
					class="block w-full mx-auto py-1 px-2 rounded-md border border-gray-300 
					shadow-sm text-sm font-normal mt-1 hover:ring-1 hover:ring-indigo-300"
					on:click={() => {
						workspaceStore.set(workspace.id)
						goto(rd ?? '/')
					}}
					><span class="font-mono">{workspace.id}</span> - {workspace.name} as
					<span class="font-mono">{workspace.username}</span>
				</button>
			</label>
		{/each}
		<div class="flex flex-row-reverse  pt-4">
			<a
				href="/user/create_workspace{rd ? `?rd=${encodeURIComponent(rd)}` : ''}"
				class="primary-button"
				>Create a new workspace &rightarrow;
			</a>
		</div>

		<h2 class="mt-6 mb-4">Invitations</h2>
		{#if invites.length == 0}
			<p class="text-sm text-gray-600 mt-2">You have no invites to any workspaces at the moment.</p>
		{/if}
		{#each invites as invite}
			<div
				class="w-full mx-auto py-1 px-2 rounded-md border border-gray-300 shadow-sm 
				text-sm mt-1 flex flex-row justify-between items-center"
			>
				<div class="grow">
					<span class="font-mono font-semibold">{invite.workspace_id}</span>
					{#if invite.is_admin}
						<span class="text-sm">as an admin</span>
					{/if}
				</div>
				<div class="flex justify-end items-center flex-col sm:flex-row gap-1">
					<a
						class="font-bold p-1"
						href="/user/accept_invite?workspace={encodeURIComponent(invite.workspace_id)}{rd
							? `&rd=${encodeURIComponent(rd)}`
							: ''}"
					>
						Accept
					</a>

					<button
						class="text-red-700 font-bold p-1"
						on:click={async () => {
							await UserService.declineInvite({
								requestBody: { workspace_id: invite.workspace_id }
							})
							sendUserToast(`Declined invite to ${invite.workspace_id}`)
							loadInvites()
						}}
					>
						Decline
					</button>
				</div>
			</div>
		{/each}
		<div class="flex justify-between items-center mt-10">
			{#if $superadmin}
				<a class="mr-10" href="/user/superadmin_settings">
					<Icon data={faCrown} class="mr-1" scale={1} />Superadmin settings</a
				>
			{/if}
			<a class="mr-10" href="/user/settings">
				<Icon data={faUserCog} class="mr-1" scale={1} />User settings</a
			>
			<Button
				variant="border"
				color="blue"
				size="sm"
				on:click={async () => {
					logout()
				}}
			>
				Logout
			</Button>
		</div>
	</div>
</div>
