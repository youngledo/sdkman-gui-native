<template>
  <div class="page-content">
    <!-- 页面头部 -->
    <div class="page-header">
      <h1 class="page-title">{{ $t('settings.title') }}</h1>
      <p class="page-subtitle">{{ $t('settings.subtitle') }}</p>
    </div>

    <!-- 设置内容 -->
    <div class="settings-container">
      <!-- 主题设置 -->
      <div class="settings-section">
        <h3 class="section-title">{{ $t('settings.theme') }}</h3>
        <div class="radio-group">
          <label class="radio-label">
            <input type="radio" v-model="theme" value="light" />
            <span>{{ $t('settings.themeLight') }}</span>
          </label>
          <label class="radio-label">
            <input type="radio" v-model="theme" value="dark" />
            <span>{{ $t('settings.themeDark') }}</span>
          </label>
          <label class="radio-label">
            <input type="radio" v-model="theme" value="auto" />
            <span>{{ $t('settings.themeAuto') }}</span>
          </label>
        </div>
      </div>

      <!-- 语言设置 -->
      <div class="settings-section">
        <h3 class="section-title">{{ $t('settings.language') }}</h3>
        <div class="radio-group">
          <label class="radio-label">
            <input type="radio" v-model="language" value="auto" />
            <span>{{ $t('settings.languageAuto') }}</span>
          </label>
          <label class="radio-label">
            <input type="radio" v-model="language" value="en" />
            <span>{{ $t('settings.languageEnglish') }}</span>
          </label>
          <label class="radio-label">
            <input type="radio" v-model="language" value="zh" />
            <span>{{ $t('settings.languageChinese') }}</span>
          </label>
        </div>
      </div>

      <!-- 代理设置 -->
      <div class="settings-section">
        <h3 class="section-title">{{ $t('settings.proxy') }}</h3>
        <div class="radio-group">
          <label class="radio-label">
            <input type="radio" v-model="proxyType" value="none" />
            <span>{{ $t('settings.proxyNone') }}</span>
          </label>
          <label class="radio-label">
            <input type="radio" v-model="proxyType" value="auto" />
            <span>{{ $t('settings.proxyAuto') }}</span>
          </label>
          <label class="radio-label">
            <input type="radio" v-model="proxyType" value="manual" />
            <span>{{ $t('settings.proxyManual') }}</span>
          </label>
        </div>

        <!-- 手动代理配置 -->
        <div v-if="proxyType === 'manual'" class="proxy-manual-config">
          <div class="form-row">
            <label class="form-label">{{ $t('settings.proxyHost') }}:</label>
            <input type="text" v-model="proxyHost" class="form-input" :placeholder="$t('settings.proxyHostPlaceholder')" />
          </div>
          <div class="form-row">
            <label class="form-label">{{ $t('settings.proxyPort') }}:</label>
            <input type="text" v-model="proxyPort" class="form-input" :placeholder="$t('settings.proxyPortPlaceholder')" />
          </div>
        </div>
      </div>

      <!-- SDKMAN 路径设置 -->
      <div class="settings-section">
        <h3 class="section-title">{{ $t('settings.sdkmanPath') }}</h3>
        <div class="path-input-group">
          <input type="text" v-model="sdkmanPath" class="form-input" readonly />
          <button class="browse-button" @click="browsePath">{{ $t('settings.sdkmanBrowse') }}</button>
        </div>
      </div>

      <!-- 应用版本 -->
      <div class="settings-section">
        <h3 class="section-title">{{ $t('settings.appVersion') }}</h3>
        <div class="version-info">
          <div class="version-left">
            <p><strong>{{ $t('settings.currentVersion') }}:</strong> {{ appVersion }}</p>
            <button class="check-update-button" @click="checkForUpdates" :disabled="checking">
              {{ $t('settings.checkUpdate') }}
            </button>
          </div>
          <div class="version-right">
            <span v-if="checking" class="update-status checking">{{ $t('settings.checking') }}</span>
            <span v-else-if="updateStatus" class="update-status">{{ updateStatus }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, inject } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

const { t, locale } = useI18n()

// Inject theme control from App.vue
const injectedSetTheme = inject<(theme: string) => void>('setTheme')

// 设置状态
const theme = ref('auto')
const language = ref('en')
const proxyType = ref('none')
const proxyHost = ref('')
const proxyPort = ref('')
const sdkmanPath = ref('~/.sdkman')
const appVersion = ref('1.0.0')

// UI 状态
const checking = ref(false)
const updateStatus = ref('')

// 加载设置
async function loadSettings() {
  try {
    const config = await invoke<any>('load_config')
    console.log('Loaded config:', config)

    theme.value = config.theme || 'auto'
    language.value = config.language || 'auto'
    proxyType.value = config.proxy_type || 'none'
    proxyHost.value = config.proxy_host || ''
    proxyPort.value = config.proxy_port || ''

    // 从后端获取真实的SDKMAN路径
    try {
      sdkmanPath.value = await invoke<string>('get_sdkman_path')
    } catch (e) {
      console.error('Failed to get SDKMAN path:', e)
      sdkmanPath.value = config.sdkman_path || '~/.sdkman'
    }

    // 应用语言设置
    if (language.value === 'auto') {
      // 如果是自动模式，检测系统语言
      const browserLang = navigator.language || (navigator as any).userLanguage
      locale.value = browserLang.toLowerCase().startsWith('zh') ? 'zh' : 'en'
    } else {
      locale.value = language.value
    }
  } catch (e) {
    console.error('Failed to load settings:', e)
  }
}

// 自动保存设置
async function saveSettings() {
  try {
    const config = {
      theme: theme.value,
      language: language.value,
      proxy_type: proxyType.value,
      proxy_host: proxyHost.value || null,
      proxy_port: proxyPort.value || null,
      sdkman_path: sdkmanPath.value,
    }

    console.log('Saving config:', config)
    await invoke('save_config', { config })

    // 立即应用主题
    applyTheme()
  } catch (e) {
    console.error('Failed to save settings:', e)
  }
}

// 应用主题
function applyTheme() {
  if (injectedSetTheme) {
    injectedSetTheme(theme.value)
  }
}

// 浏览路径
async function browsePath() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: t('settings.selectSdkmanPath')
  })

  if (selected) {
    sdkmanPath.value = selected as string
  }
}

// 检查更新
async function checkForUpdates() {
  checking.value = true
  updateStatus.value = ''

  try {
    // 模拟检查更新
    await new Promise(resolve => setTimeout(resolve, 1500))
    updateStatus.value = t('settings.upToDate')
  } catch (e) {
    updateStatus.value = t('settings.checkUpdateFailed')
  } finally {
    checking.value = false
  }
}

// 监听设置变化，实时保存到后端
watch([theme, language, proxyType, proxyHost, proxyPort, sdkmanPath], () => {
  saveSettings()
})

// 监听语言变化，实时切换
watch(language, async (newLang) => {
  if (newLang === 'auto') {
    // 如果选择自动，检测系统语言
    const browserLang = navigator.language || (navigator as any).userLanguage
    locale.value = browserLang.toLowerCase().startsWith('zh') ? 'zh' : 'en'
  } else {
    locale.value = newLang
  }
  // 更新托盘菜单语言
  const { updateTrayLanguage } = await import('../i18n')
  updateTrayLanguage(locale.value)
})

onMounted(async () => {
  await loadSettings()
})
</script>

<style scoped>
/* Settings page styles */
.page-content {
  padding: 40px;
  max-width: 1400px;
  margin: 0 auto;
}

.page-header {
  margin-bottom: 32px;
}

.page-title {
  font-size: 32px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0 0 8px 0;
}

.page-subtitle {
  font-size: 16px;
  color: var(--text-secondary);
  margin: 0;
}

.settings-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.settings-section {
  background: var(--bg-secondary);
  border-radius: 12px;
  padding: 20px 24px;
  box-shadow: 0 1px 3px var(--shadow);
}

.section-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 16px 0;
}

.radio-group {
  display: flex;
  gap: 24px;
}

.radio-label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-size: 15px;
  color: var(--text-primary);
}

.radio-label input[type="radio"] {
  width: 18px;
  height: 18px;
  cursor: pointer;
}

.proxy-manual-config {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid #e5e7eb;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.form-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.form-label {
  min-width: 120px;
  font-size: 14px;
  color: var(--text-secondary);
  font-weight: 500;
}

.form-input {
  flex: 1;
  padding: 10px 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 14px;
  transition: border-color 0.2s;
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.form-input:focus {
  outline: none;
  border-color: var(--primary-color);
}

.form-input:read-only {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
}

.path-input-group {
  display: flex;
  gap: 12px;
}

.browse-button {
  padding: 10px 20px;
  background: var(--gray-color);
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s;
  white-space: nowrap;
}

.browse-button:hover {
  background: var(--gray-hover);
}

.version-info {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.version-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.version-right {
  display: flex;
  align-items: center;
  min-width: 200px;
  justify-content: flex-end;
}

.version-info p {
  margin: 0;
  font-size: 15px;
  color: var(--text-primary);
}

.check-update-button {
  padding: 8px 16px;
  background: var(--bg-secondary);
  color: var(--primary-color);
  border: 1px solid var(--primary-color);
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.check-update-button:hover:not(:disabled) {
  background: var(--primary-color);
  color: white;
}

.check-update-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.update-status {
  font-size: 14px;
  color: var(--success-color);
}
</style>