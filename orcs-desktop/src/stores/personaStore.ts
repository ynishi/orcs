/**
 * Persona Store
 * Backend-First SSOT pattern for workspace-scoped Persona management
 *
 * Scope: Workspace/Global (not session-specific)
 * Session-specific activeParticipantIds remain in sessionSettingsStore
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { PersonaConfig } from '../types/agent';

export interface PersonaStore {
  // State
  personas: PersonaConfig[];
  isLoaded: boolean;

  // Actions
  loadPersonas: () => Promise<void>;
  savePersonaConfigs: (configs: PersonaConfig[]) => Promise<void>;
  addPersona: (persona: PersonaConfig) => Promise<void>;
  updatePersona: (persona: PersonaConfig) => Promise<void>;
  deletePersona: (personaId: string) => Promise<void>;
  saveAdhocPersona: (personaId: string) => Promise<void>;

  // Getters
  getPersonaById: (personaId: string) => PersonaConfig | undefined;
}

export const usePersonaStore = create<PersonaStore>((set, get) => ({
  // Initial state
  personas: [],
  isLoaded: false,

  // Actions
  loadPersonas: async () => {
    console.log('[PersonaStore] Loading personas...');

    try {
      const personas = await invoke<PersonaConfig[]>('get_personas');

      set({
        personas,
        isLoaded: true,
      });

      console.log('[PersonaStore] Personas loaded:', personas.length);
    } catch (error) {
      console.error('[PersonaStore] Failed to load personas:', error);
      // Set empty array on error to avoid blocking UI
      set({
        personas: [],
        isLoaded: true,
      });
    }
  },

  savePersonaConfigs: async (configs: PersonaConfig[]) => {
    console.log('[PersonaStore] Saving persona configs:', configs.length);

    try {
      await invoke('save_persona_configs', { configs });

      // Update local state
      set({ personas: configs });

      console.log('[PersonaStore] Persona configs saved successfully');
    } catch (error) {
      console.error('[PersonaStore] Failed to save persona configs:', error);
      throw error;
    }
  },

  addPersona: async (persona: PersonaConfig) => {
    console.log('[PersonaStore] Adding persona:', persona.id);

    const currentPersonas = get().personas;
    const updatedPersonas = [...currentPersonas, persona];

    await get().savePersonaConfigs(updatedPersonas);
  },

  updatePersona: async (persona: PersonaConfig) => {
    console.log('[PersonaStore] Updating persona:', persona.id);

    const currentPersonas = get().personas;
    const updatedPersonas = currentPersonas.map(p =>
      p.id === persona.id ? persona : p
    );

    await get().savePersonaConfigs(updatedPersonas);
  },

  deletePersona: async (personaId: string) => {
    console.log('[PersonaStore] Deleting persona:', personaId);

    const currentPersonas = get().personas;
    const updatedPersonas = currentPersonas.filter(p => p.id !== personaId);

    await get().savePersonaConfigs(updatedPersonas);
  },

  saveAdhocPersona: async (personaId: string) => {
    console.log('[PersonaStore] Saving adhoc persona:', personaId);

    try {
      await invoke('save_adhoc_persona', { personaId });

      // Reload personas to get updated list
      await get().loadPersonas();

      console.log('[PersonaStore] Adhoc persona saved successfully');
    } catch (error) {
      console.error('[PersonaStore] Failed to save adhoc persona:', error);
      throw error;
    }
  },

  // Getters
  getPersonaById: (personaId: string) => {
    const state = get();
    return state.personas.find(p => p.id === personaId);
  },
}));
