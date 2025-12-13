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

  addPersona: async (persona: PersonaConfig) => {
    console.log('[PersonaStore] Adding persona:', persona.id);

    try {
      await invoke('save_persona', { persona });

      set(state => ({
        personas: [...state.personas.filter(p => p.id !== persona.id), persona],
      }));

      console.log('[PersonaStore] Persona added successfully');
    } catch (error) {
      console.error('[PersonaStore] Failed to add persona:', error);
      throw error;
    }
  },

  updatePersona: async (persona: PersonaConfig) => {
    console.log('[PersonaStore] Updating persona:', persona.id);

    try {
      await invoke('save_persona', { persona });

      set(state => ({
        personas: state.personas.map(p =>
          p.id === persona.id ? persona : p
        ),
      }));

      console.log('[PersonaStore] Persona updated successfully');
    } catch (error) {
      console.error('[PersonaStore] Failed to update persona:', error);
      throw error;
    }
  },

  deletePersona: async (personaId: string) => {
    console.log('[PersonaStore] Deleting persona:', personaId);

    try {
      await invoke('delete_persona', { personaId });

      set(state => ({
        personas: state.personas.filter(p => p.id !== personaId),
      }));

      console.log('[PersonaStore] Persona deleted successfully');
    } catch (error) {
      console.error('[PersonaStore] Failed to delete persona:', error);
      throw error;
    }
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
