# Service Layer and State Synchronization Design

## Overview

This document describes the architectural design for separating application logic into a Service Layer and implementing bidirectional state synchronization between UI components (StatusBar ↔ PersonasList).

**Date:** 2025-11-15
**Version:** 1.0

---

## 1. Service Layer Architecture

### 1.1 Motivation

**Problems with previous architecture:**
- Application logic (backend persistence, system messages) was duplicated across components
- `App.tsx` became bloated with business logic
- Code reusability was low
- Testing was difficult

**Goals:**
- Extract application logic from UI components
- Centralize business logic in a dedicated service layer
- Make `App.tsx` lightweight (state management + service orchestration only)
- Improve testability and maintainability

### 1.2 Directory Structure

```
src/
├── services/              # 新規: Service Layer
│   └── talkStyleService.ts
├── components/
│   ├── chat/
│   │   └── StatusBar.tsx
│   └── personas/
│       └── PersonasList.tsx
└── App.tsx
```

### 1.3 Service Layer Responsibilities

**`src/services/talkStyleService.ts`**

```typescript
export interface TalkStyleServiceDependencies {
  invoke: <T>(cmd: string, args?: InvokeArgs) => Promise<T>;
  addMessage: (type: MessageType, author: string, text: string) => void;
}

export async function changeTalkStyle(
  style: string | null,
  deps: TalkStyleServiceDependencies
): Promise<void>
```

**Responsibilities:**
1. Backend persistence (`invoke('set_talk_style', { style })`)
2. System message display
3. Error handling

**Benefits:**
- ✅ Single source of truth for talk style change logic
- ✅ No code duplication
- ✅ Testable in isolation (dependency injection pattern)
- ✅ Easy to extend with new services

---

## 2. State Synchronization Architecture

### 2.1 Problem Statement

**Before:**
- StatusBar could change talk style → App.tsx state updated
- PersonasList could change talk style → App.tsx state updated
- But PersonasList didn't reflect changes from StatusBar (no props)

**Issue:**
```
StatusBar change → App.talkStyle updates
                    ↓
                    PersonasList doesn't update (no props binding)
```

### 2.2 Solution: Props-based State Synchronization

**Key principle:** Parent state flows down to all children

```
App.tsx (talkStyle state)
  ↓ props
  ├── Navbar (talkStyle)
  │     ↓ props
  │     └── PersonasList (talkStyle)
  │           ↓ useEffect
  │           └── setSelectedTalkStyle
  │
  └── ChatPanel (talkStyle)
        ↓ props
        └── StatusBar (talkStyle)
```

### 2.3 Implementation Details

#### PersonasList.tsx

```typescript
interface PersonasListProps {
  talkStyle?: string | null;  // Added global state
  onTalkStyleChange?: (style: string | null) => void;
}

export function PersonasList({
  talkStyle: propsTalkStyle,
  onTalkStyleChange,
  ...
}) {
  const [selectedTalkStyle, setSelectedTalkStyle] = useState<string | null>(null);

  // Sync local state with global state (from StatusBar changes)
  useEffect(() => {
    if (propsTalkStyle !== undefined) {
      setSelectedTalkStyle(propsTalkStyle);
    }
  }, [propsTalkStyle]);

  const handleTalkStyleChange = (value: string | null) => {
    setSelectedTalkStyle(value);      // Update local state
    onTalkStyleChange?.(value);       // Notify parent
  };
}
```

**Key points:**
- Local state (`selectedTalkStyle`) for UI rendering
- Props (`propsTalkStyle`) for receiving global changes
- `useEffect` synchronizes props → local state
- `onTalkStyleChange` notifies parent of local changes

#### Navbar.tsx

```typescript
interface NavbarProps {
  talkStyle?: string | null;  // Added
  onTalkStyleChange?: (style: string | null) => void;
}

export function Navbar({ talkStyle, onTalkStyleChange, ... }) {
  return (
    <PersonasList
      talkStyle={talkStyle}                    // Pass down
      onTalkStyleChange={onTalkStyleChange}    // Pass down
    />
  );
}
```

**Role:** Simple props relay (no logic)

#### App.tsx

```typescript
const [talkStyle, setTalkStyle] = useState<string | null>(null);

const handleTalkStyleChange = async (value: string | null) => {
  setTalkStyle(value);                               // Update global state
  await changeTalkStyle(value, { invoke, addMessage }); // Delegate to service
};

return (
  <Navbar
    talkStyle={talkStyle}
    onTalkStyleChange={handleTalkStyleChange}
  />
  <ChatPanel
    talkStyle={talkStyle}
    onTalkStyleChange={handleTalkStyleChange}
  />
);
```

**Responsibilities:**
1. Global state management (`talkStyle`)
2. Service layer orchestration
3. Props distribution to children

---

## 3. Data Flow

### 3.1 Change from StatusBar

```
┌─────────────────────────────────────────────────────────────┐
│ 1. User clicks talk style in StatusBar                     │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. StatusBar calls onTalkStyleChange(style)                │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. App.handleTalkStyleChange                                │
│    - setTalkStyle(style)         // Global state update     │
│    - changeTalkStyle(...)        // Service layer           │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 4. App re-renders with new talkStyle                       │
│    - Navbar receives new talkStyle prop                    │
│    - ChatPanel receives new talkStyle prop                 │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 5. PersonasList.useEffect triggers                         │
│    - propsTalkStyle changed → setSelectedTalkStyle(...)    │
│    - PersonasList UI updates ✅                             │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 Change from PersonasList

```
┌─────────────────────────────────────────────────────────────┐
│ 1. User selects talk style in PersonasList                 │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. PersonasList.handleTalkStyleChange                      │
│    - setSelectedTalkStyle(style)  // Local state update    │
│    - onTalkStyleChange(style)     // Notify parent         │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. App.handleTalkStyleChange                                │
│    - setTalkStyle(style)         // Global state update     │
│    - changeTalkStyle(...)        // Service layer           │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 4. StatusBar receives new talkStyle prop                   │
│    - StatusBar UI updates ✅                                │
└─────────────────────────────────────────────────────────────┘
```

---

## 4. Key Design Patterns

### 4.1 Dependency Injection

**Service functions receive dependencies explicitly:**

```typescript
export async function changeTalkStyle(
  style: string | null,
  deps: TalkStyleServiceDependencies  // ← Injected
): Promise<void>
```

**Benefits:**
- ✅ Testable (mock dependencies)
- ✅ No tight coupling to specific implementations
- ✅ Clear dependencies

### 4.2 Unidirectional Data Flow

```
Parent State (App.tsx)
    ↓ props
Child Component (PersonasList/StatusBar)
    ↓ event callback
Parent State Update
    ↓ props
All Children Re-render
```

**Benefits:**
- ✅ Predictable state updates
- ✅ Easy to debug (single source of truth)
- ✅ No circular dependencies

### 4.3 Local + Global State Hybrid

**PersonasList uses both:**
- **Local state** (`selectedTalkStyle`): For immediate UI updates
- **Global state** (`propsTalkStyle`): For synchronization with other components

**Synchronization:**
```typescript
useEffect(() => {
  if (propsTalkStyle !== undefined) {
    setSelectedTalkStyle(propsTalkStyle);
  }
}, [propsTalkStyle]);
```

---

## 5. Extension Guidelines

### 5.1 Adding New Services

**Template:**

```typescript
// src/services/newFeatureService.ts

export interface NewFeatureServiceDependencies {
  invoke: <T>(cmd: string, args?: InvokeArgs) => Promise<T>;
  addMessage: (type: MessageType, author: string, text: string) => void;
}

export async function changeNewFeature(
  value: SomeType,
  deps: NewFeatureServiceDependencies
): Promise<void> {
  const { invoke, addMessage } = deps;

  try {
    // 1. Backend persistence
    await invoke('backend_command', { value });

    // 2. System message
    await handleAndPersistSystemMessage(
      conversationMessage('Success message', 'info'),
      addMessage,
      invoke
    );
  } catch (error) {
    // 3. Error handling
    console.error('Failed:', error);
    await handleAndPersistSystemMessage(
      conversationMessage(`Error: ${error}`, 'error'),
      addMessage,
      invoke
    );
    throw error;
  }
}
```

**Usage in App.tsx:**

```typescript
import { changeNewFeature } from './services/newFeatureService';

const handleNewFeatureChange = async (value: SomeType) => {
  setNewFeature(value);
  await changeNewFeature(value, { invoke, addMessage });
};
```

### 5.2 Adding State Synchronization

**1. Add props to component:**

```typescript
interface MyComponentProps {
  myState?: SomeType;
  onMyStateChange?: (value: SomeType) => void;
}
```

**2. Add useEffect for synchronization:**

```typescript
const [localState, setLocalState] = useState<SomeType>(defaultValue);

useEffect(() => {
  if (propsMyState !== undefined) {
    setLocalState(propsMyState);
  }
}, [propsMyState]);
```

**3. Relay props through parent components:**

```
App.tsx → Navbar → Target Component
```

---

## 6. Benefits Summary

### 6.1 Service Layer

| Before | After |
|--------|-------|
| Logic duplicated in PersonasList + App.tsx | Single service function |
| Hard to test | Easy to test (DI) |
| App.tsx bloated | App.tsx lightweight |
| Difficult to extend | Easy to add new services |

### 6.2 State Synchronization

| Before | After |
|--------|-------|
| StatusBar changes → PersonasList not updated | ✅ Automatic sync via props |
| PersonasList changes → StatusBar not updated | ✅ Automatic sync via props |
| Inconsistent UI state | ✅ Single source of truth |

---

## 7. Testing Guidelines

### 7.1 Service Layer Tests

```typescript
describe('changeTalkStyle', () => {
  it('should update backend and show system message', async () => {
    const mockInvoke = jest.fn();
    const mockAddMessage = jest.fn();

    await changeTalkStyle('brainstorm', {
      invoke: mockInvoke,
      addMessage: mockAddMessage,
    });

    expect(mockInvoke).toHaveBeenCalledWith('set_talk_style', {
      style: 'brainstorm'
    });
    expect(mockAddMessage).toHaveBeenCalled();
  });
});
```

### 7.2 Component Synchronization Tests

```typescript
describe('PersonasList state sync', () => {
  it('should update local state when props change', () => {
    const { rerender } = render(
      <PersonasList talkStyle="casual" />
    );

    expect(screen.getByText('カジュアル')).toBeInTheDocument();

    rerender(<PersonasList talkStyle="brainstorm" />);

    expect(screen.getByText('ブレインストーミング')).toBeInTheDocument();
  });
});
```

---

## 8. Future Improvements

### 8.1 Global State Management

**Consider introducing Context API or Zustand:**

```typescript
// Current: Props drilling
App → Navbar → PersonasList

// Future: Context API
<TalkStyleContext.Provider value={talkStyle}>
  <Navbar />        // No need to relay props
  <PersonasList />  // Direct access via useContext
</TalkStyleContext.Provider>
```

**Benefits:**
- Eliminate props drilling
- Cleaner component interfaces

### 8.2 Service Layer Enhancements

**Potential improvements:**
- Add transaction support (rollback on error)
- Add caching layer
- Add middleware support (logging, analytics)

---

## Conclusion

This design achieves:

1. ✅ **Separation of Concerns**: UI ↔ Service ↔ Backend
2. ✅ **Single Source of Truth**: All state managed in App.tsx
3. ✅ **Bidirectional Sync**: StatusBar ↔ PersonasList always in sync
4. ✅ **Maintainability**: Easy to test, extend, and debug
5. ✅ **Scalability**: Pattern can be applied to other features

This architecture serves as a template for future feature development in ORCS.
