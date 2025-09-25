# LLM.md - Handy Dictation App Architecture Guide

This document provides a comprehensive analysis of the Handy dictation app codebase, designed to help AI assistants understand the structure, functionality, and relationships between components.

## Overview

Handy is a cross-platform desktop speech-to-text application built with Tauri (Rust backend + React/TypeScript frontend). It provides real-time voice transcription with advanced features like Voice Activity Detection (VAD), PII redaction, and global keyboard shortcuts.

## Project Structure

```
Handy/
├── src/                     # Frontend React/TypeScript code
├── src-tauri/              # Backend Rust/Tauri code
├── package.json            # Frontend dependencies and scripts
├── src-tauri/Cargo.toml    # Rust dependencies and configuration
├── CLAUDE.md               # Development instructions for AI assistants
├── BUILD.md                # Build and deployment instructions
└── README.md               # User-facing documentation
```

## Frontend Architecture (src/)

### Core Application Structure

#### `src/App.tsx` (Main Application Component)
- **Purpose**: Root application component managing onboarding flow and main settings interface
- **Key Features**:
  - Onboarding flow for first-time users (model selection)
  - Settings interface with sidebar navigation
  - Debug mode toggle via Ctrl+Shift+D (or Cmd+Shift+D on macOS)
  - Checks for available models to determine onboarding state
- **Dependencies**: Uses `useSettings` hook, `Sidebar` component, and Tauri commands

#### `src/components/Sidebar.tsx` (Navigation Component)
- **Purpose**: Left sidebar navigation for different settings sections
- **Sections Configuration**:
  - General (`GeneralSettings`)
  - Advanced (`AdvancedSettings`)
  - History (`HistorySettings`)
  - Debug (`DebugSettings`) - only visible when debug mode enabled
  - About (`AboutSettings`)
- **Dynamic Display**: Sections can be conditionally enabled based on settings

### State Management

#### `src/hooks/useSettings.ts` (Settings Hook)
- **Purpose**: React hook providing settings state and operations
- **Features**:
  - Settings CRUD operations
  - Audio device management (input/output)
  - Keyboard shortcut binding management
  - Loading states and error handling
- **Backend Integration**: Uses Zustand store with Tauri command integration

#### `src/stores/settingsStore.ts` (Zustand Store)
- **Purpose**: Central state management for application settings
- **Key Features**:
  - Optimistic updates with rollback on error
  - Tauri plugin store integration for persistence
  - Audio device enumeration
  - Settings synchronization with backend
- **Settings Categories**:
  - Audio settings (microphone, output devices, feedback)
  - Keyboard shortcuts and bindings
  - Model selection and preferences
  - UI preferences (overlay position, debug mode)
  - PII redaction configuration

#### `src/hooks/useModels.ts` (Model Management Hook)
- **Purpose**: Manages Whisper model downloading, selection, and status
- **Features**:
  - Model download with progress tracking
  - Model selection and switching
  - First-run detection and onboarding
  - Real-time download progress via Tauri events
- **Event Handling**: Listens to `model-download-progress`, `model-download-complete`, `model-extraction-started`, etc.

### Component Categories

#### Settings Components (`src/components/settings/`)
Each settings component manages a specific aspect of application configuration:

- **`GeneralSettings.tsx`**: Core transcription settings (model, language, shortcuts)
- **`AdvancedSettings.tsx`**: Technical settings (VAD, timeouts, custom words)
- **`HistorySettings.tsx`**: Transcription history management
- **`DebugSettings.tsx`**: Development and diagnostic tools
- **`PIIRedaction.tsx`**: Privacy settings for PII detection and redaction

#### Model Management (`src/components/model-selector/`)
- **`ModelSelector.tsx`**: Main model selection interface
- **`ModelDropdown.tsx`**: Dropdown for selecting available models
- **`ModelStatusButton.tsx`**: Shows current model status and actions
- **`DownloadProgressDisplay.tsx`**: Real-time download progress visualization

#### UI Components (`src/components/ui/`)
Reusable UI building blocks:
- **`Button.tsx`**, **`Input.tsx`**, **`Slider.tsx`**: Form elements
- **`Dropdown.tsx`**: Custom dropdown component
- **`ToggleSwitch.tsx`**: Boolean setting toggles
- **`SettingContainer.tsx`**, **`SettingsGroup.tsx`**: Layout containers
- **`InfoTooltip.tsx`**: Help tooltips for settings

### Type Definitions

#### `src/lib/types.ts` (TypeScript Types)
Comprehensive type definitions using Zod for runtime validation:
- **`Settings`**: Complete application settings schema
- **`AudioDevice`**: Audio device representation
- **`ShortcutBinding`**: Keyboard shortcut configuration
- **`ModelInfo`**: Model metadata and status
- **`OverlayPosition`**, **`ModelUnloadTimeout`**: Enum types

## Backend Architecture (src-tauri/src/)

### Application Entry Point

#### `src-tauri/src/lib.rs` (Main Application Setup)
- **Purpose**: Tauri application initialization and configuration
- **Key Responsibilities**:
  - Plugin initialization (filesystem, audio, shortcuts, etc.)
  - Manager setup (Audio, Model, Transcription, History, PII)
  - Tray icon and system integration
  - Window management and single-instance enforcement
  - Command handler registration

### Core Managers (`src-tauri/src/managers/`)

#### `managers/audio.rs` (Audio Recording Manager)
- **Purpose**: Manages audio recording lifecycle and microphone access
- **Key Features**:
  - Two modes: AlwaysOn vs OnDemand microphone access
  - VAD integration for speech detection
  - Audio level monitoring and frontend updates
  - Device enumeration and selection
- **Dependencies**: Uses `audio_toolkit` for low-level audio operations

#### `managers/model.rs` (Model Management)
- **Purpose**: Handles Whisper model downloading, storage, and availability
- **Model Support**: Small, Medium, Turbo, and Large Whisper variants
- **Download Features**:
  - Progress tracking with frontend events
  - Resumable downloads (partial_size tracking)
  - Archive extraction for compressed models
  - Model validation and storage in app data directory

#### `managers/transcription.rs` (Transcription Pipeline)
- **Purpose**: Coordinates speech-to-text processing pipeline
- **Pipeline**: Audio → VAD → Whisper → Post-processing → Output
- **Features**:
  - Model loading and unloading with timeout management
  - Custom word correction and language detection
  - Integration with PII redaction
  - Text output to system clipboard

#### `managers/history.rs` (History Management)
- **Purpose**: SQLite-based transcription history storage
- **Features**:
  - Transcription entry persistence
  - Audio file storage and retrieval
  - Search and filtering capabilities
  - Entry metadata (timestamps, model used, etc.)

### Audio Processing (`src-tauri/src/audio_toolkit/`)

#### `audio/mod.rs` (Audio Subsystem)
- **Components**:
  - **`recorder.rs`**: Core audio recording with CPAL
  - **`device.rs`**: Audio device enumeration and management
  - **`resampler.rs`**: Audio format conversion for Whisper
  - **`visualizer.rs`**: Real-time audio level calculation

#### `vad/mod.rs` (Voice Activity Detection)
- **`silero.rs`**: Silero VAD model integration for speech detection
- **`smoothed.rs`**: Smoothing wrapper to reduce false positives
- **Integration**: Filters audio before sending to Whisper for efficiency

### Command Interface (`src-tauri/src/commands/`)

#### Command Organization
- **`audio.rs`**: Audio device and recording commands
- **`models.rs`**: Model management commands (download, select, delete)
- **`transcription.rs`**: Transcription control and status
- **`history.rs`**: History CRUD operations
- **`pii.rs`**: PII redaction configuration

### Privacy Features

#### `src-tauri/src/pii_redactor.rs` (PII Redaction)
- **Purpose**: Detects and redacts personally identifiable information
- **Technology**: GLiNER model for named entity recognition
- **Entity Categories**:
  - Personal Identifiers (names, DOB, age, gender)
  - Contact Information (email, phone, address)
  - Financial Data (credit cards, bank accounts)
  - Medical Information (diagnoses, medications)
  - Government IDs (SSN, license numbers)
- **Features**: Configurable entity types, optional entity labels

### System Integration

#### `src-tauri/src/shortcut.rs` (Global Shortcuts)
- **Purpose**: System-wide keyboard shortcut handling
- **Features**:
  - Configurable key bindings
  - Push-to-talk vs toggle modes
  - Shortcut conflict detection
  - Dynamic binding updates

#### `src-tauri/src/tray.rs` (System Tray)
- **Purpose**: System tray integration with context menu
- **Features**:
  - Theme-aware icons (light/dark mode)
  - Status indication (idle/recording/processing)
  - Quick actions (settings, cancel, quit)

#### `src-tauri/src/settings.rs` (Settings Persistence)
- **Purpose**: Application settings storage and retrieval
- **Storage**: Tauri plugin-store for JSON persistence
- **Features**:
  - Default value handling
  - Settings validation
  - Cross-platform configuration paths

## Data Flow and Communication

### Frontend ↔ Backend Communication

#### Tauri Commands (Frontend → Backend)
- **Settings**: `change_binding`, `update_microphone_mode`, `set_pii_redaction_enabled`
- **Models**: `download_model`, `set_active_model`, `get_available_models`
- **Audio**: `get_available_microphones`, `set_selected_microphone`
- **History**: `get_history_entries`, `delete_history_entry`

#### Tauri Events (Backend → Frontend)
- **Model Progress**: `model-download-progress`, `model-download-complete`
- **Audio Levels**: `audio-levels` for real-time visualization
- **Transcription**: `transcription-result`, `transcription-error`

### Processing Pipeline

1. **Audio Input**: Microphone → CPAL → Audio Recorder
2. **Voice Detection**: Raw Audio → VAD → Speech Segments
3. **Transcription**: Speech Audio → Whisper Model → Raw Text
4. **Post-Processing**: Raw Text → Custom Words → PII Redaction → Final Text
5. **Output**: Final Text → Clipboard → Active Application

## Key Design Patterns

### Manager Pattern
Core functionality is organized into managers that:
- Initialize during app startup
- Maintain internal state
- Expose commands for frontend communication
- Handle resource lifecycle (loading/unloading)

### Event-Driven Architecture
- Real-time updates via Tauri events
- Decoupled communication between components
- Progress tracking for long-running operations

### Optimistic Updates
- Frontend immediately reflects user actions
- Backend operations run asynchronously
- Rollback on failure for consistency

### Resource Management
- Model loading/unloading with timeout policies
- Audio stream lifecycle management
- Memory-conscious operation for long-running sessions

## External Dependencies

### Key Rust Crates
- **`whisper-rs`**: Local Whisper inference with GPU acceleration
- **`cpal`**: Cross-platform audio I/O
- **`vad-rs`**: Voice Activity Detection
- **`rdev`**: Global keyboard shortcuts
- **`tauri`**: Desktop app framework
- **`gliner`**: PII detection via GLiNER model

### Key NPM Packages
- **`@tauri-apps/api`**: Frontend-backend communication
- **`react`**: UI framework
- **`zustand`**: State management
- **`lucide-react`**: Icon library
- **`zod`**: Runtime type validation

## Development Considerations

### Platform-Specific Features
- **macOS**: Metal acceleration, accessibility permissions, app activation policies
- **Windows**: Vulkan acceleration, code signing requirements
- **Linux**: OpenBLAS + Vulkan acceleration

### Performance Optimizations
- Model lazy loading with configurable timeouts
- Audio processing in separate threads
- Efficient VAD to minimize Whisper calls
- Progress streaming for large downloads

### Privacy and Security
- Local processing (no cloud dependencies)
- Configurable PII redaction
- Secure settings storage
- No telemetry or data collection

This architecture enables Handy to provide efficient, private, and reliable speech-to-text functionality while maintaining a responsive user experience and broad platform compatibility.