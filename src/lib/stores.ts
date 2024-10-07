import { writable } from 'svelte/store'
import * as c from '../../bindings'

export const availableModelsStore = writable<c.Models>({ models: [] })
export const availableProvidersStore = writable<c.ProviderData[]>([])
