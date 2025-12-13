<script lang="ts">
	import { onMount, onDestroy } from 'svelte'
	import { commands as c, type Chats, type Message, type Model, type Settings, type Result } from '../../bindings'
	
	// Helper to unwrap Result types from the new bindings format
	function unwrap<T>(result: Result<T, string>): T {
		if (result.status === "ok") return result.data
		throw new Error(result.error)
	}
	import { v4 as uuidv4 } from 'uuid'
	import { checkShortcut } from '$lib/general'
	import SettingsModal from '$lib/modals/Settings.svelte'
	import 'prismjs/themes/prism-funky.css'
	import { listen, type UnlistenFn } from '@tauri-apps/api/event'
	import { availableModelsStore, availableProvidersStore } from '$lib/stores'
	
	// Import extracted components
	import { ChatSidebar, ChatInput, MessageList, ModelSelector } from '$lib/components'

	// State
	let chats: Chats = []
	let currentChatMessages: Message[] = []
	let selectedChatId: string = ''
	let newChatId: string = ''
	let inputText = ''
	let modelSelectorOpen: boolean = false
	let selectedModel: Model
	let selectedModelName: string = ''
	let showSettings: boolean = false
	let settings: Settings
	let cmdHeld: boolean = false
	
	// Component references
	let messageListComponent: MessageList
	let chatInputComponent: ChatInput
	
	// Computed
	$: submitButtonDisabled =
		inputText.trim() === '' ||
		currentChatMessages[currentChatMessages.length - 1]?.role === 'animate'
	$: isNewChat = selectedChatId === newChatId
	
	// Store cleanup functions for event listeners
	let eventUnsubscribers: UnlistenFn[] = []

	onMount(async () => {
		unwrap(await c.readApiKeysFromEnv())
		chats = unwrap(await c.getChats())
		availableModelsStore.set(unwrap(await c.getModels()))
		settings = unwrap(await c.getSettings())
		availableProvidersStore.set(unwrap(await c.loadProviders()))
		
		// Set default model
		if (settings.default_model in $availableModelsStore) {
			selectedModel = $availableModelsStore.find(
				(model) =>
					model.model_name == settings.default_model &&
					model.provider_name == settings.default_provider,
			)!
		} else {
			selectedModel = $availableModelsStore[0]
		}
		selectedModelName = selectedModel?.model_name || ''
		
		newChat()
		
		// Set up Tauri event listeners and store unsubscribe functions
		eventUnsubscribers.push(await listen<string>('newMessage', handleNewMessage))
		eventUnsubscribers.push(await listen<string>('newChat', handleNewChat))
		eventUnsubscribers.push(await listen('menuNewChat', () => newChat()))
		eventUnsubscribers.push(await listen('menuOpenSettings', () => showSettings = true))
	})
	
	onDestroy(() => {
		// Clean up all Tauri event listeners
		eventUnsubscribers.forEach(unsubscribe => unsubscribe())
		eventUnsubscribers = []
	})

	function keydown(e: KeyboardEvent) {
		const isMac = navigator.userAgent.indexOf('Mac') != -1
		const cmdOrCtrl = isMac ? e.metaKey : e.ctrlKey

		if (cmdOrCtrl) {
			cmdHeld = true
		}

		if (checkShortcut(e, 'N', { cmdOrCtrl: true })) {
			newChat()
		}

		// Handle Cmd+1 through Cmd+9 for quick chat switching
		if (cmdOrCtrl && e.key >= '1' && e.key <= '9') {
			const index = parseInt(e.key) - 1
			if (index < chats.length) {
				e.preventDefault()
				inputText = ''
				loadChat(chats[index].id)
			}
		}
		
		// Handle Cmd+ArrowUp/ArrowDown for message navigation
		if (cmdOrCtrl && e.key === 'ArrowUp') {
			e.preventDefault()
			messageListComponent?.scrollToPreviousMessage()
		}
		
		if (cmdOrCtrl && e.key === 'ArrowDown') {
			e.preventDefault()
			messageListComponent?.scrollToNextMessage()
		}
	}

	function keyup(e: KeyboardEvent) {
		const isMac = navigator.userAgent.indexOf('Mac') != -1
		if ((isMac && e.key === 'Meta') || (!isMac && e.key === 'Control')) {
			cmdHeld = false
		}
	}

	async function handleSubmit(text: string) {
		newChatId = ''
		messageListComponent?.scrollToBottom()
		c.getMessage(
			text,
			selectedChatId,
			selectedModel.provider_name,
			selectedModel.model_name,
		)
		chats = unwrap(await c.getChats())
	}

	async function newChat() {
		newChatId = uuidv4()
		selectedChatId = newChatId
		currentChatMessages = unwrap(await c.loadChat(selectedChatId))
		chatInputComponent?.focus()
	}

	async function loadChat(chatId: string) {
		chatInputComponent?.focus()
		selectedChatId = chatId
		currentChatMessages = unwrap(await c.loadChat(selectedChatId))
		
		// Show animation if waiting for response
		if (currentChatMessages[currentChatMessages.length - 1]?.role === 'user') {
			currentChatMessages = [
				...currentChatMessages,
				{ id: 'animationMessage', role: 'animate', content: '', model_name: '', blocks: null },
			]
		}
		
		messageListComponent?.scrollToBottom()
		
		// Update selected model based on chat history
		const offset = currentChatMessages[currentChatMessages.length - 1]?.role === 'animate' ? 2 : 1
		const chatModel = $availableModelsStore.find(
			(model) => model.model_name == currentChatMessages[currentChatMessages.length - offset]?.model_name,
		)
		if (chatModel) {
			selectedModel = chatModel
			selectedModelName = selectedModel.model_name
		}
	}

	async function handleNewMessage(event: { payload: string }) {
		chats = unwrap(await c.getChats())
		if (event.payload == selectedChatId) {
			loadChat(selectedChatId)
		}
	}

	async function handleNewChat() {
		chats = unwrap(await c.getChats())
	}
	
	async function handleChatsUpdated() {
		chats = unwrap(await c.getChats())
		if (chats.length > 0) {
			loadChat(chats[0].id)
		}
	}
	
	function handleSelectChat(chatId: string) {
		inputText = ''
		loadChat(chatId)
	}
</script>

<svelte:window on:keydown={keydown} on:keyup={keyup} on:blur={() => cmdHeld = false} />

<main class="flex h-screen bg-chat-window-gray text-white overflow-y-auto">
	<SettingsModal bind:show={showSettings} />
	
	<ChatSidebar
		{chats}
		{selectedChatId}
		{newChatId}
		{cmdHeld}
		onNewChat={newChat}
		onSelectChat={handleSelectChat}
		onOpenSettings={() => showSettings = true}
		onChatsUpdated={handleChatsUpdated}
	/>

	<div class="flex-1 flex flex-col items-center">
		<ModelSelector
			bind:selectedModel
			bind:selectedModelName
			bind:isOpen={modelSelectorOpen}
		/>
		
		<MessageList
			bind:this={messageListComponent}
			messages={currentChatMessages}
			{selectedModel}
			{isNewChat}
			{selectedModelName}
			{cmdHeld}
		/>
		
		<ChatInput
			bind:this={chatInputComponent}
			bind:inputText
			disabled={submitButtonDisabled}
			onSubmit={handleSubmit}
		/>
	</div>
</main>
