<script lang="ts">
	import type { Message, Model } from '../../../bindings'
	import { availableModelsStore } from '$lib/stores'
	
	export let message: Message
	export let selectedModel: Model
	
	function getModelDisplayName(modelName: string): string {
		const model = $availableModelsStore.find((m) => m.model_name === modelName)
		return model?.model_display_name || selectedModel?.model_display_name || modelName
	}
	
	function getModelName(modelName: string): string {
		const model = $availableModelsStore.find((m) => m.model_name === modelName)
		return model?.model_name || selectedModel?.model_name || modelName
	}
	
	async function copyToClipboard(content: string, block: any) {
		try {
			await navigator.clipboard.writeText(content)
			block.copied = true
			setTimeout(() => {
				block.copied = false
			}, 2000)
		} catch (err) {
			console.error('Failed to copy: ', err)
		}
	}
</script>

{#if message.role === 'user'}
	<div class="font-bold p-1 whitespace-nowrap">
		<div>You</div>
	</div>
	<div class="p-1 whitespace-pre-wrap word-break:break-word overflow-wrap:break-word">
		{message.content}
	</div>
{:else}
	<div class="relative p-1 min-w-fit h-fit whitespace-nowrap group">
		<div id="display_name_{message.id}" class="font-bold text-gradient rounded-md relative">
			{getModelDisplayName(message.model_name)}
		</div>
		<div
			id="model_name_{message.id}"
			class="absolute top-1 left-0 opacity-0 transition-opacity duration-150 ease-in-out z-10 inset-0 w-fit pointer-events-none model-tooltip"
		>
			<div class="bg-white text-gray-800 px-1 rounded-md">
				{getModelName(message.model_name)}
			</div>
		</div>
	</div>
	<div class="p-1">
		{#if message.role === 'animate'}
			<div
				class="mt-1 animate-ping rounded-full self-center self-middle size-4 bg-white opacity-100"
			></div>
		{:else if message.blocks}
			{#each message.blocks as block}
				<div class="pb-2">
					{#if block.type_ === 'code'}
						<div class="relative group">
							{#if block.language}
								<div class="bg-gray2 text-gray-300 text-xs font-mono px-3 py-3 rounded-t-md">
									{block.language}
								</div>
							{/if}
							<div class="text-white text-xs font-mono whitespace-pre-wrap overflow-x-scroll">
								{@html block.rendered_content}
							</div>
							<button
								on:mousedown={() => copyToClipboard(block.raw_content, block)}
								class="absolute right-2 top-2 flex items-center justify-center w-6 h-6 bg-gray2 text-gray-300 rounded hover:bg-gray-500 cursor-pointer"
								title="Copy code"
							>
								{#if block.copied}
									<span class="icon-[lucide--check-check]" style="color: white;"></span>
								{:else}
									<span class="icon-[lucide--clipboard]" style="color: white;"></span>
								{/if}
							</button>
						</div>
					{:else if block.type_ === 'text'}
						<div class="word-break:break-word overflow-wrap:break-word">
							{@html block.rendered_content}
						</div>
					{/if}
				</div>
			{/each}
		{/if}
	</div>
{/if}

<style>
	.group:hover .model-tooltip {
		opacity: 1;
		z-index: 10;
	}
</style>
