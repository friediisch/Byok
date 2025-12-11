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
</script>

<div
	class="flex flex-col min-w-72 max-w-96 bg-sidebar-gray overflow-y-auto overscroll-contain h-screen px-4 pt-6"
>
	<div class="text-3xl pl-2 font-bold">Byok</div>
	<hr class="my-4" />
	<div class="overflow-y-auto flex-1">
		<!-- New Chat Button -->
		<div
			class="relative p-2 m-2 rounded-md flex flex-row justify-between
				{selectedChatId === newChatId ? 'bg-gray-600' : 'hover:bg-gray-800'}"
			on:mousedown={onNewChat}
			role="button"
			aria-pressed="false"
			tabindex="0"
		>
			<div>New Chat</div>
			{#if cmdHeld}
				<div class="absolute right-2 top-1/2 -translate-y-1/2 bg-gray-500 text-white text-xs font-mono px-1.5 py-0.5 rounded">
					N
				</div>
			{:else}
				<Icon
					icon="octicon:comment-discussion-16"
					class="mt-1 mr-2 scale-125"
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
		class="flex flex-row mt-2 mb-4 p-2 rounded-md hover:bg-gray-800 hover:cursor-pointer justify-between"
		on:mousedown={onOpenSettings}
	>
		Settings
		<Icon icon="octicon:gear-24" class="mt-1 mr-2 scale-125" style="color: white" />
	</button>
</div>
