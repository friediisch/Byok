<script lang="ts">
	import { onMount, onDestroy } from 'svelte'
	import type { Model } from '../../../bindings'
	import { availableModelsStore, availableProvidersStore } from '$lib/stores'
	
	export let selectedModel: Model
	export let selectedModelName: string = ''
	export let isOpen: boolean = false
	
	let selectorContainer: HTMLElement
	let buttonElement: HTMLElement
	
	function handleModelSelect(model: Model) {
		selectedModel = model
		selectedModelName = model.model_name
		isOpen = false
	}
	
	function toggleOpen(event: MouseEvent) {
		event.stopPropagation()
		isOpen = !isOpen
	}
	
	// Handle clicks outside the selector to close it
	function handleClickOutside(event: MouseEvent) {
		if (!isOpen) return
		
		const target = event.target as Node
		if (selectorContainer && buttonElement) {
			if (!selectorContainer.contains(target) && !buttonElement.contains(target)) {
				isOpen = false
			}
		}
	}
	
	onMount(() => {
		document.addEventListener('click', handleClickOutside)
	})
	
	onDestroy(() => {
		document.removeEventListener('click', handleClickOutside)
	})
</script>

<div class="w-full h-fit px-2">
	<button
		bind:this={buttonElement}
		class="group text-lg px-2 py-1 align-middle hover:bg-gray2 w-fit rounded-md cursor-pointer my-2 mx-1"
		on:mousedown={toggleOpen}
	>
		{selectedModelName}
		<span class="icon-[octicon--chevron-down-12] scale-75 text-white"></span>
	</button>
	<hr class="border-gray-600" />
	{#if isOpen}
		<div
			bind:this={selectorContainer}
			class="absolute z-10 bg-gray2 rounded-md p-2 mt-2 overflow-x-scroll"
			style="max-height: 80%;"
		>
			{#each $availableProvidersStore as provider}
				{@const providerModels = $availableModelsStore.filter(
					(model) => model.provider_name === provider.provider_name
				)}
				{#if providerModels.length > 0}
					<div class="font-bold">{provider.display_name}</div>
					<hr class="border-gray-600 pb-1" />
					{#each providerModels as model}
						<button
							class="block p-2 mx-2 hover:bg-gray-600 rounded-md w-full text-left"
							class:bg-white={selectedModel === model}
							class:text-black={selectedModel === model}
							on:mousedown={(e) => {
								e.stopPropagation()
								handleModelSelect(model)
							}}
						>
							{model.model_name}
						</button>
					{/each}
				{/if}
			{/each}
		</div>
	{/if}
</div>
