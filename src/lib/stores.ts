import { writable } from 'svelte/store'
import type { Models, ProviderData } from '../../bindings'

export const availableModelsStore = writable<Models>([])
export const availableProvidersStore = writable<ProviderData[]>([])
