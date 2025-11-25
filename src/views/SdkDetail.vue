<template>
  <div class="page-container">
    <!-- 错误通知 (Element Plus 风格) -->
    <Transition name="notification">
      <div v-if="errorMessage" class="el-notification error">
        <div class="el-notification__icon">
          <svg viewBox="0 0 1024 1024" width="24" height="24">
            <path fill="#f56c6c" d="M512 64a448 448 0 1 1 0 896 448 448 0 0 1 0-896zm0 393.664L407.936 353.6a38.4 38.4 0 1 0-54.336 54.336L457.664 512 353.6 616.064a38.4 38.4 0 1 0 54.336 54.336L512 566.336 616.064 670.4a38.4 38.4 0 1 0 54.336-54.336L566.336 512 670.4 407.936a38.4 38.4 0 1 0-54.336-54.336L512 457.664z"/>
          </svg>
        </div>
        <div class="el-notification__content">
          <h3 class="el-notification__title">{{ $t('alert.error') }}</h3>
          <p class="el-notification__message">{{ errorMessage }}</p>
        </div>
        <button class="el-notification__close" @click="errorMessage = ''">
          <svg viewBox="0 0 1024 1024" width="16" height="16">
            <path fill="currentColor" d="M764.288 214.592 512 466.88 259.712 214.592a31.936 31.936 0 0 0-45.12 45.12L466.752 512 214.528 764.224a31.936 31.936 0 1 0 45.12 45.184L512 557.184l252.288 252.288a31.936 31.936 0 0 0 45.12-45.12L557.12 512.064l252.288-252.352a31.936 31.936 0 1 0-45.12-45.184z"/>
          </svg>
        </button>
      </div>
    </Transition>

    <!-- 返回按钮和头部 -->
    <div class="page-header">
      <div class="header-top">
        <button class="back-button" @click="goBack">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
          </svg>
          {{ $t('common.back') }}
        </button>
        <div class="header-actions">
          <input
            v-model="searchQuery"
            type="text"
            class="search-field"
            :placeholder="$t('sdk.searchVersionPlaceholder')"
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
      <div class="header-info">
        <div class="title-row">
          <h1 class="page-title">{{ sdkName }}</h1>
          <a v-if="websiteUrl" class="website-link" @click="openWebsite" href="javascript:void(0)">
            {{ websiteUrl }}
          </a>
        </div>
        <p class="page-subtitle">{{ sdkDescription }}</p>
      </div>
    </div>

    <!-- 筛选工具栏 -->
    <div class="filter-toolbar">
      <div class="filter-group">
        <label class="filter-label">{{ $t('sdk.filterLabel') }}:</label>
        <select v-model="statusFilter" class="filter-select">
          <option value="all">{{ $t('sdk.filterAll') }}</option>
          <option value="installed">{{ $t('sdk.filterInstalled') }}</option>
          <option value="not-installed">{{ $t('sdk.filterNotInstalled') }}</option>
        </select>
      </div>
    </div>

    <!-- 版本列表 - 可滚动区域 -->
    <div class="scrollable-content">
      <div v-if="!hasLoaded || (loading && versions.length === 0)" class="loading-state">
        <div class="spinner"></div>
        <p>{{ $t('sdk.loadingVersions') }}</p>
      </div>

      <div v-else-if="filteredVersions.length === 0" class="empty-state">
        <div class="empty-icon">📦</div>
        <h3>{{ $t('sdk.emptyVersionsTitle') }}</h3>
        <p>{{ $t('sdk.emptyVersionsSubtitle') }}</p>
      </div>

      <div v-else class="versions-tree">
      <!-- 按主版本号分组 -->
      <div
        v-for="group in groupedVersions"
        :key="group.majorVersion"
        class="version-group"
      >
        <!-- 主版本标题（可折叠） -->
        <div
          class="version-group-header"
          @click="toggleVersionGroup(group.majorVersion)"
        >
          <div class="group-info">
            <svg
              class="expand-icon"
              :class="{ 'expanded': expandedGroups.has(group.majorVersion) }"
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="currentColor"
            >
              <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/>
            </svg>
            <h3 class="group-name">v{{ group.majorVersion }}</h3>
            <span class="version-count">{{ group.versions.length }} {{ $t('jdk.versionCount') }}</span>
          </div>
          <div class="group-stats">
            <span v-if="group.installedCount > 0" class="stat-badge installed">
              {{ group.installedCount }} {{ $t('sdk.filterInstalled') }}
            </span>
          </div>
        </div>

        <!-- 版本列表（可展开） -->
        <transition name="accordion">
          <div v-show="expandedGroups.has(group.majorVersion)" class="group-versions">
            <div
              v-for="version in group.versions"
              :key="version.identifier"
              class="version-row"
              :class="{ 'version-installed': version.installed, 'version-active': version.inUse }"
            >
              <div class="version-info">
                <div class="version-header">
                  <span class="version-number">{{ version.version }}</span>
                  <span v-if="version.inUse" class="current-badge">{{ $t('sdk.currentVersion') }}</span>
                </div>
                <div class="version-meta">
                  <span class="identifier">{{ version.identifier }}</span>
                </div>
              </div>

              <div class="version-actions">
                <!-- 进度显示 -->
                <div v-if="isOperating(version.version)" class="progress-container">
                  <div class="progress-bar">
                    <div
                      class="progress-fill"
                      :style="{ width: (getProgress(version.version)?.percentage || 0) + '%' }"
                    ></div>
                  </div>
                  <div class="progress-actions">
                    <span class="progress-text" :key="`text-${version.version}`">
                      {{ formatProgressMessage(version.version) }}
                    </span>
                  </div>
                </div>

                <!-- 操作按钮 -->
                <template v-else>
                  <button
                    v-if="!version.installed"
                    class="action-btn install-btn"
                    @click="installSdk(version)"
                    :disabled="loading"
                  >
                    {{ $t('sdk.actionInstall') }}
                  </button>
                  <template v-else>
                    <button
                      v-if="version.inUse"
                      class="action-btn default-btn"
                      disabled
                    >
                      {{ $t('sdk.actionDefault') }}
                    </button>
                    <button
                      v-else
                      class="action-btn use-btn"
                      @click="setDefault(version)"
                      :disabled="loading"
                    >
                      {{ $t('sdk.actionUse') }}
                    </button>
                    <button
                      class="action-btn uninstall-btn"
                      @click="uninstallSdk(version)"
                      :disabled="loading || localUninstallingVersions.has(version.version) || sdkStore.isUninstalling(candidate, version.version)"
                    >
                      {{ (localUninstallingVersions.has(version.version) || sdkStore.isUninstalling(candidate, version.version)) ? $t('sdk.actionUninstalling') : $t('sdk.actionUninstall') }}
                    </button>
                  </template>
                </template>
              </div>
            </div>
          </div>
        </transition>
      </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useSdkStore, type SdkVersion } from '../stores/sdkStore'
import { useInstallProgressStore } from '../stores/installProgressStore'
import { useI18n } from 'vue-i18n'
import { open } from '@tauri-apps/plugin-shell'

const { t } = useI18n()
const router = useRouter()
const route = useRoute()
const sdkStore = useSdkStore()
const installProgressStore = useInstallProgressStore()

const candidate = ref(route.params.candidate as string)
const sdkName = ref(candidate.value)
const sdkDescription = ref('')
const websiteUrl = ref('')

// 从SDK候选者列表中获取名称和描述
function loadSdkInfo() {
  const sdk = sdkStore.sdkCandidates.find(s => s.candidate === candidate.value)
  if (sdk) {
    sdkName.value = sdk.name
    sdkDescription.value = sdk.description
    websiteUrl.value = sdk.website || ''
    console.log('SDK Info loaded:', { name: sdkName.value, website: websiteUrl.value })
  } else {
    console.log('SDK not found in candidates list, candidate:', candidate.value, 'total candidates:', sdkStore.sdkCandidates.length)
  }
}

const searchQuery = ref('')
const statusFilter = ref('all')
const hasLoaded = ref(false)
const errorMessage = ref('')
const expandedGroups = ref<Set<string>>(new Set())
// 本地状态：正在卸载的版本集合（用于立即防重复点击）
const localUninstallingVersions = ref<Set<string>>(new Set())

// 显示错误通知
function showError(message: string) {
  errorMessage.value = message
  // 5秒后自动关闭
  setTimeout(() => {
    errorMessage.value = ''
  }, 5000)
}

// 从错误对象中提取简短的错误信息
function extractErrorMessage(e: any): string {
  const message = e?.message || String(e)
  // 尝试提取 HTTP 状态码相关信息 (如 "404 Not Found", "500 Internal Server Error")
  const httpStatusMatch = message.match(/(\d{3}\s+[A-Za-z\s]+)/)
  if (httpStatusMatch) {
    return httpStatusMatch[1].trim()
  }
  // 尝试提取 "status: xxx" 格式
  const statusMatch = message.match(/status[:\s]+(\d{3})/i)
  if (statusMatch) {
    return `HTTP ${statusMatch[1]}`
  }
  // 如果消息太长，截取前50个字符
  if (message.length > 50) {
    return message.substring(0, 50) + '...'
  }
  return message
}

// 进度跟踪 - 使用全局 store
// 辅助函数：将全局 store 的方法映射到本地，保持接口兼容性

const loading = computed(() => sdkStore.loading)
const versions = ref<SdkVersion[]>([])

// 版本号比较函数（降序，新版本在前）
function compareVersions(a: string, b: string): number {
  const partsA = a.split(/[.\-_]/)
  const partsB = b.split(/[.\-_]/)
  const maxLen = Math.max(partsA.length, partsB.length)

  for (let i = 0; i < maxLen; i++) {
    const partA = partsA[i] || '0'
    const partB = partsB[i] || '0'
    const numA = parseInt(partA, 10)
    const numB = parseInt(partB, 10)

    if (!isNaN(numA) && !isNaN(numB)) {
      if (numA !== numB) return numB - numA
    } else {
      const cmp = partB.localeCompare(partA)
      if (cmp !== 0) return cmp
    }
  }
  return 0
}

const filteredVersions = computed(() => {
  let result = versions.value

  // 搜索过滤
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(v =>
      v.version.toLowerCase().includes(query) ||
      v.identifier.toLowerCase().includes(query)
    )
  }

  // 状态过滤
  if (statusFilter.value === 'installed') {
    result = result.filter(v => v.installed)
  } else if (statusFilter.value === 'not-installed') {
    result = result.filter(v => !v.installed)
  }

  // 按版本号排序（新版本在前）
  return [...result].sort((a, b) => compareVersions(a.version, b.version))
})

// 按主版本号分组
const groupedVersions = computed(() => {
  const groups = new Map<string, typeof versions.value>()

  filteredVersions.value.forEach(version => {
    // 提取主版本号（第一个数字）
    const majorVersion = version.version.split(/[.\-_]/)[0]
    if (!groups.has(majorVersion)) {
      groups.set(majorVersion, [])
    }
    groups.get(majorVersion)!.push(version)
  })

  // 转换为数组并添加统计信息
  return Array.from(groups.entries()).map(([majorVersion, versions]) => ({
    majorVersion,
    versions,
    installedCount: versions.filter(v => v.installed).length
  })).sort((a, b) => {
    // 按主版本号降序排列（新版本在前）
    const numA = parseInt(a.majorVersion, 10)
    const numB = parseInt(b.majorVersion, 10)
    if (!isNaN(numA) && !isNaN(numB)) {
      return numB - numA
    }
    return b.majorVersion.localeCompare(a.majorVersion)
  })
})

// 切换分组展开/折叠状态
function toggleVersionGroup(majorVersion: string) {
  if (expandedGroups.value.has(majorVersion)) {
    expandedGroups.value.delete(majorVersion)
  } else {
    expandedGroups.value.add(majorVersion)
  }
}

// 监听分组变化，自动展开第一个分组
watch(groupedVersions, (newGroups) => {
  if (newGroups.length > 0 && expandedGroups.value.size === 0) {
    expandedGroups.value.add(newGroups[0].majorVersion)
  }
}, { immediate: true })

// 生成进度跟踪的唯一标识符（与后端事件格式一致）
function getProgressKey(version: string): string {
  return `${candidate.value}-${version}`
}

function isOperating(version: string): boolean {
  const key = getProgressKey(version)
  return installProgressStore.isOperating(key)
}

function getProgress(version: string) {
  const key = getProgressKey(version)
  return installProgressStore.getProgress(key)
}

// 格式化进度消息（支持国际化）
function formatProgressMessage(version: string): string {
  const key = getProgressKey(version)
  const progress = installProgressStore.getProgress(key)
  if (!progress) {
    return t('sdk.progressProcessing')
  }

  const task = installProgressStore.tasks.get(key)
  if (!task) {
    return progress.message || t('sdk.progressProcessing')
  }

  // 根据任务状态返回国际化消息
  switch (task.status) {
    case 'downloading':
      // 检查是否是"Starting..."状态
      if (progress.percentage === 0 || progress.message === 'Starting...') {
        return t('sdk.progressStarting')
      }
      // 下载中显示具体进度
      return progress.message
    case 'installing':
      return t('sdk.progressInstalling')
    case 'completed':
      return t('sdk.progressCompleted')
    case 'failed':
      return t('sdk.installFailed')
    default:
      return progress.message || t('sdk.progressProcessing')
  }
}

async function refreshData(forceRefresh = true) {
  try {
    console.log('[SdkDetail] refreshData: Fetching versions for', candidate.value,
      forceRefresh ? '(force refresh)' : '(from cache)')
    // 获取该SDK的所有版本
    versions.value = await sdkStore.listSdkVersions(candidate.value, forceRefresh)
    console.log('[SdkDetail] refreshData: Got', versions.value.length, 'versions.',
      'Installed:', versions.value.filter(v => v.installed).length)
    hasLoaded.value = true
    // 全局 store 会自动管理进度清理
  } catch (e) {
    console.error('[SdkDetail] Failed to refresh SDK versions:', e)
    hasLoaded.value = true
  }
}

// 清理已完成的进度 - 不再需要，由全局 store 自动管理
// function cleanupCompletedProgress() {
//   // 全局 store 会在安装完成后自动清理
// }

async function installSdk(version: SdkVersion) {
  const progressKey = getProgressKey(version.version)

  // 使用全局 store 启动安装任务
  installProgressStore.startTask(candidate.value, version.version)

  try {
    await sdkStore.downloadAndInstallSdk(candidate.value, version.version)

    // 刷新数据
    await refreshData()

    // 数据刷新完成后移除任务，避免UI闪烁
    installProgressStore.removeTask(progressKey)
  } catch (e: any) {
    console.error('Failed to install SDK:', e)
    // 提取简短的错误信息
    const errorDetail = extractErrorMessage(e)
    showError(`${t('sdk.installFailed')}: ${errorDetail}`)
    // 失败时移除任务
    installProgressStore.removeTask(progressKey)
  }
}

async function uninstallSdk(version: SdkVersion) {
  console.log('[SdkDetail] uninstallSdk clicked:', candidate.value, version.version)

  const key = version.version

  // 立即检查本地状态，防止快速双击
  if (localUninstallingVersions.value.has(key)) {
    console.log('[SdkDetail] Already uninstalling (local check), ignoring click')
    return
  }

  // 双重检查 store 状态
  if (sdkStore.isUninstalling(candidate.value, version.version)) {
    console.log('[SdkDetail] Already uninstalling (store check), ignoring click')
    return
  }

  // 立即添加到本地状态
  localUninstallingVersions.value.add(key)
  localUninstallingVersions.value = new Set(localUninstallingVersions.value)

  try {
    console.log('[SdkDetail] Calling store.uninstallSdk...')
    await sdkStore.uninstallSdk(candidate.value, version.version)
    console.log('[SdkDetail] Uninstall completed, refreshing data...')

    // 刷新数据以更新UI状态
    await refreshData()
    console.log('[SdkDetail] Data refreshed')
  } catch (e: any) {
    console.error('[SdkDetail] Failed to uninstall SDK:', e)
    const errorDetail = extractErrorMessage(e)
    showError(`${t('sdk.uninstallFailed')}: ${errorDetail}`)
  } finally {
    // 移除本地状态
    localUninstallingVersions.value.delete(key)
    localUninstallingVersions.value = new Set(localUninstallingVersions.value)
  }
}

async function setDefault(version: SdkVersion) {
  try {
    await sdkStore.setDefaultSdkVersion(candidate.value, version.version)
    // Update all versions: clear previous default and set new one
    versions.value.forEach(v => {
      v.inUse = (v.version === version.version)
    })
  } catch (e: any) {
    console.error('Failed to set default:', e)
    const errorDetail = extractErrorMessage(e)
    showError(`${t('sdk.setDefaultFailed')}: ${errorDetail}`)
  }
}

function goBack() {
  router.push('/sdk')
}

// 打开网站链接
async function openWebsite() {
  if (websiteUrl.value) {
    try {
      await open(websiteUrl.value)
    } catch (error) {
      console.error('Failed to open URL:', error)
    }
  }
}

// 监听安装/卸载完成事件，刷新数据
let unlistenInstallComplete: (() => void) | null = null
let unlistenUninstallComplete: (() => void) | null = null

onMounted(async () => {
  // 确保先加载SDK候选者列表，然后再获取详细信息（使用缓存）
  if (sdkStore.sdkCandidates.length === 0) {
    await sdkStore.fetchSdkCandidates(false)
  }
  loadSdkInfo()
  // 初始加载使用缓存
  await refreshData(false)

  // 监听安装完成事件，刷新当前页面数据
  const { listen } = await import('@tauri-apps/api/event')

  unlistenInstallComplete = await listen<any>('install-complete', async (event) => {
    const { candidate: eventCandidate } = event.payload
    // 只刷新当前SDK的数据
    if (eventCandidate === candidate.value) {
      await refreshData()
    }
  })

  unlistenUninstallComplete = await listen<any>('uninstall-complete', async (event) => {
    const { candidate: eventCandidate } = event.payload
    // 只刷新当前SDK的数据
    if (eventCandidate === candidate.value) {
      await refreshData()
    }
  })
})

// 清理事件监听器
onUnmounted(() => {
  if (unlistenInstallComplete) {
    unlistenInstallComplete()
  }
  if (unlistenUninstallComplete) {
    unlistenUninstallComplete()
  }
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
  margin-bottom: 24px;
  flex-shrink: 0;
}

.header-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.header-actions {
  display: flex;
  gap: 12px;
  align-items: center;
}

.header-info {
  width: 100%;
}

.back-button {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-size: 14px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.back-button:hover {
  border-color: var(--primary-color);
  color: var(--primary-color);
}

.title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.page-title {
  font-size: 32px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
}

.website-link {
  font-size: 14px;
  color: var(--primary-color);
  text-decoration: none;
  cursor: pointer;
  transition: all 0.2s;
  padding: 6px 12px;
  border-radius: 6px;
  margin-left: auto;
  font-weight: 500;
}

.website-link:hover {
  color: var(--primary-hover);
  background: var(--bg-tertiary);
  text-decoration: underline;
}

.page-subtitle {
  font-size: 16px;
  color: var(--text-secondary);
  margin: 8px 0 0 0;
}

.search-field {
  padding: 10px 16px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-size: 14px;
  width: 250px;
  transition: border-color 0.2s;
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
  gap: 16px;
  margin-bottom: 24px;
  padding: 16px;
  background: var(--bg-secondary);
  border-radius: 12px;
  box-shadow: 0 1px 3px var(--shadow);
  flex-shrink: 0;
}

.scrollable-content {
  flex: 1;
  overflow-y: auto;
  padding-bottom: 40px;
}

.filter-group {
  display: flex;
  align-items: center;
  gap: 12px;
}

.filter-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-secondary);
  white-space: nowrap;
}

.filter-select {
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 14px;
  color: var(--text-primary);
  background: var(--bg-secondary);
  cursor: pointer;
  transition: border-color 0.2s;
}

.filter-select:focus {
  outline: none;
  border-color: var(--primary-color);
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

/* 树形布局 */
.versions-tree {
  display: flex;
  flex-direction: column;
  gap: 0;
}

/* 版本分组 */
.version-group {
  background: var(--bg-secondary);
  overflow: hidden;
  transition: background 0.2s;
  box-shadow: 1px 0 0 0 var(--bg-tertiary), -1px 0 0 0 var(--bg-tertiary);
}

.version-group + .version-group {
  box-shadow: 0 -0.5px 0 0 var(--border-color), 1px 0 0 0 var(--bg-tertiary), -1px 0 0 0 var(--bg-tertiary);
}

.version-group:first-child {
  box-shadow: 0 -0.5px 0 0 var(--border-color), 1px 0 0 0 var(--bg-tertiary), -1px 0 0 0 var(--bg-tertiary);
  border-top-left-radius: 12px;
  border-top-right-radius: 12px;
}

.version-group:last-child {
  box-shadow: 0 0.5px 0 0 var(--border-color), 1px 0 0 0 var(--bg-tertiary), -1px 0 0 0 var(--bg-tertiary);
  border-bottom-left-radius: 12px;
  border-bottom-right-radius: 12px;
}

/* 分组标题 */
.version-group-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: var(--bg-secondary);
  cursor: pointer;
  transition: background 0.2s;
  user-select: none;
  border-bottom: 1px solid var(--bg-tertiary);
}

.version-group-header:hover {
  background: var(--bg-tertiary);
}

.group-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.expand-icon {
  color: var(--text-secondary);
  transition: transform 0.3s ease;
}

.expand-icon.expanded {
  transform: rotate(90deg);
}

.group-name {
  margin: 0;
  font-size: 18px;
  font-weight: 700;
  color: var(--text-primary);
}

.version-count {
  font-size: 14px;
  color: var(--text-secondary);
  font-weight: 500;
}

.group-stats {
  display: flex;
  gap: 8px;
}

.stat-badge {
  padding: 4px 12px;
  border-radius: 12px;
  font-size: 13px;
  font-weight: 600;
}

.stat-badge.installed {
  background: #d1fae5;
  color: #065f46;
}

/* 分组版本列表容器 */
.group-versions {
  background: var(--bg-secondary);
  padding: 4px 8px;
}

.version-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 16px;
  margin: 2px 0;
  background: var(--bg-secondary);
  border-radius: 6px;
  transition: transform 0.2s, box-shadow 0.2s;
}

.version-row:hover {
  transform: translateX(2px);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
}

.version-info {
  flex: 1;
  min-width: 0;
}

.version-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 6px;
  flex-wrap: wrap;
}

.version-number {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.current-badge {
  padding: 3px 8px;
  background: #d1fae5;
  color: #065f46;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 600;
}

.version-meta {
  font-size: 13px;
  color: var(--text-tertiary);
}

.identifier {
  font-family: 'Monaco', 'Courier New', monospace;
}

.version-actions {
  display: flex;
  gap: 6px;
  align-items: center;
}

.action-btn {
  padding: 6px 12px;
  border: none;
  border-radius: 4px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.install-btn {
  background: var(--success-color);
  color: white;
}

.install-btn:hover:not(:disabled) {
  background: var(--success-hover);
}

.install-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.use-btn {
  background: var(--primary-color);
  color: white;
}

.use-btn:hover:not(:disabled) {
  background: var(--primary-hover);
}

.use-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.default-btn {
  background: var(--text-tertiary);
  color: white;
  cursor: not-allowed;
  opacity: 0.8;
}

.uninstall-btn {
  background: var(--bg-secondary);
  color: var(--danger-color);
  border: 1px solid var(--danger-color);
}

.uninstall-btn:hover:not(:disabled) {
  background: var(--danger-color);
  color: white;
}

.uninstall-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.progress-container {
  min-width: 200px;
  flex-shrink: 0;
  flex-grow: 0;
}

.progress-bar {
  height: 8px;
  background: var(--border-color);
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 6px;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--primary-color) 0%, #3b82f6 100%);
  transition: width 0.3s ease;
  border-radius: 4px;
}

.progress-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  justify-content: space-between;
}

.progress-text {
  font-size: 11px;
  color: var(--text-secondary);
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-height: 18px;
  line-height: 18px;
}

/* Element Plus 风格通知样式 */
.el-notification {
  position: fixed;
  top: 16px;
  right: 16px;
  z-index: 9999;
  display: flex;
  align-items: flex-start;
  gap: 12px;
  width: 330px;
  padding: 14px 20px 14px 14px;
  background: #fff;
  border-radius: 8px;
  box-shadow: 0 6px 16px 0 rgba(0, 0, 0, 0.08), 0 3px 6px -4px rgba(0, 0, 0, 0.12), 0 9px 28px 8px rgba(0, 0, 0, 0.05);
  border: 1px solid #ebeef5;
  overflow: hidden;
}


.el-notification__icon {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.el-notification__content {
  flex: 1;
  min-width: 0;
}

.el-notification__title {
  margin: 0 0 6px 0;
  font-size: 16px;
  font-weight: 600;
  color: #303133;
  line-height: 1.4;
}

.el-notification__message {
  margin: 0;
  font-size: 14px;
  color: #606266;
  line-height: 1.5;
  word-break: break-word;
}

.el-notification__close {
  position: absolute;
  top: 12px;
  right: 12px;
  padding: 0;
  background: transparent;
  border: none;
  cursor: pointer;
  color: #909399;
  transition: color 0.2s;
}

.el-notification__close:hover {
  color: #606266;
}

@keyframes notification-in {
  0% {
    opacity: 0;
    transform: translateX(100%);
  }
  100% {
    opacity: 1;
    transform: translateX(0);
  }
}

@keyframes notification-out {
  0% {
    opacity: 1;
    transform: translateX(0);
  }
  100% {
    opacity: 0;
    transform: translateX(100%);
  }
}
</style>
