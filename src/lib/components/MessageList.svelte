<script lang="ts">
	import type { Message, Model } from '../../../bindings'
	import MessageItem from './MessageItem.svelte'
	
	export let messages: Message[] = []
	export let selectedModel: Model
	export let isNewChat: boolean = false
	export let selectedModelName: string = ''
	export let cmdHeld: boolean = false
	
	let messagesContainer: HTMLElement
	
	export async function scrollToBottom() {
		await new Promise((resolve) => setTimeout(resolve))
		if (messagesContainer) {
			messagesContainer.scrollTop = messagesContainer.scrollHeight
		}
	}
	
	function getMessageElements(): HTMLElement[] {
		if (!messagesContainer) return []
		return Array.from(messagesContainer.querySelectorAll('[data-message]'))
	}

	export function scrollToPreviousMessage() {
		if (messages.length === 0 || isNewChat || !messagesContainer) return
		
		const containerRect = messagesContainer.getBoundingClientRect()
		const threshold = 5
		const messageElements = getMessageElements()
		
		// Find the last message whose top is above the viewport top (scrolled off)
		for (let i = messageElements.length - 1; i >= 0; i--) {
			const el = messageElements[i]
			const rect = el.getBoundingClientRect()
			const topRelative = rect.top - containerRect.top
			
			// This message's top is above the viewport - scroll to it
			if (topRelative < -threshold) {
				el.scrollIntoView({ behavior: 'smooth', block: 'start' })
				return
			}
		}
		
		// No message above, scroll to very top
		messagesContainer.scrollTo({ top: 0, behavior: 'smooth' })
	}
	
	export function scrollToNextMessage() {
		if (messages.length === 0 || isNewChat || !messagesContainer) return
		
		const containerRect = messagesContainer.getBoundingClientRect()
		const threshold = 5
		const messageElements = getMessageElements()
		
		// Find the first message whose top is below the viewport top (not yet scrolled to)
		for (let i = 0; i < messageElements.length; i++) {
			const el = messageElements[i]
			const rect = el.getBoundingClientRect()
			const topRelative = rect.top - containerRect.top
			
			// This message's top is below the viewport top - scroll to it
			if (topRelative > threshold) {
				el.scrollIntoView({ behavior: 'smooth', block: 'start' })
				return
			}
		}
		
		// No message below, scroll to very bottom
		messagesContainer.scrollTo({ top: messagesContainer.scrollHeight, behavior: 'smooth' })
	}
</script>

<div class="relative flex flex-col flex-1 min-w-[12rem] w-[56rem] max-w-[56rem] overflow-hidden">
	<!-- Navigation indicators when Cmd is held -->
	{#if cmdHeld && !isNewChat && messages.length > 0}
		<!-- Up arrow at center top -->
		<div class="absolute top-2 left-1/2 -translate-x-1/2 pointer-events-none z-50">
			<div class="bg-gray-500 text-white text-xs font-mono px-2 py-1 rounded">
				↑
			</div>
		</div>
		<!-- Down arrow at center bottom -->
		<div class="absolute bottom-2 left-1/2 -translate-x-1/2 pointer-events-none z-50">
			<div class="bg-gray-500 text-white text-xs font-mono px-2 py-1 rounded">
				↓
			</div>
		</div>
	{/if}
	
	<div
		class="flex flex-col flex-1 overflow-y-auto overscroll-contain px-2"
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
					<div data-message={i} class="grid grid-cols-[auto_minmax(0,1fr)] gap-x-1">
						<MessageItem {message} {selectedModel} />
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>
