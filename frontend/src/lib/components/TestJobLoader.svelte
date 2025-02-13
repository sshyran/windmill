<script lang="ts">
	import { Job, JobService } from '$lib/gen'
	import { workspaceStore } from '$lib/stores'

	import { onDestroy } from 'svelte'

	import type { Preview } from '$lib/gen/models/Preview'
	import { createEventDispatcher } from 'svelte'

	const dispatch = createEventDispatcher()

	export let isLoading = false
	export let job: Job | undefined = undefined

	let intervalId: NodeJS.Timer

	let syncIteration: number = 0
	let ITERATIONS_BEFORE_SLOW_REFRESH = 100

	export async function runPreview(
		path: string | undefined,
		code: string,
		lang: 'deno' | 'go' | 'python3',
		args: Record<string, any>
	): Promise<void> {
		try {
			intervalId && clearInterval(intervalId)
			if (isLoading && job) {
				JobService.cancelQueuedJob({
					workspace: $workspaceStore!,
					id: job.id,
					requestBody: {}
				})
			}
			isLoading = true

			const testId = await JobService.runScriptPreview({
				workspace: $workspaceStore!,
				requestBody: {
					path,
					content: code,
					args,
					language: lang as Preview.language
				}
			})
			await watchJob(testId)
		} catch (err) {
			isLoading = false
			throw err
		}
	}

	export async function cancelJob() {
		await JobService.cancelQueuedJob({
			workspace: $workspaceStore ?? '',
			id: job?.id ?? '',
			requestBody: {}
		})
		console.log('cancelled')
	}

	export async function watchJob(testId: string) {
		console.log('watch jobs')
		intervalId && clearInterval(intervalId)
		job = undefined
		syncIteration = 0
		const isCompleted = await loadTestJob(testId)
		if (!isCompleted) {
			isLoading = true
			intervalId = setInterval(() => {
				syncer(testId)
			}, 500)
		}
	}

	async function loadTestJob(id: string): Promise<boolean> {
		let isCompleted = false
		try {
			if (job && `running` in job) {
				let previewJobUpdates = await JobService.getJobUpdates({
					workspace: $workspaceStore!,
					id,
					running: job.running,
					logOffset: job.logs?.length ?? 0
				})

				if (previewJobUpdates.new_logs) {
					job.logs = (job.logs ?? '').concat(previewJobUpdates.new_logs)
				}
				if ((previewJobUpdates.running ?? false) || (previewJobUpdates.completed ?? false)) {
					job = await JobService.getJob({ workspace: $workspaceStore!, id })
				}
			} else {
				job = await JobService.getJob({ workspace: $workspaceStore!, id })
			}
			if (job?.type === 'CompletedJob') {
				//only CompletedJob has success property
				isCompleted = true
				clearInterval(intervalId)
				if (isLoading) {
					dispatch('done', job)
					isLoading = false
				}
			}
		} catch (err) {
			console.error(err)
		}
		return isCompleted
	}

	function syncer(id: string): void {
		if (syncIteration == ITERATIONS_BEFORE_SLOW_REFRESH) {
			intervalId && clearInterval(intervalId)
			intervalId = setInterval(() => syncer(id), 2000)
		}
		syncIteration++
		loadTestJob(id)
	}

	onDestroy(() => {
		intervalId && clearInterval(intervalId)
	})
</script>
