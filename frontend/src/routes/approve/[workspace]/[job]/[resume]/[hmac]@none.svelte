<script lang="ts">
	import { Job, JobService } from '$lib/gen'
	import { page } from '$app/stores'
	import Button from '$lib/components/common/button/Button.svelte'
	import CenteredModal from '$lib/components/CenteredModal.svelte'
	import { sendUserToast } from '$lib/utils'
	import FlowMetadata from '$lib/components/FlowMetadata.svelte'
	import FlowModulesViewer from '$lib/components/FlowModulesViewer.svelte'
	import JobArgs from '$lib/components/JobArgs.svelte'
	import { onDestroy, onMount } from 'svelte'
	import Tooltip from '$lib/components/Tooltip.svelte'

	let job: Job | undefined = undefined
	let currentApprovers: { resume_id: number; approver: string }[] = []
	let approver = $page.url.searchParams.get('approver') ?? undefined

	let completed: boolean = false
	$: completed = job?.type == 'CompletedJob'

	let timeout: NodeJS.Timer | undefined = undefined

	getJob()

	onMount(() => {
		timeout = setInterval(getJob, 1000)
		window.onunhandledrejection = (event: PromiseRejectionEvent) => {
			event.preventDefault()

			if (event.reason?.message) {
				const { message, body, status } = event.reason

				if (body) {
					sendUserToast(`${body}`, true)
				} else {
					sendUserToast(`${message}`, true)
				}
			} else {
				console.log('Caught unhandled promise rejection without message', event)
			}
		}
	})

	onDestroy(() => {
		timeout && clearInterval(timeout)
	})

	async function getJob() {
		const suspendedJobFlow = await JobService.getSuspendedJobFlow({
			workspace: $page.params.workspace,
			id: $page.params.job,
			resumeId: new Number($page.params.resume).valueOf(),
			signature: $page.params.hmac,
			approver
		})
		job = suspendedJobFlow.job
		currentApprovers = suspendedJobFlow.approvers
	}

	async function resume() {
		await JobService.resumeSuspendedJobPost({
			workspace: $page.params.workspace,
			id: $page.params.job,
			resumeId: new Number($page.params.resume).valueOf(),
			signature: $page.params.hmac,
			approver,
			requestBody: {}
		})
		sendUserToast('Flow approved')
		getJob()
	}

	async function cancel() {
		await JobService.cancelSuspendedJobPost({
			workspace: $page.params.workspace,
			id: $page.params.job,
			resumeId: new Number($page.params.resume).valueOf(),
			signature: $page.params.hmac,
			approver,
			requestBody: {}
		})
		sendUserToast('Flow disapproved!')
		getJob()
	}
</script>

<div class="min-h-screen antialiased text-gray-900">
	<CenteredModal title="Approve resuming of flow?">
		<div class="flex flex-row justify-between flex-wrap sm:flex-nowrap gap-x-4">
			<div class="w-full">
				<h2 class="mt-4">Current approvers</h2>
				<p class="text-xs italic"
					>Each approver can only approve once and cannot change his approver name set by the
					approval sender</p
				>
				<div class="my-4">
					{#if currentApprovers.length > 0}
						<ul>
							{#each currentApprovers as approver}
								<li
									><b
										>{approver.approver}<Tooltip
											>Unique id of approval: {approver.resume_id}</Tooltip
										></b
									></li
								>
							{/each}
						</ul>
					{:else}
						<p class="text-sm"
							>No current approvers for this step (approval steps can require more than one
							approval)</p
						>
					{/if}
				</div>
			</div>
			<div class="w-full">
				{#if job && job.raw_flow}
					<FlowMetadata {job} />
				{/if}
			</div>
		</div>
		<h2 class="mt-4">Flow arguments</h2>

		<JobArgs {job} />
		<div class="mt-8">
			{#if approver}
				<p>Dis/approving as: <b>{approver}</b></p>
			{/if}
		</div>
		{#if completed}
			<div class="my-2"
				><p><b>The flow is not running anymore. You cannot cancel or resume it.</b></p></div
			>
		{/if}

		<div class="w-max-md flex flex-row gap-x-4 gap-y-4 justify-between w-full flex-wrap mt-2">
			<Button btnClasses="grow" color="red" on:click|once={cancel} size="md" disabled={completed}
				>Disapprove/Cancel</Button
			>
			<Button btnClasses="grow" color="green" on:click|once={resume} size="md" disabled={completed}
				>Approve/Resume</Button
			>
		</div>

		<div class="mt-4 flex flex-row flex-wrap justify-between"
			><a href="https://windmill.dev">Learn more about Windmill</a>
			<a target="_blank" href="/run/{job?.id}">Flow run details (require auth)</a>
		</div>
		{#if job && job.raw_flow}
			<h2 class="mt-10">Flow details</h2>
			<FlowModulesViewer
				modules={job.raw_flow?.modules}
				failureModule={job.raw_flow?.failure_module}
			/>
		{/if}
	</CenteredModal>
</div>
