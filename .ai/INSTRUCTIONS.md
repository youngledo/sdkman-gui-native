# SDKMAN GUI - AI Development Guide

## Project Overview

This is a cross-platform GUI application for SDKMAN, built with **Tauri 2.0 + Vue 3 + Rust**.

**Key Features**:
- Sidebar navigation: Home, JDK, SDK, Settings pages
- Modern UI with custom CSS variables and theming
- Full internationalization support (Chinese & English)
- Async operations with Rust backend

## Technology Stack

- **Frontend**: Vue 3 + TypeScript + Vite
- **Backend**: Rust + Tauri 2.0
- **UI**: Custom components with CSS variables
- **State Management**: Pinia
- **Internationalization**: Vue-i18n
- **Router**: Vue Router

## Architecture

### Frontend (Vue 3)

```
sdkman-gui-tauri/src/
├── components/           # Reusable UI components
│   ├── Sidebar.vue       # Navigation sidebar
│   ├── SdkCard.vue       # SDK display card
│   └── ...
├── views/                # Page components
│   ├── Home.vue          # Home page with statistics
│   ├── Jdk.vue           # JDK management page
│   ├── Sdk.vue           # SDK browsing page
│   ├── SdkDetail.vue     # SDK version details
│   └── Settings.vue      # Settings page
├── stores/               # Pinia state management
│   └── sdkStore.ts       # SDK data store
├── locales/              # i18n translations
│   ├── en.json           # English translations
│   └── zh.json           # Chinese translations
├── router/               # Vue Router configuration
│   └── index.ts
└── App.vue               # Root component with theme management
```

### Backend (Rust)

```
sdkman-gui-tauri/src-tauri/src/
├── main.rs               # Application entry point
├── lib.rs                # Library exports
├── sdkman.rs             # SDKMAN CLI wrapper
├── config.rs             # Configuration management
└── ...
```

## 🚨 CRITICAL DEVELOPMENT RULES 🚨

### 1. Directory Structure Standards - MUST FOLLOW ⚠️

**Golden Rule: All application configuration and cache data MUST be stored in the unified config directory!**

#### Application Data Storage Location

**macOS/Linux**:
```
~/.config/sdkman-gui/          # ✅ Unified config directory
├── config.json                # Application config file
└── cache/                     # Cache directory
    ├── jdk_versions.json      # JDK versions list cache
    ├── sdk_candidates.json    # SDK candidates list cache
    └── {candidate}_versions.json  # Individual SDK version list cache
```

**Windows**:
```
C:\Users\<username>\.config\sdkman-gui\
├── config.json
└── cache\
    ├── jdk_versions.json
    ├── sdk_candidates.json
    └── {candidate}_versions.json
```

#### ❌ FORBIDDEN Paths

```rust
// ❌ NEVER use these paths
let path = home.join(".sdkman-gui");           // FORBIDDEN!
let path = home.join(".cache/sdkman-gui");     // FORBIDDEN!
let path = home.join("Library/Application Support/sdkman-gui");  // FORBIDDEN!
```

#### ✅ CORRECT Path Access

```rust
// ✅ Config file path
use crate::config::AppConfig;
let config = AppConfig::load()?;  // Auto uses ~/.config/sdkman-gui/config.json

// ✅ Cache directory path
use crate::cache;
let cache_dir = cache::get_cache_dir()?;  // ~/.config/sdkman-gui/cache/
```

#### Steps for Adding New Cache Files

1. Add file name constant in `src-tauri/src/cache.rs`:
```rust
const NEW_CACHE_FILE: &str = "new_cache.json";
```

2. Implement cache functions (MUST use `get_cache_dir()`):
```rust
pub fn cache_new_data(data: &[SomeType]) -> Result<()> {
    let cache_dir = get_cache_dir()?;  // ✅ Use unified dir function
    let cache_file = cache_dir.join(NEW_CACHE_FILE);
    // Implementation...
}

pub fn read_new_data_cache() -> Result<Option<Vec<SomeType>>> {
    let cache_dir = get_cache_dir()?;  // ✅ Use unified dir function
    let cache_file = cache_dir.join(NEW_CACHE_FILE);
    // Implementation...
}
```

#### Cache Strategy Standards

All network-fetched data should support caching:

```rust
#[tauri::command]
pub async fn get_some_data(
    force_refresh: bool,  // ✅ MUST support this parameter
    client: State<'_, Arc<Mutex<SdkmanApiClient>>>
) -> Result<Vec<SomeType>, String> {
    // Prioritize cache
    if !force_refresh {
        if let Ok(Some(cached)) = cache::read_some_data_cache() {
            println!("Using cached data");
            return Ok(cached);
        }
    }

    // Fetch from API and cache
    println!("Fetching from API");
    let data = fetch_from_api(&client).await?;
    let _ = cache::cache_some_data(&data);  // Cache result
    Ok(data)
}
```

Frontend calling convention:
```typescript
// ✅ Initial load uses cache
onMounted(async () => {
    await fetchData(false)  // forceRefresh = false
})

// ✅ Refresh button forces update
async function onRefresh() {
    await fetchData(true)   // forceRefresh = true
}
```

### 2. Internationalization (I18n) - MUST FOLLOW

**Golden Rule: ALL user-facing text MUST use i18n. NO hardcoded strings!**

#### Vue Template Usage

```vue
<template>
  <!-- ❌ WRONG -->
  <h1>Welcome to SDKMAN</h1>
  <button>Install</button>

  <!-- ✅ CORRECT -->
  <h1>{{ $t('home.welcome') }}</h1>
  <button>{{ $t('jdk.action.install') }}</button>
</template>
```

#### Adding New Translations

1. Add to `src/locales/en.json`:
```json
{
  "home": {
    "welcome": "Welcome to SDKMAN"
  },
  "jdk": {
    "action": {
      "install": "Install"
    }
  }
}
```

2. Add to `src/locales/zh.json`:
```json
{
  "home": {
    "welcome": "欢迎使用SDKMAN"
  },
  "jdk": {
    "action": {
      "install": "安装"
    }
  }
}
```

### 2. Code Style & Conventions

#### TypeScript Naming
- Components: `PascalCase` (SdkCard.vue, HomeView.vue)
- Functions: `camelCase` (loadStatistics, setupTheme)
- Variables: `camelCase` (sdkList, isLoading)
- Constants: `UPPER_SNAKE_CASE` (DEFAULT_LOCALE, MAX_RETRY)

#### Chinese-English Spacing
**NO spaces between Chinese and English characters** (user's explicit requirement)

```typescript
// ✅ CORRECT
const message = "欢迎使用SDKMAN"

// ❌ WRONG
const message = "欢迎使用 SDKMAN"
```

#### Vue Component Structure

```vue
<template>
  <!-- Template content -->
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

// Reactive state
const isLoading = ref(false)

// Functions
async function loadData() {
  // ...
}

// Lifecycle
onMounted(() => {
  loadData()
})
</script>

<style scoped>
/* Component styles */
</style>
```

### 3. Tauri Command Integration

#### Defining Rust Commands

```rust
// src-tauri/src/lib.rs
#[tauri::command]
async fn get_sdk_list() -> Result<Vec<SdkInfo>, String> {
    // Implementation
}

// Register in builder
.invoke_handler(tauri::generate_handler![get_sdk_list])
```

#### Calling from Vue

```typescript
import { invoke } from '@tauri-apps/api/core'

async function fetchSdkList() {
  try {
    const list = await invoke<SdkInfo[]>('get_sdk_list')
    return list
  } catch (e) {
    console.error('Failed to fetch SDK list:', e)
    throw e
  }
}
```

### 4. Theme Management

The app supports three theme modes: light, dark, and auto.

#### CSS Variables

```css
:root {
  --bg-primary: #ffffff;
  --bg-secondary: #f5f5f5;
  --text-primary: #1a1a1a;
  --text-secondary: #666666;
  --primary-color: #007aff;
  /* ... more variables */
}

[data-theme="dark"] {
  --bg-primary: #1a1a1a;
  --bg-secondary: #2d2d2d;
  --text-primary: #ffffff;
  --text-secondary: #a0a0a0;
  --primary-color: #0a84ff;
}
```

#### Using Theme Variables

```vue
<style scoped>
.card {
  background: var(--bg-secondary);
  color: var(--text-primary);
}
</style>
```

### 5. State Management with Pinia

```typescript
// stores/sdkStore.ts
import { defineStore } from 'pinia'

export const useSdkStore = defineStore('sdk', {
  state: () => ({
    sdkList: [] as SdkInfo[],
    isLoading: false,
    error: null as string | null,
  }),

  actions: {
    async fetchSdkList() {
      this.isLoading = true
      try {
        this.sdkList = await invoke<SdkInfo[]>('get_sdk_list')
      } catch (e) {
        this.error = String(e)
      } finally {
        this.isLoading = false
      }
    }
  }
})
```

### 6. Error Handling

#### Frontend

```typescript
try {
  const result = await invoke('some_command')
  // Handle success
} catch (e) {
  console.error('Operation failed:', e)
  // Show user-friendly error notification
}
```

#### Backend (Rust)

```rust
#[tauri::command]
async fn install_sdk(candidate: String, version: String) -> Result<(), String> {
    // Use Result type for proper error handling
    execute_sdkman_command(&format!("sdk install {} {}", candidate, version))
        .map_err(|e| e.to_string())
}
```

### 7. SDKMAN CLI Integration

All SDKMAN commands are executed through the Rust backend:

```rust
// Execute SDKMAN command
fn execute_sdkman_command(command: &str) -> Result<String, Box<dyn Error>> {
    let full_command = format!(
        "source ~/.sdkman/bin/sdkman-init.sh && {}",
        command
    );

    let output = Command::new("bash")
        .args(["-c", &full_command])
        .output()?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
```

Common SDKMAN commands:
- `sdk list` - List all candidates
- `sdk list java` - List Java versions
- `sdk install java 21` - Install Java 21
- `sdk default java 21` - Set default Java
- `sdk uninstall java 21` - Uninstall Java 21

## Development Workflow

### Running Development Mode

```bash
cd sdkman-gui-tauri
npm install
npm run tauri dev
```

### Building for Production

```bash
npm run tauri build
```

### Project Structure

```
sdkman-gui/
├── sdkman-gui-tauri/          # Main Tauri application
│   ├── src/                   # Vue 3 frontend
│   ├── src-tauri/             # Rust backend
│   ├── package.json           # Node.js dependencies
│   └── vite.config.ts         # Vite configuration
├── docs/                      # Documentation and images
├── README.md                  # English README
├── README_ZH.md               # Chinese README
└── AI_GUIDE.md                # This file
```

## Common Anti-Patterns to Avoid

### ❌ Hardcoding Text
```vue
<template>
  <span>欢迎</span>  <!-- NEVER DO THIS -->
</template>
```

### ❌ Blocking UI with Sync Operations
```typescript
// WRONG - blocks UI
const result = someHeavyOperation()
```

### ❌ Ignoring Error Handling
```typescript
// WRONG - no error handling
const data = await invoke('get_data')
```

### ❌ Using console.log in Production
```typescript
console.log("Debug info")  // Remove in production
```

## Key Files Reference

**Configuration Files:**
- `sdkman-gui-tauri/tauri.conf.json` - Tauri configuration
- `sdkman-gui-tauri/package.json` - Node.js dependencies
- `sdkman-gui-tauri/vite.config.ts` - Vite build configuration
- `sdkman-gui-tauri/tsconfig.json` - TypeScript configuration

**Main Entry Points:**
- `sdkman-gui-tauri/src/main.ts` - Vue application entry
- `sdkman-gui-tauri/src-tauri/src/main.rs` - Rust entry point

**Key Components:**
- `src/App.vue` - Root component with theme management
- `src/views/Home.vue` - Home page
- `src/views/Jdk.vue` - JDK management
- `src/views/Settings.vue` - Settings page

---

**Remember: This AI_GUIDE.md file should be referenced when working on this project. Always follow these rules to maintain code quality and consistency!**
