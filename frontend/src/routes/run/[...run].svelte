<script context="module">
	export function load({ params }) {
		return {
			stuff: { title: `Run ${params.run}` }
		}
	}
</script>

<script lang="ts">
	import { page } from '$app/stores'
	import { JobService, Job } from '$lib/gen'
	import { canWrite, encodeState, forLater, sendUserToast, truncateHash } from '$lib/utils'
	import Icon from 'svelte-awesome'
	import { check } from 'svelte-awesome/icons'
	import {
		faRefresh,
		faCircle,
		faTimes,
		faTrash,
		faCalendar,
		faTimesCircle,
		faList,
		faEdit,
		faHourglassHalf,
		faScroll,
		faFastForward
	} from '@fortawesome/free-solid-svg-icons'
	import Tooltip from '$lib/components/Tooltip.svelte'
	import DisplayResult from '$lib/components/DisplayResult.svelte'
	import { userStore, workspaceStore } from '$lib/stores'
	import CenteredPage from '$lib/components/CenteredPage.svelte'
	import FlowStatusViewer from '$lib/components/FlowStatusViewer.svelte'
	import HighlightCode from '$lib/components/HighlightCode.svelte'
	import TestJobLoader from '$lib/components/TestJobLoader.svelte'
	import LogViewer from '$lib/components/LogViewer.svelte'
	import { Button, ActionRow, Skeleton } from '$lib/components/common'
	import FlowMetadata from '$lib/components/FlowMetadata.svelte'
	import JobArgs from '$lib/components/JobArgs.svelte'
	import FlowProgressBar from '$lib/components/flows/FlowProgressBar.svelte'

	let workspace_id_query: string | undefined = $page.url.searchParams.get('workspace') ?? undefined
	let workspace_id: string | undefined

	let job: Job | undefined
	const iconScale = 1

	let viewTab: 'result' | 'logs' | 'code' = 'result'

	// Test
	let testIsLoading = false

	let testJobLoader: TestJobLoader

	const SMALL_ICON_SCALE = 0.7

	async function deleteCompletedJob(id: string): Promise<void> {
		await JobService.deleteCompletedJob({ workspace: workspace_id!, id })
		getLogs()
	}

	async function cancelJob(id: string) {
		try {
			await JobService.cancelQueuedJob({ workspace: workspace_id!, id, requestBody: {} })
			sendUserToast(`job ${id} canceled`)
		} catch (err) {
			sendUserToast('could not cancel job', true)
		}
	}

	// If we get results, focus on that tab. Else, focus on logs
	function initView(): void {
		if (job && 'result' in job && job.result !== undefined) {
			viewTab = 'result'
		} else if (viewTab == 'result') {
			viewTab = 'logs'
		}
	}

	async function getLogs() {
		await testJobLoader?.watchJob($page.params.run)
		initView()
	}

	$: {
		if ($workspaceStore && $page.params.run && testJobLoader) {
			workspace_id = workspace_id_query ?? $workspaceStore
			getLogs()
		}
	}
</script>

<TestJobLoader
	on:done={() => (viewTab = 'result')}
	bind:this={testJobLoader}
	bind:isLoading={testIsLoading}
	bind:job
/>

<Skeleton
	class="!max-w-6xl !px-4 sm:!px-6 md:!px-8"
	loading={!job}
	layout={[0.75, [2, 0, 2], 2.25, [{ h: 1.5, w: 40 }]]}
/>
{#if job?.job_kind === 'script' || job?.job_kind === 'flow'}
	<ActionRow applyPageWidth stickToTop>
		<svelte:fragment slot="left">
			{@const stem = `/${job?.job_kind}s`}
			{@const isScript = job?.job_kind === 'script'}
			{@const route = isScript ? job?.script_hash : job?.script_path}
			{@const runHref = `${stem}/run/${route}${
				job?.args ? '?args=' + encodeURIComponent(encodeState(job?.args)) : ''
			}`}
			{@const editHref = `${stem}/edit/${route}${isScript ? '?step=2' : ''}`}
			{@const isRunning = job && 'running' in job && job.running}
			{@const runsHref = `/runs/${job?.script_path}${!isScript ? '?jobKind=flow' : ''}`}
			{@const viewHref = `${stem}/get/${isScript ? job?.script_hash : job?.script_path}`}
			{#if isRunning}
				<Button
					color="red"
					size="xs"
					startIcon={{ icon: faTimesCircle }}
					on:click|once={() => {
						if (job?.id) {
							cancelJob(job?.id)
						}
					}}
				>
					Cancel
				</Button>
			{/if}
			<Button
				href={runHref}
				disabled={isRunning}
				color="blue"
				size="xs"
				startIcon={{ icon: faRefresh }}>Run again</Button
			>
			{#if canWrite(job?.script_path ?? '', {}, $userStore)}
				<Button href={editHref} color="blue" size="xs" startIcon={{ icon: faEdit }}>Edit</Button>
			{/if}
			<Button href={viewHref} color="blue" size="xs" startIcon={{ icon: faScroll }}>
				View {job?.job_kind}
			</Button>
			<Button href={runsHref} variant="border" color="blue" size="xs" startIcon={{ icon: faList }}>
				View runs
			</Button>
		</svelte:fragment>
		<svelte:fragment slot="right">
			{#if job && 'deleted' in job && !job?.deleted && ($userStore?.is_admin ?? false)}
				<Button
					variant="border"
					color="red"
					size="xs"
					startIcon={{ icon: faTrash }}
					on:click={() => job?.id && deleteCompletedJob(job.id)}
				>
					Delete
				</Button>
			{/if}
		</svelte:fragment>
	</ActionRow>
{/if}
<CenteredPage>
	<h1 class="flex flex-row flex-wrap justify-between items-center gap-4 py-6">
		<div>
			{#if job}
				{#if 'success' in job && job.success}
					{#if job.is_skipped}
						<Icon
							class="text-green-600"
							data={faFastForward}
							scale={SMALL_ICON_SCALE}
							label="Job completed successfully but was skipped"
						/>
					{:else}
						<Icon
							class="text-green-600"
							data={check}
							scale={SMALL_ICON_SCALE}
							label="Job completed successfully"
						/>
					{/if}
				{:else if job && 'success' in job}
					<Icon
						class="text-red-700"
						data={faTimes}
						scale={iconScale}
						label="Job completed with an error"
					/>
				{:else if job && 'running' in job && job.running}
					<Icon class="text-yellow-500" data={faCircle} scale={iconScale} label="Job is running" />
				{:else if job && 'running' in job && job.scheduled_for && forLater(job.scheduled_for)}
					<Icon
						class="text-gray-700"
						data={faCalendar}
						scale={iconScale}
						label="Job is scheduled for a later time"
					/>
				{:else if job && 'running' in job && job.scheduled_for}
					<Icon
						class="text-gray-500"
						data={faHourglassHalf}
						scale={iconScale}
						label="Job is waiting for an executor"
					/>
				{/if}
				{job.script_path ?? (job.job_kind == 'dependencies' ? 'lock dependencies' : 'No path')}
				{#if job.script_hash}
					<a
						href="/scripts/get/{job.script_hash}"
						class="text-2xs text-gray-500 bg-gray-100 font-mono">{truncateHash(job.script_hash)}</a
					>
				{/if}
				{#if job && 'job_kind' in job}<span
						class="bg-blue-200 text-gray-700 text-xs rounded px-1 mx-3">{job.job_kind}</span
					>
				{/if}
			{/if}
		</div>
	</h1>
	{#if job && 'deleted' in job && job?.deleted}
		<div class="bg-red-100 border-l-4 border-red-600 text-orange-700 p-4" role="alert">
			<p class="font-bold">Deleted</p>
			<p>The content of this run was deleted (by an admin, no less)</p>
		</div>
	{/if}

	<!-- Arguments and actions -->
	<div class="flex flex-col mr-2 sm:mr-0 sm:grid sm:grid-cols-3 sm:gap-5">
		<div class="col-span-2">
			<JobArgs {job} />

			{#if job?.job_kind == 'flow' || job?.job_kind == 'flowpreview'}
				<div class="mt-10" />
				<FlowProgressBar {job} class="py-4" />
				<div class="max-w-lg">
					<FlowStatusViewer
						jobId={job.id}
						on:jobsLoaded={({ detail }) => {
							job = detail
						}}
					/>
				</div>
			{/if}
		</div>
		<div>
			<Skeleton loading={!job} layout={[[9.5]]} />
			{#if job}<FlowMetadata {job} />{/if}
		</div>
	</div>

	{#if job?.job_kind !== 'flow' && job?.job_kind !== 'flowpreview'}
		<!-- Logs and outputs-->
		<div class="mr-2 sm:mr-0 mt-12">
			<div class="flex flex-col sm:flex-row text-base">
				<button
					class=" py-1 px-6 block border-gray-200 hover:bg-gray-50  {viewTab !== 'result'
						? 'text-gray-500'
						: 'text-gray-700 font-semibold  '}"
					on:click={() => (viewTab = 'result')}
				>
					Result <Tooltip
						>What is returned by the <span class="font-mono">main</span> function of the script,
						stringified to JSON. Then for some specific cases, like having "png", "jpeg" or "file"
						as sole key, they are displayed more richly. See
						<a href="https://docs.windmill.dev/docs/reference#rich-display-rendering">here</a> for more
						details.</Tooltip
					>
				</button>
				<button
					class="py-1 px-6 block border-gray-200 hover:bg-gray-50  {viewTab !== 'logs'
						? 'text-gray-500'
						: 'text-gray-700 font-semibold  '}"
					on:click={() => (viewTab = 'logs')}
				>
					Logs
				</button>
				{#if job && 'raw_code' in job && job.raw_code}
					<button
						class="py-1 px-6 block border-gray-200 hover:bg-gray-50  {viewTab !== 'code'
							? 'text-gray-500'
							: 'text-gray-700 font-semibold  '}"
						on:click={() => (viewTab = 'code')}
					>
						{job.job_kind == 'dependencies' ? 'Input Dependencies' : 'Code previewed'}
					</button>
				{/if}
			</div>
			<Skeleton loading={!job} layout={[[5]]} />
			{#if job}
				<div class="flex flex-row border rounded-md p-3 max-h-1/2 overflow-auto">
					{#if viewTab == 'logs'}
						<div class="w-full">
							<LogViewer isLoading={!(job && 'logs' in job && job.logs)} content={job?.logs} />
						</div>
					{:else if viewTab == 'code'}
						{#if job && 'raw_code' in job && job.raw_code}
							<HighlightCode language={job.language} code={job.raw_code} />
						{:else if job}
							No code is available
						{:else}
							<Skeleton layout={[[5]]} />
						{/if}
					{:else if job !== undefined && 'result' in job && job.result !== undefined}
						<DisplayResult result={job.result} />
					{:else if job}
						No output is available yet
					{/if}
				</div>
			{/if}
		</div>
	{/if}
</CenteredPage>
