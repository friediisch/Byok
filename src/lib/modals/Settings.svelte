<script lang="ts">
	import Modal from 'modal-svelte'
	import * as c from '../../../bindings'
	import { onMount } from 'svelte'
	import Icon from '@iconify/svelte'
	import { availableModelsStore, availableProvidersStore } from '$lib/stores'
	export let show: boolean = false

	let currentView: string
	let settings: c.Settings
	const themes = [
		'InspiredGitHub',
		'Solarized (dark)',
		'Solarized (light)',
		'base16-eighties.dark',
		'base16-mocha.dark',
		'base16-ocean.dark',
		'base16-ocean.light',
	]
	onMount(async () => {
		availableProvidersStore.set(await c.loadProviders())
		settings = await c.getSettings()
	})
	// if show is set to true, set current view to menu
	$: if (show) {
		currentView = 'menu'
	}

	async function updateApiKey(provider: any) {
		await c.setApiKey(provider)
		availableModelsStore.set(await c.getModels())
		availableProvidersStore.set(await c.loadProviders())
	}

	let typingTimeout: NodeJS.Timeout | null = null

	function handleApiKeyInput(provider: any) {
		if (typingTimeout) clearTimeout(typingTimeout)
		typingTimeout = setTimeout(() => updateApiKey(provider), 500)
	}
</script>

{#if show}
	<Modal
		onCancel={() => {
			show = false
		}}
		class="h-[36rem] w-[36rem]"
	>
		{#if currentView === 'menu'}
			<div class="grid grid-col-1 gap-y-4 m-8">
				<button on:click={() => (currentView = 'api-keys')}
					><span class="hover:underline">API-Keys</span></button
				>
				<button on:click={() => (currentView = 'code-theme')}
					><span class="hover:underline">Code Theme</span></button
				>
				<button>
					<div>For feedback and feature requests: fschestag@icloud.com</div>
				</button>
			</div>
		{:else}
			<div class="flex flex-col justify-between">
				<div>
					{#if currentView === 'api-keys'}
						API-keys:
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
										class="text-black w-96 px-1"
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
					{:else if currentView === 'code-theme'}
						{#each themes as theme}
							<input
								type="radio"
								id={theme}
								name="code-theme"
								value={theme}
								bind:group={settings.code_theme}
								on:change={() => c.applyAndSaveSettings(settings)}
							/>
							<label for={theme}>{theme}</label><br />
						{/each}
					{/if}
				</div>
				<div class="flex justify-center mt-8">
					<button
						on:click={() => (currentView = 'menu')}
						class="flex flex-row p-2 px-4 rounded-md group"
					>
						<Icon
							icon="ic:twotone-arrow-back-ios"
							class="mt-1 mr-2 scale-125"
							style="color: white"
						/>
						<span class="group-hover:underline">Back</span></button
					>
				</div>
			</div>
		{/if}
	</Modal>
{/if}
