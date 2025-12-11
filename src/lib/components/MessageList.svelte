<script lang="ts">
	import type { Message, Model } from '../../../bindings'
	import MessageItem from './MessageItem.svelte'
	
	export let messages: Message[] = []
	export let selectedModel: Model
	export let isNewChat: boolean = false
	export let selectedModelName: string = ''
	
	let messagesContainer: HTMLElement
	
	export async function scrollToBottom() {
		await new Promise((resolve) => setTimeout(resolve))
		if (messagesContainer) {
			messagesContainer.scrollTop = messagesContainer.scrollHeight
		}
	}
</script>

<div
	class="flex flex-col flex-1 min-w-[12rem] w-[56rem] max-w-[56rem] overflow-y-auto overscroll-contain px-2"
	bind:this={messagesContainer}
>
	{#if isNewChat}
		<div class="flex flex-col items-center justify-center h-[66.67vh]">
			<div class="text-center text-3xl text-gradient animate-fly-and-fade">
				How can I help you today?
			</div>
			<div class="text-center text-md text-gray-500 animate-fly-and-fade">
				{selectedModelName}
			</div>
		</div>
	{:else}
		<div class="grid grid-cols-[auto_minmax(0,1fr)] gap-x-1">
			<div class="p-2"></div>
			<div class="p-2"></div>
			{#each messages as message}
				<MessageItem {message} {selectedModel} />
			{/each}
		</div>
	{/if}
</div>
