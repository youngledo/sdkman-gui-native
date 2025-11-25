<template>
  <div class="page-content">
    <div class="page-header">
      <div class="welcome-text">
        <h1 class="page-title">
          {{ $t('home.welcomePrefix') }} <span class="app-name" @click="openGithub">SDKMAN GUI</span>{{ $t('home.welcomeMark') }}
        </h1>
        <p class="page-subtitle">{{ $t('home.subtitle') }}</p>
      </div>
    </div>

    <!-- 统计卡片 -->
    <div class="stats-container">
      <div class="stat-card">
        <svg class="stat-icon coffee" width="48" height="48" viewBox="0 0 24 24" fill="currentColor">
          <path d="M2,21H20V19H2M20,8H18V5H20M20,3H4V13A4,4 0 0,0 8,17H14A4,4 0 0,0 18,13V10H20A2,2 0 0,0 22,8V5C22,3.89 21.1,3 20,3Z"/>
        </svg>
        <div class="stat-number">{{ statistics.jdk_installed }}</div>
        <div class="stat-label">{{ $t('home.statJdk') }}</div>
      </div>

      <div class="stat-card">
        <svg class="stat-icon package" width="48" height="48" viewBox="0 0 24 24" fill="currentColor">
          <path d="M21 16.5c0 .38-.21.71-.53.88l-7.9 4.44c-.16.12-.36.18-.57.18-.21 0-.41-.06-.57-.18l-7.9-4.44A.99.99 0 0 1 3 16.5v-9c0-.38.21-.71.53-.88l7.9-4.44c.16-.12.36-.18.57-.18.21 0 .41.06.57.18l7.9 4.44c.32.17.53.5.53.88v9M12 4.15L6.04 7.5 12 10.85l5.96-3.35L12 4.15M5 15.91l6 3.38v-6.71L5 9.21v6.7m14 0v-6.7l-6 3.37v6.71l6-3.38z"/>
        </svg>
        <div class="stat-number">{{ statistics.sdk_installed }}</div>
        <div class="stat-label">{{ $t('home.statSdk') }}</div>
      </div>

      <div class="stat-card">
        <svg class="stat-icon refresh" width="48" height="48" viewBox="0 0 24 24" fill="currentColor">
          <path d="M17.65 6.35A7.958 7.958 0 0 0 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08A5.99 5.99 0 0 1 12 18c-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
        </svg>
        <div class="stat-number">{{ availableUpdates }}</div>
        <div class="stat-label">{{ $t('home.statUpdates') }}</div>
      </div>
    </div>

    <!-- 快速操作 -->
    <div class="quick-actions-section">
      <h2 class="section-title">{{ $t('home.quickActions') }}</h2>
      <div class="action-buttons">
        <button class="action-button" @click="navigateTo('/jdk')">
          {{ $t('home.actionBrowseJdk') }}
        </button>
        <button class="action-button" @click="navigateTo('/sdk')">
          {{ $t('home.actionBrowseSdk') }}
        </button>
        <button class="action-button" @click="checkUpdates">
          {{ $t('home.actionCheckUpdate') }}
        </button>
      </div>
    </div>

    <!-- 底部提示 -->
    <div class="hint-text">
      {{ $t('home.hint') }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useSdkStore } from '../stores/sdkStore'
import { open } from '@tauri-apps/plugin-shell'

const router = useRouter()
const sdkStore = useSdkStore()

const statistics = computed(() => sdkStore.statistics)
const availableUpdates = computed(() => {
  return statistics.value.jdk_available - statistics.value.jdk_installed +
         statistics.value.sdk_available - statistics.value.sdk_installed
})

const navigateTo = (path: string) => {
  router.push(path)
}

const checkUpdates = async () => {
  await sdkStore.fetchStatistics()
  await sdkStore.fetchJdkVersions()
}

const openGithub = async () => {
  try {
    await open('https://github.com/youngledo/sdkman-gui-native')
  } catch (error) {
    console.error('Failed to open URL:', error)
  }
}

onMounted(async () => {
  await sdkStore.fetchStatistics()
})
</script>

<style scoped>
.page-content {
  padding: 40px;
  max-width: 1200px;
  margin: 0 auto;
}

.page-header {
  margin-bottom: 40px;
}

.welcome-text {
  text-align: left;
}

.page-title {
  font-size: 36px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0 0 12px 0;
}

.app-name {
  color: var(--primary-color);
  text-decoration: none;
  transition: all 0.2s;
  cursor: pointer;
}

.app-name:hover {
  color: var(--primary-hover);
  text-decoration: underline;
}

.page-subtitle {
  font-size: 18px;
  color: var(--text-secondary);
  margin: 0;
}

.stats-container {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 20px;
  margin-bottom: 40px;
}

.stat-card {
  background: var(--bg-secondary);
  border-radius: 12px;
  padding: 24px;
  box-shadow: 0 1px 3px var(--shadow);
  text-align: center;
  transition: transform 0.2s, box-shadow 0.2s;
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 6px var(--shadow-hover);
}

.stat-icon {
  margin-bottom: 12px;
}

.stat-icon.coffee {
  color: #f59e0b;
}

.stat-icon.package {
  color: #10b981;
}

.stat-icon.refresh {
  color: #3b82f6;
}

.stat-number {
  font-size: 48px;
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: 8px;
}

.stat-label {
  font-size: 16px;
  color: var(--text-secondary);
}

.quick-actions-section {
  margin-bottom: 40px;
}

.section-title {
  font-size: 24px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 16px;
}

.action-buttons {
  display: flex;
  gap: 12px;
}

.action-button {
  padding: 12px 24px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s;
}

.action-button:hover {
  background: var(--primary-hover);
}

.hint-text {
  color: var(--text-tertiary);
  font-size: 14px;
  font-style: italic;
  margin-top: 60px;
}
</style>
