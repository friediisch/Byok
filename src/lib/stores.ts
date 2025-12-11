import { writable, derived, type Writable, type Readable } from "svelte/store";
import type {
  Models,
  ProviderData,
  Settings,
  Chats,
  Message,
  Model,
} from "../../bindings";

// ============================================================================
// Store Definitions
// ============================================================================

/** Available LLM models */
export const availableModelsStore: Writable<Models> = writable([]);

/** Available providers with their configurations */
export const availableProvidersStore: Writable<ProviderData[]> = writable([]);

/** Application settings */
export const settingsStore: Writable<Settings | null> = writable(null);

/** Current chat state */
interface ChatState {
  chats: Chats;
  selectedChatId: string | null;
  currentMessages: Message[];
  isLoading: boolean;
}

export const chatStateStore: Writable<ChatState> = writable({
  chats: [],
  selectedChatId: null,
  currentMessages: [],
  isLoading: false,
});

/** Currently selected model */
export const selectedModelStore: Writable<Model | null> = writable(null);

// ============================================================================
// Derived Stores
// ============================================================================

/** Models grouped by provider */
export const modelsByProvider: Readable<Map<string, Model[]>> = derived(
  [availableModelsStore, availableProvidersStore],
  ([$models, $providers]) => {
    const grouped = new Map<string, Model[]>();
    for (const provider of $providers) {
      const providerModels = $models.filter(
        (m) => m.provider_name === provider.provider_name
      );
      if (providerModels.length > 0) {
        grouped.set(provider.provider_name, providerModels);
      }
    }
    return grouped;
  }
);

/** Providers with valid API keys */
export const validProvidersStore: Readable<ProviderData[]> = derived(
  availableProvidersStore,
  ($providers) => $providers.filter((p) => p.api_key_valid)
);

/** Check if currently waiting for a response */
export const isWaitingForResponse: Readable<boolean> = derived(
  chatStateStore,
  ($state) => {
    const lastMessage =
      $state.currentMessages[$state.currentMessages.length - 1];
    return lastMessage?.role === "animate";
  }
);

// ============================================================================
// Store Actions / Helpers
// ============================================================================

/** Update settings store and optionally persist to backend */
export function updateSettings(updates: Partial<Settings>) {
  settingsStore.update((current) => {
    if (!current) return current;
    return { ...current, ...updates };
  });
}

/** Set the selected chat and messages */
export function setCurrentChat(chatId: string, messages: Message[]) {
  chatStateStore.update((state) => ({
    ...state,
    selectedChatId: chatId,
    currentMessages: messages,
  }));
}

/** Update the chat list */
export function updateChats(chats: Chats) {
  chatStateStore.update((state) => ({
    ...state,
    chats,
  }));
}

/** Set loading state */
export function setLoading(isLoading: boolean) {
  chatStateStore.update((state) => ({
    ...state,
    isLoading,
  }));
}

/** Add a message to the current chat */
export function addMessage(message: Message) {
  chatStateStore.update((state) => ({
    ...state,
    currentMessages: [...state.currentMessages, message],
  }));
}

/** Clear messages (for new chat) */
export function clearMessages() {
  chatStateStore.update((state) => ({
    ...state,
    currentMessages: [],
  }));
}
