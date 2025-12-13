<script lang="ts">
	import type { Chats } from '../../../bindings'
	import Icon from '@iconify/svelte'
	import ChatItem from './ChatItem.svelte'
	
	export let chats: Chats = []
	export let selectedChatId: string = ''
	export let newChatId: string = ''
	export let cmdHeld: boolean = false
	export let onNewChat: () => void
	export let onSelectChat: (chatId: string) => void
	export let onOpenSettings: () => void
	export let onChatsUpdated: () => void
	
	let sidebarWidth = 288 // 18rem default (min-w-72)
	let isResizing = false
	
	const MIN_WIDTH = 200
	const MAX_WIDTH = 400
	
	function startResize(e: MouseEvent) {
		isResizing = true
		document.addEventListener('mousemove', handleResize)
		document.addEventListener('mouseup', stopResize)
		e.preventDefault()
	}
	
	function handleResize(e: MouseEvent) {
		if (!isResizing) return
		const newWidth = e.clientX
		sidebarWidth = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, newWidth))
	}
	
	function stopResize() {
		isResizing = false
		document.removeEventListener('mousemove', handleResize)
		document.removeEventListener('mouseup', stopResize)
	}
</script>

<div
	class="relative flex flex-col bg-sidebar-gray overflow-y-auto overscroll-contain h-screen px-4 pt-6"
	style="width: {sidebarWidth}px;"
>
	<!-- Resize handle -->
	<div
		class="absolute top-0 right-0 w-1 h-full cursor-col-resize hover:bg-gray-500 transition-colors"
		on:mousedown={startResize}
		role="separator"
		aria-orientation="vertical"
		tabindex="0"
	></div>
	<div class="text-3xl pl-2 font-bold">Byok</div>
	<hr class="my-4" />
	<div class="overflow-y-auto flex-1">
		<!-- New Chat Button -->
		<div
			class="relative py-1.5 px-2 mx-2 rounded-md flex flex-row justify-between items-center
				{selectedChatId === newChatId ? 'bg-gray-600' : 'hover:bg-gray-800'}"
			on:mousedown={onNewChat}
			role="button"
			aria-pressed="false"
			tabindex="0"
		>
			<div class="text-sm">New Chat</div>
			{#if cmdHeld}
				<div class="absolute right-2 top-1/2 -translate-y-1/2 bg-gray-500 text-white text-xs font-mono px-1.5 py-0.5 rounded">
					N
				</div>
			{:else}
				<Icon
					icon="octicon:comment-discussion-16"
					class="scale-110"
					style="color: white"
				/>
			{/if}
		</div>

		<!-- Chat List -->
		{#each chats as chat, i}
			<ChatItem
				{chat}
				isSelected={chat.id === selectedChatId}
				shortcutIndex={i}
				{cmdHeld}
				onSelect={onSelectChat}
				{onChatsUpdated}
			/>
		{/each}
	</div>
	<hr class="mt-4" />
	<button
		class="flex flex-row mt-2 mb-4 py-1.5 px-2 mx-2 rounded-md hover:bg-gray-800 hover:cursor-pointer justify-between items-center w-[calc(100%-1rem)]"
		on:mousedown={onOpenSettings}
	>
		<span class="text-sm">Settings</span>
		<Icon icon="octicon:gear-24" class="scale-110" style="color: white" />
	</button>
</div>
