<script lang="ts">
	import type { Chat } from '../../../bindings'
	import Icon from '@iconify/svelte'
	import { commands as c, type Result } from '../../../bindings'
	
	function unwrap<T>(result: Result<T, string>): T {
		if (result.status === "ok") return result.data
		throw new Error(result.error)
	}
	
	export let chat: Chat
	export let isSelected: boolean = false
	export let shortcutIndex: number = -1
	export let cmdHeld: boolean = false
	export let onSelect: (chatId: string) => void
	export let onChatsUpdated: () => void
	
	let showContextMenu: boolean = false
	let isRenaming: boolean = false
	let renameInput: HTMLTextAreaElement
	
	function handleSelect() {
		onSelect(chat.id)
	}
	
	function toggleContextMenu(event: MouseEvent) {
		event.stopPropagation()
		showContextMenu = !showContextMenu
		handleSelect()
	}
	
	async function startRename() {
		showContextMenu = false
		isRenaming = true
		setTimeout(() => {
			renameInput?.focus()
		}, 0)
	}
	
	async function finishRename() {
		chat.display_name = chat.display_name.trim()
		await c.renameChat(chat.id, chat.display_name)
		isRenaming = false
	}
	
	function handleRenameKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			event.preventDefault()
			finishRename()
		}
	}
	
	async function archiveChat() {
		showContextMenu = false
		await c.archiveChat(chat.id)
		onChatsUpdated()
	}
	
	async function deleteChat() {
		showContextMenu = false
		await c.deleteChat(chat.id)
		onChatsUpdated()
	}
</script>

<div
	class="relative block p-2 mx-2 rounded-md group
		{isSelected ? 'bg-gray-600' : 'hover:bg-gray-800'}"
	on:mousedown={handleSelect}
	role="button"
	aria-pressed="false"
	tabindex="0"
>
	{#if cmdHeld && shortcutIndex >= 0 && shortcutIndex < 9}
		<div class="absolute right-2 top-1/2 -translate-y-1/2 bg-gray-500 text-white text-xs font-mono px-1.5 py-0.5 rounded">
			{shortcutIndex + 1}
		</div>
	{/if}
	
	{#if chat.display_name.startsWith('unnamed_new_chat_')}
		<div
			class="block p-2 mx-2 animate-ping rounded-full self-center self-middle size-4 bg-white opacity-100"
		></div>
	{:else}
		<div class="flex flex-row justify-between">
			{#if isRenaming}
				<textarea
					class="flex flex-grow p-2 bg-gray-600 rounded-md"
					bind:this={renameInput}
					bind:value={chat.display_name}
					on:keydown={handleRenameKeydown}
					on:blur={finishRename}
					rows="1"
					style="resize: none;"
				></textarea>
			{:else}
				<div
					class="flex flex-grow break-all"
					on:mousedown={() => showContextMenu = false}
					role="button"
					aria-pressed="false"
					tabindex="0"
				>
					{chat.display_name}
				</div>
			{/if}
			<div
				on:mousedown={toggleContextMenu}
				role="button"
				aria-pressed="false"
				tabindex="0"
			>
				<Icon
					icon="mdi:dots-horizontal"
					class="mt-1 m-2 scale-125 opacity-0 group-hover:opacity-100 hover:cursor-pointer"
					style="color: white"
				/>
			</div>
		</div>
		
		{#if showContextMenu && isSelected}
			<div class="flex flex-row justify-between px-8">
				<div
					class="bg-gray2 rounded-md p-2 mt-2"
					on:mousedown={startRename}
					role="button"
					aria-pressed="false"
					tabindex="0"
				>
					Rename
				</div>
				<div
					class="bg-gray2 rounded-md p-2 mt-2"
					on:mousedown={archiveChat}
					role="button"
					aria-pressed="false"
					tabindex="0"
				>
					Archive
				</div>
				<div
					class="bg-gray2 rounded-md p-2 mt-2"
					on:mousedown={deleteChat}
					role="button"
					aria-pressed="false"
					tabindex="0"
				>
					Delete
				</div>
			</div>
		{/if}
	{/if}
</div>
