<script lang="ts">
	import Modal from 'modal-svelte'
	import { commands as c, type Settings, type Result, type Model, type Models } from '../../../bindings'
	
	// Helper to unwrap Result types from the new bindings format
	function unwrap<T>(result: Result<T, string>): T {
		if (result.status === "ok") return result.data
		throw new Error(result.error)
	}
	import { onMount } from 'svelte'
	import Icon from '@iconify/svelte'
	import { availableModelsStore, availableProvidersStore } from '$lib/stores'
	export let show: boolean = false

	let currentView: string
	let settings: Settings
	const themes = [
		'InspiredGitHub',
		'Solarized (dark)',
		'Solarized (light)',
		'base16-eighties.dark',
		'base16-mocha.dark',
		'base16-ocean.dark',
		'base16-ocean.light',
	]
	
	// Models management state
	let allModels: Models = []
	let editingModel: Model | null = null
	let isAddingModel: boolean = false
	let newModel: Model = {
		provider_name: '',
		model_name: '',
		model_display_name: '',
		show: true,
		max_tokens: 4096,
		context_window: 8192
	}
	let modelError: string = ''
	
	onMount(async () => {
		availableProvidersStore.set(unwrap(await c.loadProviders()))
		settings = unwrap(await c.getSettings())
	})
	
	// if show is set to true, set current view to menu
	$: if (show) {
		currentView = 'menu'
		editingModel = null
		isAddingModel = false
		modelError = ''
	}

	async function updateApiKey(provider: any) {
		unwrap(await c.setApiKey(provider))
		availableModelsStore.set(unwrap(await c.getModels()))
		availableProvidersStore.set(unwrap(await c.loadProviders()))
	}

	let typingTimeout: NodeJS.Timeout | null = null

	function handleApiKeyInput(provider: any) {
		if (typingTimeout) clearTimeout(typingTimeout)
		typingTimeout = setTimeout(() => updateApiKey(provider), 500)
	}
	
	async function loadAllModels() {
		allModels = unwrap(await c.getAllModels())
	}
	
	async function handleAddModel() {
		modelError = ''
		if (!newModel.provider_name.trim() || !newModel.model_name.trim() || !newModel.model_display_name.trim()) {
			modelError = 'Please fill in all required fields'
			return
		}
		try {
			const result = await c.addModel(newModel)
			if (result.status === 'error') {
				modelError = result.error
				return
			}
			await loadAllModels()
			availableModelsStore.set(unwrap(await c.getModels()))
			isAddingModel = false
			newModel = {
				provider_name: '',
				model_name: '',
				model_display_name: '',
				show: true,
				max_tokens: 4096,
				context_window: 8192
			}
		} catch (e: any) {
			modelError = e.message || 'Failed to add model'
		}
	}
	
	async function handleUpdateModel() {
		if (!editingModel) return
		modelError = ''
		try {
			const result = await c.updateModel(editingModel)
			if (result.status === 'error') {
				modelError = result.error
				return
			}
			await loadAllModels()
			availableModelsStore.set(unwrap(await c.getModels()))
			editingModel = null
		} catch (e: any) {
			modelError = e.message || 'Failed to update model'
		}
	}
	
	async function handleDeleteModel(model: Model) {
		modelError = ''
		try {
			const result = await c.deleteModel(model.provider_name, model.model_name)
			if (result.status === 'error') {
				modelError = result.error
				return
			}
			await loadAllModels()
			availableModelsStore.set(unwrap(await c.getModels()))
		} catch (e: any) {
			modelError = e.message || 'Failed to delete model'
		}
	}
	
	function startEditModel(model: Model) {
		editingModel = { ...model }
		isAddingModel = false
		modelError = ''
	}
	
	function startAddModel() {
		isAddingModel = true
		editingModel = null
		modelError = ''
		newModel = {
			provider_name: $availableProvidersStore[0]?.provider_name || '',
			model_name: '',
			model_display_name: '',
			show: true,
			max_tokens: 4096,
			context_window: 8192
		}
	}
	
	function cancelEdit() {
		editingModel = null
		isAddingModel = false
		modelError = ''
	}
</script>

{#if show}
	<Modal
		onCancel={() => {
			show = false
		}}
		class="h-[36rem] w-[42rem]"
	>
		{#if currentView === 'menu'}
			<div class="grid grid-col-1 gap-y-4 m-8">
				<button on:click={() => (currentView = 'api-keys')}
					><span class="hover:underline">API-Keys</span></button
				>
				<button on:click={() => { currentView = 'models'; loadAllModels(); }}
					><span class="hover:underline">Models</span></button
				>
				<button on:click={() => (currentView = 'code-theme')}
					><span class="hover:underline">Code Theme</span></button
				>
				<button>
					<div>For feedback and feature requests: fschestag@icloud.com</div>
				</button>
			</div>
		{:else}
			<div class="flex flex-col justify-between h-full">
				<div class="overflow-y-auto flex-1">
					{#if currentView === 'api-keys'}
						<div class="text-lg font-semibold mb-4">API Keys</div>
						<form>
							{#each $availableProvidersStore as provider}
								<div class="flex flex-row m-1 items-center">
									<label for="{provider.provider_name}-api-key" class="w-24"
										>{provider.display_name}:</label
									>
									<input
										type="password"
										id="{provider.provider_name}-api-key"
										name="{provider.provider_name}-api-key"
										class="text-black w-96 px-1 rounded"
										bind:value={provider.api_key}
										on:input={() => handleApiKeyInput(provider)}
									/>
									{#if provider.api_key_valid}
										<Icon icon="mdi:check-circle" class="text-green-500 ml-2" />
									{:else}
										<Icon icon="mdi:close-circle" class="text-red-500 ml-2" />
									{/if}
								</div>
							{/each}
						</form>
					{:else if currentView === 'models'}
						<div class="text-lg font-semibold mb-4">Models</div>
						
						{#if modelError}
							<div class="bg-red-500/20 border border-red-500 text-red-300 px-3 py-2 rounded mb-4">
								{modelError}
							</div>
						{/if}
						
						{#if isAddingModel}
							<!-- Add Model Form -->
							<div class="bg-gray-700/50 rounded-lg p-4 mb-4">
								<div class="text-md font-semibold mb-3">Add New Model</div>
								<div class="space-y-3">
									<div class="flex items-center gap-2">
										<label class="w-32 text-sm">Provider:</label>
										<select 
											class="flex-1 bg-gray-800 text-white px-2 py-1 rounded border border-gray-600"
											bind:value={newModel.provider_name}
										>
											{#each $availableProvidersStore as provider}
												<option value={provider.provider_name}>{provider.display_name}</option>
											{/each}
										</select>
									</div>
									<div class="flex items-center gap-2">
										<label class="w-32 text-sm">Model Name:</label>
										<input 
											type="text" 
											class="flex-1 bg-gray-800 text-white px-2 py-1 rounded border border-gray-600"
											placeholder="e.g. gpt-4o-mini"
											bind:value={newModel.model_name}
										/>
									</div>
									<div class="flex items-center gap-2">
										<label class="w-32 text-sm">Display Name:</label>
										<input 
											type="text" 
											class="flex-1 bg-gray-800 text-white px-2 py-1 rounded border border-gray-600"
											placeholder="e.g. GPT-4o Mini"
											bind:value={newModel.model_display_name}
										/>
									</div>
									<div class="flex items-center gap-2">
										<label class="w-32 text-sm">Max Tokens:</label>
										<input 
											type="number" 
											class="flex-1 bg-gray-800 text-white px-2 py-1 rounded border border-gray-600"
											bind:value={newModel.max_tokens}
										/>
									</div>
									<div class="flex items-center gap-2">
										<label class="w-32 text-sm">Context Window:</label>
										<input 
											type="number" 
											class="flex-1 bg-gray-800 text-white px-2 py-1 rounded border border-gray-600"
											bind:value={newModel.context_window}
										/>
									</div>
									<div class="flex items-center gap-2">
										<label class="w-32 text-sm">Visible:</label>
										<input 
											type="checkbox" 
											class="w-5 h-5"
											bind:checked={newModel.show}
										/>
									</div>
									<div class="flex gap-2 pt-2">
										<button 
											type="button"
											class="px-4 py-1.5 bg-emerald-600 hover:bg-emerald-500 rounded text-sm"
											on:click={handleAddModel}
										>
											Add Model
										</button>
										<button 
											type="button"
											class="px-4 py-1.5 bg-gray-600 hover:bg-gray-500 rounded text-sm"
											on:click={cancelEdit}
										>
											Cancel
										</button>
									</div>
								</div>
							</div>
						{:else if editingModel}
							<!-- Edit Model Form -->
							<div class="bg-gray-700/50 rounded-lg p-4 mb-4">
								<div class="text-md font-semibold mb-3">Edit Model</div>
								<div class="space-y-3">
									<div class="flex items-center gap-2">
										<label class="w-32 text-sm text-gray-400">Provider:</label>
										<span class="text-gray-300">{editingModel.provider_name}</span>
									</div>
									<div class="flex items-center gap-2">
										<label class="w-32 text-sm text-gray-400">Model Name:</label>
										<span class="text-gray-300">{editingModel.model_name}</span>
									</div>
									<div class="flex items-center gap-2">
										<label class="w-32 text-sm">Display Name:</label>
										<input 
											type="text" 
											class="flex-1 bg-gray-800 text-white px-2 py-1 rounded border border-gray-600"
											bind:value={editingModel.model_display_name}
										/>
									</div>
									<div class="flex items-center gap-2">
										<label class="w-32 text-sm">Max Tokens:</label>
										<input 
											type="number" 
											class="flex-1 bg-gray-800 text-white px-2 py-1 rounded border border-gray-600"
											bind:value={editingModel.max_tokens}
										/>
									</div>
									<div class="flex items-center gap-2">
										<label class="w-32 text-sm">Context Window:</label>
										<input 
											type="number" 
											class="flex-1 bg-gray-800 text-white px-2 py-1 rounded border border-gray-600"
											bind:value={editingModel.context_window}
										/>
									</div>
									<div class="flex items-center gap-2">
										<label class="w-32 text-sm">Visible:</label>
										<input 
											type="checkbox" 
											class="w-5 h-5"
											bind:checked={editingModel.show}
										/>
									</div>
									<div class="flex gap-2 pt-2">
										<button 
											type="button"
											class="px-4 py-1.5 bg-blue-600 hover:bg-blue-500 rounded text-sm"
											on:click={handleUpdateModel}
										>
											Save Changes
										</button>
										<button 
											type="button"
											class="px-4 py-1.5 bg-gray-600 hover:bg-gray-500 rounded text-sm"
											on:click={cancelEdit}
										>
											Cancel
										</button>
									</div>
								</div>
							</div>
						{:else}
							<!-- Models List -->
							<button 
								type="button"
								class="mb-4 px-4 py-2 bg-emerald-600 hover:bg-emerald-500 rounded flex items-center gap-2"
								on:click={startAddModel}
							>
								<Icon icon="mdi:plus" />
								Add Model
							</button>
							
							<div class="space-y-2">
								{#each $availableProvidersStore as provider}
									{@const providerModels = allModels.filter(m => m.provider_name === provider.provider_name)}
									{#if providerModels.length > 0}
										<div class="mb-4">
											<div class="font-semibold text-sm text-gray-400 mb-2">{provider.display_name}</div>
											{#each providerModels as model}
												<div class="flex items-center justify-between bg-gray-700/30 rounded px-3 py-2 mb-1 group">
													<div class="flex items-center gap-3">
														<span class="text-sm" class:text-gray-500={!model.show}>
															{model.model_display_name}
														</span>
														<span class="text-xs text-gray-500">
															{model.model_name}
														</span>
														{#if !model.show}
															<span class="text-xs bg-gray-600 px-1.5 py-0.5 rounded">hidden</span>
														{/if}
													</div>
													<div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
														<button 
															type="button"
															class="p-1.5 hover:bg-gray-600 rounded"
															title="Edit"
															on:click={() => startEditModel(model)}
														>
															<Icon icon="mdi:pencil" class="text-blue-400" />
														</button>
														<button 
															type="button"
															class="p-1.5 hover:bg-gray-600 rounded"
															title="Delete"
															on:click={() => handleDeleteModel(model)}
														>
															<Icon icon="mdi:delete" class="text-red-400" />
														</button>
													</div>
												</div>
											{/each}
										</div>
									{/if}
								{/each}
							</div>
						{/if}
					{:else if currentView === 'code-theme'}
						<div class="text-lg font-semibold mb-4">Code Theme</div>
						{#each themes as theme}
							<div class="mb-1">
								<input
									type="radio"
									id={theme}
									name="code-theme"
									value={theme}
									bind:group={settings.code_theme}
									on:change={() => c.applyAndSaveSettings(settings)}
								/>
								<label for={theme} class="ml-2 cursor-pointer">{theme}</label>
							</div>
						{/each}
					{/if}
				</div>
				<div class="flex justify-center mt-4 pt-4 border-t border-gray-600">
					<button
						on:click={() => (currentView = 'menu')}
						class="flex flex-row p-2 px-4 rounded-md group hover:bg-gray-700"
					>
						<Icon
							icon="ic:twotone-arrow-back-ios"
							class="mt-1 mr-2 scale-125"
							style="color: white"
						/>
						<span class="group-hover:underline">Back</span>
					</button>
				</div>
			</div>
		{/if}
	</Modal>
{/if}
