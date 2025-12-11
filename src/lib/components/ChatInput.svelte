<script lang="ts">
	import Icon from '@iconify/svelte'

	export let inputText: string = ''
	export let disabled: boolean = false
	export let onSubmit: (text: string) => void
	
	let textareaElement: HTMLTextAreaElement
	let textareaHeight: string = 'auto'
	let textareaOverflow: string = 'hidden'
	
	const MAX_HEIGHT = 300
	
	// Calculate textarea height based on content
	function updateTextareaSize() {
		if (!textareaElement) return
		
		// Reset to auto to get the actual scrollHeight
		textareaElement.style.height = 'auto'
		const scrollHeight = textareaElement.scrollHeight
		
		if (scrollHeight > MAX_HEIGHT) {
			textareaHeight = `${MAX_HEIGHT}px`
			textareaOverflow = 'scroll'
		} else {
			textareaHeight = `${scrollHeight}px`
			textareaOverflow = 'hidden'
		}
	}
	
	function handleInput() {
		updateTextareaSize()
	}
	
	function resetTextarea() {
		textareaHeight = 'auto'
		textareaOverflow = 'hidden'
	}
	
	function handleSubmit(event: Event) {
		event.preventDefault()
		if (inputText.trim() === '' || disabled) return
		
		const textToSend = inputText
		inputText = ''
		resetTextarea()
		onSubmit(textToSend)
	}
	
	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && !event.shiftKey) {
			event.preventDefault()
			handleSubmit(event)
		}
	}
	
	export function focus() {
		textareaElement?.focus()
	}
</script>

<div class="min-w-[12rem] w-full max-w-[56rem] mx-auto px-2">
	<form
		on:submit={handleSubmit}
		class="flex bg-chat-window-gray items-center border border-gray-600 rounded-2xl px-2 py-1 my-4 w-full"
	>
		<textarea
			bind:this={textareaElement}
			class="flex-grow bg-chat-window-gray rounded-lg p-2 text-gray-200 focus:outline-none mx-2 w-full"
			placeholder="Enter your message..."
			rows="1"
			style="resize: none; height: {textareaHeight}; overflow-y: {textareaOverflow};"
			bind:value={inputText}
			on:input={handleInput}
			on:keydown={handleKeydown}
		></textarea>
		<button
			type="submit"
			class="ml-4 text-3xl text-black rounded-lg px-2"
			class:bg-slate-600={disabled}
			class:bg-white={!disabled}
			{disabled}
		>
			<Icon
				icon="octicon:arrow-up-16"
				class="my-1"
			/>
		</button>
	</form>
</div>
