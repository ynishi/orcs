import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { Task, TaskProgress, TaskStatus } from '../types/task';

interface TaskStore {
  // State
  tasks: Map<string, Task>;
  taskProgress: Map<string, TaskProgress>;
  isLoaded: boolean;

  // Actions
  initialize: () => Promise<void>;
  refreshTasks: () => Promise<void>;
  deleteTask: (taskId: string) => Promise<void>;

  // Getters
  getTask: (taskId: string) => Task | undefined;
  getTasksBySession: (sessionId: string) => Task[];
  getRunningTasks: () => Task[];
}

export const useTaskStore = create<TaskStore>((set, get) => ({
  tasks: new Map(),
  taskProgress: new Map(),
  isLoaded: false,

  initialize: async () => {
    console.log('[TaskStore] Initializing...');

    // 1. Get initial snapshot
    const snapshot = await invoke<Task[]>('get_tasks_snapshot');
    const tasksMap = new Map(snapshot.map(t => [t.id, t]));
    set({ tasks: tasksMap, isLoaded: true });
    console.log('[TaskStore] Initial snapshot:', snapshot.length, 'tasks');

    // 2. Listen for task-event (real-time progress)
    await listen<any>('task-event', (event) => {
      const payload = event.payload;

      // Filter by event_type
      const isTaskLifecycleEvent = payload.event_type === 'task_lifecycle';
      const isOrchestratorEvent = payload.target?.includes('llm_toolkit') ||
                                   payload.target?.includes('parallel_orchestrator');

      if (!isTaskLifecycleEvent && !isOrchestratorEvent) {
        return;
      }

      const taskId = payload.fields?.task_id;
      const status = payload.fields?.status;

      // Task lifecycle event - update task
      if (isTaskLifecycleEvent && taskId && status) {
        console.log(`[TaskStore] Task lifecycle event: task_id=${taskId}, status=${status}`);

        set((state) => {
          const updatedTask: Task = {
            id: payload.fields.taskId,
            sessionId: payload.fields.sessionId,
            title: payload.fields.title || '',
            description: payload.fields.description || '',
            status: payload.fields.status as TaskStatus,
            createdAt: payload.fields.createdAt,
            updatedAt: payload.fields.updatedAt,
            completedAt: payload.fields.completedAt,
            stepsExecuted: payload.fields.stepsExecuted || 0,
            stepsSkipped: payload.fields.stepsSkipped || 0,
            contextKeys: payload.fields.contextKeys || 0,
            error: payload.fields.error,
            result: payload.fields.result,
            executionDetails: payload.fields.executionDetails,
          };

          const newTasks = new Map(state.tasks);
          newTasks.set(taskId, updatedTask);

          // Clear progress if completed/failed
          const newProgress = new Map(state.taskProgress);
          if (status === 'Completed' || status === 'Failed') {
            newProgress.delete(taskId);
          }

          return { tasks: newTasks, taskProgress: newProgress };
        });
      } else if (taskId) {
        // Progress update
        set((state) => {
          const newProgress = new Map(state.taskProgress);

          if (status === 'Completed' || status === 'Failed') {
            newProgress.delete(taskId);
          } else {
            newProgress.set(taskId, {
              taskId,
              currentWave: payload.fields?.wave_number,
              currentStep: payload.fields?.step_id,
              currentAgent: payload.fields?.agent,
              lastMessage: payload.message,
              lastUpdated: Date.now(),
            });
          }

          return { taskProgress: newProgress };
        });
      } else if (isOrchestratorEvent) {
        // ParallelOrchestrator event - find running task
        const runningTask = Array.from(get().tasks.values())
          .find(t => t.status === 'Running');

        if (runningTask) {
          set((state) => {
            const newProgress = new Map(state.taskProgress);
            newProgress.set(runningTask.id, {
              taskId: runningTask.id,
              currentWave: payload.fields?.wave_number,
              currentStep: payload.fields?.step_id,
              currentAgent: payload.fields?.agent,
              lastMessage: payload.message,
              lastUpdated: Date.now(),
            });
            return { taskProgress: newProgress };
          });
        }
      }
    });

    // 3. Listen for task:delete (future)
    await listen<string>('task:delete', (event) => {
      console.log('[TaskStore] Task deleted:', event.payload);
      set((state) => {
        const newTasks = new Map(state.tasks);
        newTasks.delete(event.payload);
        return { tasks: newTasks };
      });
    });

    console.log('[TaskStore] Initialized successfully');
  },

  refreshTasks: async () => {
    console.log('[TaskStore] Refreshing tasks...');
    const snapshot = await invoke<Task[]>('list_tasks');
    const tasksMap = new Map(snapshot.map(t => [t.id, t]));
    set({ tasks: tasksMap });
    console.log('[TaskStore] Refreshed:', snapshot.length, 'tasks');
  },

  deleteTask: async (taskId: string) => {
    console.log('[TaskStore] Deleting task:', taskId);
    await invoke('delete_task', { taskId });
    // Note: Backend will emit task:delete event (future)
    // For now, manually remove
    set((state) => {
      const newTasks = new Map(state.tasks);
      newTasks.delete(taskId);
      return { tasks: newTasks };
    });
  },

  getTask: (taskId: string) => {
    return get().tasks.get(taskId);
  },

  getTasksBySession: (sessionId: string) => {
    return Array.from(get().tasks.values())
      .filter(t => t.sessionId === sessionId)
      .sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime());
  },

  getRunningTasks: () => {
    return Array.from(get().tasks.values())
      .filter(t => t.status === 'Running');
  },
}));
