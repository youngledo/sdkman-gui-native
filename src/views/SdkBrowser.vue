<template>
  <div class="page-container">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-top">
        <h1 class="page-title">{{ $t('sdk.title') }}</h1>
        <div class="header-actions">
          <input
            v-model="searchQuery"
            type="text"
            class="search-field"
            :placeholder="$t('sdk.searchPlaceholder')"
          />
          <button class="refresh-button" @click="() => refreshData(true)" :disabled="loading">
            <svg v-if="!loading" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
              <path d="M17.65 6.35A7.958 7.958 0 0 0 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08A5.99 5.99 0 0 1 12 18c-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
            </svg>
            <span v-if="!loading">{{ $t('sdk.actionRefresh') }}</span>
            <span v-else>{{ $t('common.loading') }}</span>
          </button>
        </div>
      </div>
      <p class="page-subtitle">{{ $t('sdk.subtitle') }}</p>
    </div>

    <!-- 分类筛选工具栏 -->
    <div class="filter-toolbar">
      <div class="filter-chips">
        <button
          v-for="category in categories"
          :key="category.value"
          class="filter-chip"
          :class="{ 'filter-chip-active': selectedCategory === category.value }"
          @click="selectedCategory = category.value"
        >
          {{ $t(category.labelKey) }}
        </button>
      </div>
      <div class="filter-right">
        <label class="installed-filter">
          <input type="checkbox" v-model="installedOnly" />
          <span>{{ $t('sdk.filterInstalledOnly') }}</span>
        </label>
      </div>
    </div>

    <!-- SDK网格区域 - 可滚动区域 -->
    <div class="scrollable-content">
      <div v-if="loading || (sdks.length === 0 && !hasLoaded)" class="loading-state">
        <div class="spinner"></div>
        <p>{{ $t('sdk.loadingSdks') }}</p>
      </div>

      <div v-else-if="sdks.length === 0" class="empty-state">
        <div class="empty-icon">📦</div>
        <h3>{{ $t('sdk.emptyTitle') }}</h3>
        <p>{{ $t('sdk.emptySubtitle') }}</p>
      </div>

      <div v-else-if="filteredSdks.length === 0" class="empty-state">
        <div class="empty-icon">🔍</div>
        <h3>{{ $t('sdk.emptyVersionsTitle') }}</h3>
        <p>{{ $t('sdk.emptyVersionsSubtitle') }}</p>
      </div>

      <div v-else class="sdk-grid">
      <div
        v-for="sdk in filteredSdks"
        :key="sdk.candidate"
        class="sdk-card"
        @click="selectSdk(sdk)"
      >
        <div class="sdk-header">
          <h3 class="sdk-name">{{ sdk.name }}</h3>
          <span class="sdk-badge">{{ $t(getCategoryLabel(sdk.category)) }}</span>
        </div>
        <p class="sdk-description">{{ sdk.description }}</p>
        <div class="sdk-footer">
          <span class="sdk-candidate">{{ sdk.candidate }}</span>
          <span v-if="isInstalled(sdk.candidate)" class="installed-badge">✓ {{ $t('jdk.filterInstalled') }}</span>
        </div>
      </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useSdkStore, type Sdk } from '../stores/sdkStore'

const router = useRouter()
const sdkStore = useSdkStore()

const searchQuery = ref('')
const selectedCategory = ref('all')
const installedOnly = ref(false)
const installedCandidates = ref<string[]>([])
const hasLoaded = ref(false)

const loading = computed(() => sdkStore.loading)
const sdks = computed(() => sdkStore.sdkCandidates)

const categories = [
  { value: 'all', labelKey: 'sdk.categoryAll' },
  { value: 'LANGUAGES', labelKey: 'sdk.categoryLanguages' },
  { value: 'BUILD_TOOLS', labelKey: 'sdk.categoryBuildTools' },
  { value: 'FRAMEWORKS', labelKey: 'sdk.categoryFrameworks' },
  { value: 'SERVERS', labelKey: 'sdk.categoryServers' },
  { value: 'MQ', labelKey: 'sdk.categoryMq' },
  { value: 'TOOLS', labelKey: 'sdk.categoryTools' },
  { value: 'OTHER', labelKey: 'sdk.categoryOther' },
]

const filteredSdks = computed(() => {
  let result = sdks.value

  // 搜索过滤
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(sdk =>
      sdk.name.toLowerCase().includes(query) ||
      sdk.candidate.toLowerCase().includes(query) ||
      sdk.description.toLowerCase().includes(query)
    )
  }

  // 分类过滤
  if (selectedCategory.value !== 'all') {
    result = result.filter(sdk => sdk.category === selectedCategory.value)
  }

  // 已安装过滤
  if (installedOnly.value) {
    result = result.filter(sdk => isInstalled(sdk.candidate))
  }

  return result
})

function getCategoryLabel(category: string): string {
  const cat = categories.find(c => c.value === category)
  return cat ? cat.labelKey : category
}

function isInstalled(candidate: string): boolean {
  return installedCandidates.value.includes(candidate)
}

async function refreshData(forceRefresh = true) {
  console.log('Refreshing SDK data...', forceRefresh ? '(force refresh)' : '(from cache)')
  await sdkStore.fetchSdkCandidates(forceRefresh)
  console.log('SDK data loaded:', sdks.value.length, 'items')
  await loadInstalledCandidates()
  hasLoaded.value = true
}

async function loadInstalledCandidates() {
  try {
    installedCandidates.value = await sdkStore.listInstalledCandidates()
  } catch (e) {
    console.error('Failed to load installed candidates:', e)
  }
}

function selectSdk(sdk: Sdk) {
  console.log('Selected SDK:', sdk)
  router.push(`/sdk/${sdk.candidate}`)
}

onMounted(async () => {
  // 初始加载使用缓存
  await refreshData(false)
})
</script>

<style scoped>
.page-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  max-width: 1400px;
  margin: 0 auto;
  padding: 40px;
  padding-bottom: 0;
}

.page-header {
  flex-shrink: 0;
  margin-bottom: 24px;
}

.header-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  gap: 20px;
}

.header-actions {
  display: flex;
  gap: 12px;
  align-items: center;
  flex-shrink: 0;
}

.page-title {
  font-size: 32px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
  white-space: nowrap;
}

.page-subtitle {
  font-size: 16px;
  color: var(--text-secondary);
  margin: 0;
  line-height: 1.5;
}

.search-field {
  padding: 10px 16px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-size: 14px;
  width: 250px;
  transition: border-color 0.2s;
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.search-field:focus {
  outline: none;
  border-color: var(--primary-color);
}

.refresh-button {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s;
  white-space: nowrap;
}

.refresh-button:hover:not(:disabled) {
  background: var(--primary-hover);
}

.refresh-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.filter-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  padding: 16px;
  background: var(--bg-secondary);
  border-radius: 12px;
  box-shadow: 0 1px 3px var(--shadow);
  gap: 16px;
  flex-wrap: wrap;
  flex-shrink: 0;
}

.scrollable-content {
  flex: 1;
  overflow-y: auto;
  padding-bottom: 40px;
}

.filter-chips {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.filter-chip {
  padding: 8px 16px;
  border: 1px solid var(--border-color);
  background: var(--bg-secondary);
  border-radius: 20px;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.filter-chip:hover {
  border-color: var(--primary-color);
  color: var(--primary-color);
}

.filter-chip-active {
  background: var(--primary-color);
  color: white;
  border-color: var(--primary-color);
}

.filter-chip-active:hover {
  color: white;
}

.filter-right {
  display: flex;
  align-items: center;
}

.installed-filter {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-size: 14px;
  color: var(--text-secondary);
}

.installed-filter input[type="checkbox"] {
  width: 18px;
  height: 18px;
  cursor: pointer;
}

.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 20px;
  color: var(--text-secondary);
}

.spinner {
  width: 48px;
  height: 48px;
  border: 4px solid var(--border-color);
  border-top-color: var(--primary-color);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 16px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.empty-icon {
  font-size: 64px;
  margin-bottom: 16px;
}

.sdk-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: 16px;
}

.sdk-card {
  background: var(--bg-secondary);
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 1px 3px var(--shadow);
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.sdk-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 6px var(--shadow-hover);
}

.sdk-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
}

.sdk-name {
  font-size: 20px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
  flex: 1;
}

.sdk-badge {
  padding: 4px 10px;
  background: #e0e7ff;
  color: #3730a3;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
}

.sdk-description {
  font-size: 14px;
  color: var(--text-secondary);
  margin: 0;
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.sdk-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 8px;
  border-top: 1px solid var(--border-color);
}

.sdk-candidate {
  font-size: 13px;
  color: var(--text-tertiary);
  font-family: 'Monaco', 'Courier New', monospace;
}

.installed-badge {
  padding: 4px 8px;
  background: #d1fae5;
  color: #065f46;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
}
</style>
