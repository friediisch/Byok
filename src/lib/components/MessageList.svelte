<script lang="ts">
	import type { Message, Model } from '../../../bindings'
	import MessageItem from './MessageItem.svelte'
	
	export let messages: Message[] = []
	export let selectedModel: Model
	export let isNewChat: boolean = false
	export let selectedModelName: string = ''
	export let cmdHeld: boolean = false
	
	let messagesContainer: HTMLElement
	let messageElements: HTMLElement[] = []
	
	// Reset message elements when messages change
	$: if (messages) {
		messageElements = []
	}
	
	export async function scrollToBottom() {
		await new Promise((resolve) => setTimeout(resolve))
		if (messagesContainer) {
			messagesContainer.scrollTop = messagesContainer.scrollHeight
		}
	}
	
	export function scrollToPreviousMessage() {
		if (messages.length === 0 || isNewChat || !messagesContainer) return
		
		const containerRect = messagesContainer.getBoundingClientRect()
		const threshold = 5 // pixels tolerance for "aligned with top"
		
		// Find the message whose top is just above the container's visible top
		for (let i = messageElements.length - 1; i >= 0; i--) {
			const el = messageElements[i]
			if (!el) continue
			
			const rect = el.getBoundingClientRect()
			const topRelativeToContainer = rect.top - containerRect.top
			
			// Find message whose top is above the viewport (negative relative position)
			if (topRelativeToContainer < -threshold) {
				el.scrollIntoView({ behavior: 'instant', block: 'start' })
				return
			}
		}
		
		// If no message found above, scroll to the very top
		messagesContainer.scrollTo({ top: 0, behavior: 'instant' })
	}
	
	export function scrollToNextMessage() {
		if (messages.length === 0 || isNewChat || !messagesContainer) return
		
		const containerRect = messagesContainer.getBoundingClientRect()
		const threshold = 5 // pixels tolerance for "aligned with top"
		
		// Find the first message whose top is below the container's top (not aligned)
		for (let i = 0; i < messageElements.length; i++) {
			const el = messageElements[i]
			if (!el) continue
			
			const rect = el.getBoundingClientRect()
			const topRelativeToContainer = rect.top - containerRect.top
			
			// Find message whose top is below the viewport top (positive relative position beyond threshold)
			if (topRelativeToContainer > threshold) {
				el.scrollIntoView({ behavior: 'instant', block: 'start' })
				return
			}
		}
		
		// If no message found below, scroll to the bottom
		messagesContainer.scrollTo({ top: messagesContainer.scrollHeight, behavior: 'instant' })
	}
</script>

<div
	class="relative flex flex-col flex-1 min-w-[12rem] w-[56rem] max-w-[56rem] overflow-y-auto overscroll-contain px-2"
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
		<div class="flex flex-col">
			<div class="p-2"></div>
			{#each messages as message, i}
				<div bind:this={messageElements[i]} class="grid grid-cols-[auto_minmax(0,1fr)] gap-x-1">
					<MessageItem {message} {selectedModel} />
				</div>
			{/each}
		</div>
	{/if}
	
	<!-- Navigation indicators when Cmd is held -->
	{#if cmdHeld && !isNewChat && messages.length > 0}
		<div class="fixed bottom-24 right-8 flex flex-col gap-1 z-50">
			<div class="bg-gray-500 text-white text-xs font-mono px-2 py-1 rounded flex items-center gap-1">
				<span>↑</span>
			</div>
			<div class="bg-gray-500 text-white text-xs font-mono px-2 py-1 rounded flex items-center gap-1">
				<span>↓</span>
			</div>
		</div>
	{/if}
</div>
