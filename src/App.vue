<template>
  <div :class="['app-container', themeClass]">
    <MainLayout />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, provide } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import MainLayout from './components/layout/MainLayout.vue'
import { useInstallProgressStore } from './stores/installProgressStore'

type ThemeType = 'light' | 'dark' | 'auto'

const { locale } = useI18n()

const theme = ref<ThemeType>('auto')
const systemPrefersDark = ref(false)

// Initialize install progress store
const installProgressStore = useInstallProgressStore()

// Computed theme class based on current settings
const themeClass = computed(() => {
  if (theme.value === 'auto') {
    return systemPrefersDark.value ? 'theme-dark' : 'theme-light'
  }
  return theme.value === 'dark' ? 'theme-dark' : 'theme-light'
})

// Detect system theme preference
function detectSystemTheme() {
  if (window.matchMedia) {
    systemPrefersDark.value = window.matchMedia('(prefers-color-scheme: dark)').matches
  }
}

// Listen for system theme changes
function setupSystemThemeListener() {
  if (window.matchMedia) {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    mediaQuery.addEventListener('change', (e) => {
      systemPrefersDark.value = e.matches
    })
  }
}

// Load configuration (theme and language)
async function loadConfig() {
  try {
    const config = await invoke<any>('load_config')
    theme.value = config.theme || 'auto'

    // 应用语言设置
    // 如果是 'auto' 或未设置，保持使用系统检测的语言（已在 i18n/index.ts 中设置）
    // 否则使用用户保存的语言偏好
    if (config.language && config.language !== 'auto') {
      locale.value = config.language
    }
  } catch (e) {
    console.error('Failed to load config:', e)
  }
}

// Update theme
function setTheme(newTheme: ThemeType) {
  theme.value = newTheme
}

// Provide theme control to child components
provide('theme', theme)
provide('setTheme', setTheme)

onMounted(async () => {
  detectSystemTheme()
  setupSystemThemeListener()
  await loadConfig()
  // Initialize global install progress event listeners
  try {
    await installProgressStore.initEventListeners()
    console.log('[App] Install progress event listeners initialized')
  } catch (error) {
    console.error('[App] Failed to initialize install progress event listeners:', error)
  }
})

onUnmounted(async () => {
  // Cleanup global event listeners when app is destroyed
  await installProgressStore.cleanupEventListeners()
})
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
}

.app-container {
  width: 100%;
  height: 100vh;
  transition: background-color 0.3s, color 0.3s;
}

/* ⚠️警告：千万不要删除看似未使用的样式 */
/* Light theme (default) */
.theme-light {
  --bg-primary: #f3f4f6;
  --bg-secondary: #ffffff;
  --bg-tertiary: #f9fafb;
  --text-primary: #1f2937;
  --text-secondary: #6b7280;
  --text-tertiary: #9ca3af;
  --border-color: #e5e7eb;
  --border-color-light: #f3f4f6;
  --shadow: rgba(0, 0, 0, 0.1);
  --shadow-hover: rgba(0, 0, 0, 0.15);
  --primary-color: #2563eb;
  --primary-hover: #1d4ed8;
  --success-color: #10b981;
  --success-hover: #059669;
  --danger-color: #ef4444;
  --danger-hover: #dc2626;
  --warning-color: #f59e0b;
  --gray-color: #6b7280;
  --gray-hover: #4b5563;
}

/* ⚠️警告：千万不要删除看似未使用的样式 */
/* Dark theme */
.theme-dark {
  --bg-primary: #111827;
  --bg-secondary: #1f2937;
  --bg-tertiary: #374151;
  --text-primary: #f9fafb;
  --text-secondary: #d1d5db;
  --text-tertiary: #9ca3af;
  --border-color: #374151;
  --border-color-light: #4b5563;
  --shadow: rgba(0, 0, 0, 0.3);
  --shadow-hover: rgba(0, 0, 0, 0.5);
  --primary-color: #3b82f6;
  --primary-hover: #2563eb;
  --success-color: #10b981;
  --success-hover: #059669;
  --danger-color: #ef4444;
  --danger-hover: #dc2626;
  --warning-color: #f59e0b;
  --gray-color: #9ca3af;
  --gray-hover: #d1d5db;
}

.theme-dark .app-container {
  background-color: var(--bg-primary);
  color: var(--text-primary);
}

.theme-light .app-container {
  background-color: var(--bg-primary);
  color: var(--text-primary);
}
</style>
