<script lang="ts">
	import Modal from './Modal.svelte'
	import TableCustom from './TableCustom.svelte'

	import { GranularAclService } from '$lib/gen/services/GranularAclService'
	import { sendUserToast } from '$lib/utils'
	import { GroupService, UserService } from '$lib/gen'
	import { createEventDispatcher } from 'svelte'
	import AutoComplete from 'simple-svelte-autocomplete'
	import { workspaceStore } from '$lib/stores'
	import { Button } from './common'

	const dispatch = createEventDispatcher()

	export let kind: 'script' | 'group_' | 'resource' | 'schedule' | 'variable' | 'flow'
	export let path: string = ''

	let ownerKind: 'user' | 'group' = 'user'
	let owner: string = ''

	let newOwner: string = ''
	let write: boolean = false
	let acls: [string, boolean][] = []
	let groups: String[] = []
	let usernames: string[] = []

	let modal: Modal

	$: newOwner = [ownerKind === 'group' ? 'g' : 'u', owner].join('/')

	export async function openModal(newPath?: string) {
		if (newPath) {
			path = newPath
		}
		loadAcls()
		loadGroups()
		loadUsernames()
		modal.openModal()
	}

	async function loadAcls() {
		acls = Object.entries(
			await GranularAclService.getGranularAcls({ workspace: $workspaceStore!, path, kind })
		)
	}

	async function loadGroups(): Promise<void> {
		groups = await GroupService.listGroupNames({ workspace: $workspaceStore! })
	}

	async function loadUsernames(): Promise<void> {
		usernames = await UserService.listUsernames({ workspace: $workspaceStore! })
	}

	async function deleteAcl(owner: string) {
		try {
			await GranularAclService.removeGranularAcls({
				workspace: $workspaceStore!,
				path,
				kind,
				requestBody: { owner }
			})
			loadAcls()
			dispatch('change')
		} catch (err) {
			sendUserToast(err.toString(), true)
		}
	}

	async function addAcl(owner: string, write: boolean) {
		await GranularAclService.addGranularAcls({
			workspace: $workspaceStore!,
			path,
			kind,
			requestBody: { owner, write }
		})
		loadAcls()
		dispatch('change')
	}
</script>

<Modal bind:this={modal}>
	<div slot="title">
		Share {path}
		<span class="text-sm text-gray-500 ml-1">(group)</span>
	</div>

	<div slot="content">
		<div class="flex flex-row flex-wrap pb-0 mb-5 justify-between">
			<div class="flex gap-4 mr-2">
				<label class="block">
					<span class="text-gray-700 text-sm"> Owner Kind </span>
					<select
						class="block mt-1 w-20"
						bind:value={ownerKind}
						on:change={() => {
							if (ownerKind === 'group') {
								owner = 'all'
							} else {
								owner = ''
							}
						}}
					>
						<option>user</option>
						<option>group</option>
					</select>
				</label>
				<label class="block" for="inp">
					<span class="text-sm text-gray-700">Owner</span>
					<div class="block mt-1">
						<div class="static z-50">
							<AutoComplete
								items={ownerKind === 'user' ? usernames : groups}
								bind:selectedItem={owner}
							/>
						</div>
					</div>
				</label>
				<label class="flex flex-col grow pb-3 justify-between">
					<div class="text-sm text-gray-700">Editor</div>
					<input class="block ml-1" type="checkbox" bind:checked={write} />
				</label>
			</div>
			<div class="center-center">
				<Button size="sm" on:click={() => addAcl(newOwner, write)}>Add permission</Button>
			</div>
		</div>
		<TableCustom>
			<tr slot="header-row">
				<th>owner</th>
				<th>has write permission</th>
				<th />
			</tr>
			<tbody slot="body">
				{#each acls as [owner, write]}
					<tr>
						<td>{owner}</td>
						<td>{write}</td>
						<td>
							<Button variant="border" color="red" size="sm" on:click={() => deleteAcl(owner)}>
								Delete
							</Button>
						</td>
					</tr>
				{/each}
			</tbody>
		</TableCustom>
	</div>
</Modal>
