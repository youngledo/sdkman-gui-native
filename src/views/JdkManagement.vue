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

    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-top">
        <h1 class="page-title">{{ $t('jdk.title') }}</h1>
        <div class="header-actions">
          <input
            v-model="searchQuery"
            type="text"
            class="search-field"
            :placeholder="$t('jdk.searchPlaceholder')"
          />
          <button class="refresh-button" @click="() => refreshData(true)" :disabled="loading">
            <svg v-if="!loading" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
              <path d="M17.65 6.35A7.958 7.958 0 0 0 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08A5.99 5.99 0 0 1 12 18c-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
            </svg>
            <span v-if="!loading">{{ $t('jdk.actionRefresh') }}</span>
            <span v-else>{{ $t('common.loading') }}</span>
          </button>
        </div>
      </div>
      <p class="page-subtitle">{{ $t('home.subtitle') }}</p>
    </div>

    <!-- 筛选工具栏 -->
    <div class="filter-toolbar">
      <div class="filter-group">
        <label class="filter-label">{{ $t('jdk.filterLabel') }}:</label>
        <select v-model="statusFilter" class="filter-select">
          <option value="all">{{ $t('jdk.filterAll') }}</option>
          <option value="installed">{{ $t('jdk.filterInstalled') }}</option>
          <option value="not-installed">{{ $t('jdk.filterNotInstalled') }}</option>
        </select>
      </div>

      <div class="filter-group">
        <label class="filter-label">{{ $t('jdk.vendorLabel') }}:</label>
        <select v-model="vendorFilter" class="filter-select">
          <option value="all">{{ $t('jdk.vendorAll') }}</option>
          <option v-for="vendor in uniqueVendors" :key="vendor" :value="vendor">
            {{ vendor }}
          </option>
        </select>
      </div>

      <div class="filter-group">
        <label class="filter-label">{{ $t('jdk.categoryLabel') }}:</label>
        <select v-model="categoryFilter" class="filter-select">
          <option value="all">{{ $t('jdk.categoryAll') }}</option>
          <option value="JDK">JDK</option>
          <option value="JAVAFX">JavaFX</option>
          <option value="NIK">NIK</option>
        </select>
      </div>
    </div>

    <!-- JDK树形列表（手风琴模式） - 可滚动区域 -->
    <div class="scrollable-content">
      <div v-if="!hasLoaded || (loading && versions.length === 0)" class="loading-state">
        <div class="spinner"></div>
        <p>{{ $t('common.loading') }}</p>
      </div>

      <div v-else-if="versions.length === 0" class="empty-state">
        <div class="empty-icon">☕</div>
        <h3>{{ $t('jdk.messageNoJdkFound') }}</h3>
        <p>{{ $t('jdk.emptyHint') }}</p>
      </div>

      <div v-else-if="filteredVersions.length === 0" class="empty-state">
        <div class="empty-icon">🔍</div>
        <h3>{{ $t('jdk.messageNoJdkFound') }}</h3>
        <p>{{ $t('jdk.emptyHint') }}</p>
      </div>

      <div v-else class="jdk-tree">
      <!-- 按供应商分组 -->
      <div
        v-for="group in groupedVersions"
        :key="group.vendor"
        class="vendor-group"
      >
        <!-- 供应商标题（可折叠） -->
        <div
          class="vendor-header"
          @click="toggleVendor(group.vendor)"
        >
          <div class="vendor-info">
            <svg
              class="expand-icon"
              :class="{ 'expanded': expandedVendors.has(group.vendor) }"
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="currentColor"
            >
              <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/>
            </svg>
            <h3 class="vendor-name">{{ group.vendor }}</h3>
            <a
              v-if="getVendorWebsite(group.vendor)"
              class="vendor-website"
              @click.stop="openVendorWebsite(group.vendor)"
              href="javascript:void(0)"
            >
              {{ $t('sdk.website') }}
            </a>
            <span v-if="group.vendor === 'Temurin'" class="recommended-badge">{{ $t('jdk.recommended') }}</span>
            <span class="version-count">{{ group.versions.length }} {{ $t('jdk.versionCount') }}</span>
          </div>
          <div class="vendor-stats">
            <span v-if="group.installedCount > 0" class="stat-badge installed">
              {{ group.installedCount }} {{ $t('jdk.filterInstalled') }}
            </span>
          </div>
        </div>

        <!-- 版本列表（可展开） -->
        <transition name="accordion">
          <div v-show="expandedVendors.has(group.vendor)" class="vendor-versions">
            <div
              v-for="version in group.versions"
              :key="version.identifier"
              class="jdk-item"
              :class="{ 'jdk-item-installed': version.installed, 'jdk-item-active': version.inUse }"
            >
              <div class="jdk-info">
                <div class="jdk-version">
                  {{ version.version }}
                  <span v-if="version.inUse" class="badge badge-active">{{ $t('jdk.statusActive') }}</span>
                  <span v-else-if="version.installed" class="badge badge-installed">{{ $t('jdk.statusInstalled') }}</span>
                </div>
                <div class="jdk-identifier">{{ version.identifier }}</div>
                <div class="jdk-dist">{{ version.dist }}</div>
              </div>

              <div class="jdk-actions">
                <!-- 进度显示 -->
                <div v-if="isOperating(version.identifier)" class="progress-container" :key="`progress-${version.identifier}`">
                  <div class="progress-bar">
                    <div
                      class="progress-fill"
                      :style="{ width: (getProgress(version.identifier)?.percentage || 0) + '%' }"
                    ></div>
                  </div>
                  <div class="progress-actions">
                    <span class="progress-text" :key="`text-${version.identifier}`">
                      {{ formatProgressMessage(version.identifier) }}
                    </span>
                  </div>
                </div>

                <!-- 操作按钮 -->
                <template v-else>
                  <button
                    v-if="!version.installed"
                    class="action-btn install-btn"
                    @click="installJdk(version)"
                    :disabled="loading"
                  >
                    {{ $t('jdk.actionInstall') }}
                  </button>
                  <template v-else>
                    <button
                      v-if="version.inUse"
                      class="action-btn default-btn"
                      disabled
                    >
                      {{ $t('jdk.actionDefault') }}
                    </button>
                    <button
                      v-else
                      class="action-btn use-btn"
                      @click="setDefault(version)"
                      :disabled="loading"
                    >
                      {{ $t('jdk.actionUse') }}
                    </button>
                    <button
                      class="action-btn uninstall-btn"
                      @click="uninstallJdk(version)"
                      :disabled="loading || localUninstallingVersions.has(version.identifier) || sdkStore.isUninstalling('java', version.identifier)"
                    >
                      {{ (localUninstallingVersions.has(version.identifier) || sdkStore.isUninstalling('java', version.identifier)) ? $t('jdk.actionUninstalling') : $t('jdk.actionUninstall') }}
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
import { ref, computed, watch, onMounted } from 'vue'
import { useSdkStore, type SdkVersion } from '../stores/sdkStore'
import { useInstallProgressStore } from '../stores/installProgressStore'
import { useI18n } from 'vue-i18n'
import { open } from '@tauri-apps/plugin-shell'

const { t } = useI18n()
const sdkStore = useSdkStore()
const installProgressStore = useInstallProgressStore()

// JDK 供应商官网映射
const vendorWebsites: Record<string, string> = {
  'Temurin': 'https://adoptium.net/',
  'Oracle': 'https://www.oracle.com/java/',
  'Corretto': 'https://aws.amazon.com/corretto/',
  'Zulu': 'https://www.azul.com/downloads/',
  'SapMachine': 'https://sap.github.io/SapMachine/',
  'Liberica': 'https://bell-sw.com/pages/downloads/',
  'Liberica NIK': 'https://bell-sw.com/liberica-native-image-kit/',
  'GraalVM CE': 'https://www.graalvm.org/downloads/',
  'GraalVM Oracle': 'https://www.graalvm.org/downloads/',
  'Microsoft': 'https://www.microsoft.com/openjdk',
  'Semeru': 'https://developer.ibm.com/languages/java/semeru-runtimes/',
  'Dragonwell': 'https://dragonwell-jdk.io/',
  'JetBrains': 'https://www.jetbrains.com/java/',
  'Mandrel': 'https://github.com/graalvm/mandrel',
  'Trava': 'https://github.com/TravaOpenJDK/trava-jdk-11-dcevm',
  'Java.net': 'https://jdk.java.net/',
  'Tencent': 'https://tencent.github.io/konajdk/',
  'Gluon': 'https://gluonhq.com/',
}

const searchQuery = ref('')
const statusFilter = ref('all')
const vendorFilter = ref('all')
const categoryFilter = ref('all')
const hasLoaded = ref(false)
const errorMessage = ref('')
const expandedVendors = ref<Set<string>>(new Set())
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
const versions = computed(() => sdkStore.jdkVersions)

const uniqueVendors = computed(() => {
  const vendors = new Set(versions.value.map(v => v.vendor))
  return Array.from(vendors).sort()
})

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
      v.identifier.toLowerCase().includes(query) ||
      v.vendor.toLowerCase().includes(query)
    )
  }

  // 状态过滤
  if (statusFilter.value === 'installed') {
    result = result.filter(v => v.installed)
  } else if (statusFilter.value === 'not-installed') {
    result = result.filter(v => !v.installed)
  }

  // 供应商过滤
  if (vendorFilter.value !== 'all') {
    result = result.filter(v => v.vendor === vendorFilter.value)
  }

  // 分类过滤
  if (categoryFilter.value !== 'all') {
    result = result.filter(v => {
      // categories 是一个数组，检查是否包含指定分类
      return v.categories && v.categories.includes(categoryFilter.value)
    })
  }

  // 按版本号排序（新版本在前）
  return [...result].sort((a, b) => compareVersions(a.version, b.version))
})

// 按供应商分组
const groupedVersions = computed(() => {
  const groups = new Map<string, typeof versions.value>()

  filteredVersions.value.forEach(version => {
    const vendor = version.vendor
    if (!groups.has(vendor)) {
      groups.set(vendor, [])
    }
    groups.get(vendor)!.push(version)
  })

  // 转换为数组并添加统计信息
  return Array.from(groups.entries()).map(([vendor, versions]) => ({
    vendor,
    versions,
    installedCount: versions.filter(v => v.installed).length
  })).sort((a, b) => {
    // Temurin 始终排在第一位（推荐）
    if (a.vendor === 'Temurin') return -1
    if (b.vendor === 'Temurin') return 1

    // 其他供应商：按已安装数量降序，然后按供应商名称升序
    if (b.installedCount !== a.installedCount) {
      return b.installedCount - a.installedCount
    }
    return a.vendor.localeCompare(b.vendor)
  })
})

// 切换供应商展开/折叠状态
function toggleVendor(vendor: string) {
  if (expandedVendors.value.has(vendor)) {
    expandedVendors.value.delete(vendor)
  } else {
    expandedVendors.value.add(vendor)
  }
}

// 监听分组变化，自动展开第一个分组
watch(groupedVersions, (newGroups) => {
  if (newGroups.length > 0 && expandedVendors.value.size === 0) {
    expandedVendors.value.add(newGroups[0].vendor)
  }
}, { immediate: true })

async function refreshData(forceRefresh = true) {
  await sdkStore.fetchJdkVersions(forceRefresh)
  hasLoaded.value = true

  // 添加调试日志
  console.log('JDK versions loaded:', versions.value.length)
  if (versions.value.length > 0) {
    console.log('First JDK sample:', {
      version: versions.value[0].version,
      identifier: versions.value[0].identifier,
      vendor: versions.value[0].vendor,
      installed: versions.value[0].installed,
      categories: versions.value[0].categories
    })

    const installedCount = versions.value.filter(v => v.installed).length
    console.log('Installed JDKs:', installedCount)
  }

  // 全局 store 会自动管理进度清理
}

// 清理已完成的进度 - 不再需要，由全局 store 自动管理
// function cleanupCompletedProgress() {
//   // 全局 store 会在安装完成后自动清理
// }

// 检查版本是否正在操作中
function isOperating(identifier: string): boolean {
  // identifier 格式如 "25.0.1-tem"
  // store 中的键格式是 "java-25.0.1-tem"
  return installProgressStore.isOperating(`java-${identifier}`)
}

// 获取版本的进度信息
function getProgress(identifier: string) {
  // identifier 格式如 "25.0.1-tem"
  // store 中的键格式是 "java-25.0.1-tem"
  return installProgressStore.getProgress(`java-${identifier}`)
}

// 格式化进度消息（添加国际化支持）
function formatProgressMessage(identifier: string): string {
  const progress = getProgress(identifier)
  if (!progress) {
    return t('jdk.progressProcessing')
  }

  const task = installProgressStore.tasks.get(`java-${identifier}`)
  if (!task) {
    return progress.message || t('jdk.progressProcessing')
  }

  // 根据任务状态返回国际化消息
  switch (task.status) {
    case 'downloading':
      // 如果是初始状态（0% 或 Starting...），显示"开始下载..."
      if (progress.percentage === 0 || progress.message === 'Starting...') {
        return t('jdk.progressStarting')
      }
      // 下载中显示原始消息（包含进度百分比和大小）
      return progress.message
    case 'installing':
      return t('jdk.progressInstalling')
    case 'completed':
      return t('jdk.progressCompleted') || '安装完成'
    case 'failed':
      return t('jdk.progressFailed') || '安装失败'
    default:
      return progress.message || t('jdk.progressProcessing')
  }
}

async function installJdk(version: SdkVersion) {
  console.log('installJdk clicked:', version.identifier)
  const identifier = version.identifier
  console.log('Starting install for:', identifier)

  // 使用全局 store 启动安装任务
  // 注意：version.identifier 就是完整的标识符，不需要再加 'java' 前缀
  installProgressStore.startTask('java', identifier)

  try {
    console.log('Calling downloadAndInstallJdk...')
    await sdkStore.downloadAndInstallJdk(identifier)
    console.log('Install completed successfully')

    // 只更新这个版本的状态，而不是刷新整个列表
    const versionIndex = versions.value.findIndex(v => v.identifier === identifier)
    if (versionIndex !== -1) {
      versions.value[versionIndex].installed = true
    }

    // 同时更新 store 中的数据
    await sdkStore.scanInstalledJdks()

    // 数据刷新完成后移除任务，避免UI闪烁
    installProgressStore.removeTask(`java-${identifier}`)
  } catch (e: any) {
    console.error('Install failed:', e)
    const errorDetail = extractErrorMessage(e)
    showError(`${t('jdk.installingFailed', [identifier])}: ${errorDetail}`)

    // 失败时也移除任务
    installProgressStore.removeTask(`java-${identifier}`)
  }
}

async function uninstallJdk(version: SdkVersion) {
  console.log('uninstallJdk clicked:', version.identifier)
  const identifier = version.identifier

  // 立即检查本地状态，防止快速双击
  if (localUninstallingVersions.value.has(identifier)) {
    console.log('Already uninstalling (local state), ignoring click')
    return
  }

  // 再检查全局 store 状态
  if (sdkStore.isUninstalling('java', identifier)) {
    console.log('Already uninstalling (store state), ignoring click')
    return
  }

  // 立即更新本地状态，触发按钮禁用
  localUninstallingVersions.value.add(identifier)
  localUninstallingVersions.value = new Set(localUninstallingVersions.value)

  // 卸载操作很快，不需要显示进度条，直接执行
  try {
    await sdkStore.uninstallJdk(identifier)
    console.log('Uninstall completed successfully')

    // sdkStore.uninstallJdk 内部已经刷新了数据，不需要手动更新状态
    // 这样避免了竞态条件导致的UI闪烁
  } catch (e: any) {
    console.error('Uninstall failed:', e)
    const errorDetail = extractErrorMessage(e)
    showError(`${t('jdk.uninstallFailed', [identifier])}: ${errorDetail}`)
  } finally {
    // 清理本地状态
    localUninstallingVersions.value.delete(identifier)
    localUninstallingVersions.value = new Set(localUninstallingVersions.value)
  }
}

async function setDefault(version: SdkVersion) {
  console.log('setDefault clicked:', version.identifier)
  console.log('Before setDefault - inUse status:', versions.value.find(v => v.identifier === version.identifier)?.inUse)

  try {
    await sdkStore.setDefaultJdkVersion(version.identifier)
    console.log('Set default completed successfully')

    // 刷新JDK列表以更新状态
    await refreshData()

    console.log('After refresh - inUse status:', versions.value.find(v => v.identifier === version.identifier)?.inUse)
    console.log('All versions inUse status:', versions.value.filter(v => v.installed).map(v => ({
      identifier: v.identifier,
      inUse: v.inUse
    })))
  } catch (e: any) {
    console.error('Set default failed:', e)
    const errorDetail = extractErrorMessage(e)
    showError(`${t('jdk.setDefaultFailed', [version.identifier])}: ${errorDetail}`)
  }
}

// 获取供应商官网
function getVendorWebsite(vendor: string): string | undefined {
  return vendorWebsites[vendor]
}

// 打开供应商官网
async function openVendorWebsite(vendor: string) {
  const website = getVendorWebsite(vendor)
  if (website) {
    try {
      await open(website)
    } catch (error) {
      console.error('Failed to open vendor website:', error)
    }
  }
}

onMounted(async () => {
  // 初始加载使用缓存
  await refreshData(false)
  // 事件监听器现在在全局 App.vue 中管理，无需在此处设置
})

// 不再需要 onUnmounted，因为事件监听器在全局管理
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
  gap: 24px;
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
  gap: 8px;
}

.filter-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-secondary);
}

.filter-select {
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 14px;
  color: var(--text-primary);
  background: var(--bg-secondary);
  cursor: pointer;
  min-width: 150px;
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
.jdk-tree {
  display: flex;
  flex-direction: column;
  gap: 0;
}

/* 供应商分组 */
.vendor-group {
  background: var(--bg-secondary);
  overflow: hidden;
  transition: background 0.2s;
  box-shadow: 1px 0 0 0 var(--bg-tertiary), -1px 0 0 0 var(--bg-tertiary);
}

.vendor-group + .vendor-group {
  box-shadow: 0 -0.5px 0 0 var(--border-color), 1px 0 0 0 var(--bg-tertiary), -1px 0 0 0 var(--bg-tertiary);
}

.vendor-group:first-child {
  box-shadow: 0 -0.5px 0 0 var(--border-color), 1px 0 0 0 var(--bg-tertiary), -1px 0 0 0 var(--bg-tertiary);
  border-top-left-radius: 12px;
  border-top-right-radius: 12px;
}

.vendor-group:last-child {
  box-shadow: 0 0.5px 0 0 var(--border-color), 1px 0 0 0 var(--bg-tertiary), -1px 0 0 0 var(--bg-tertiary);
  border-bottom-left-radius: 12px;
  border-bottom-right-radius: 12px;
}

/* 供应商标题 */
.vendor-header {
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

.vendor-header:hover {
  background: var(--bg-tertiary);
}

.vendor-info {
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

.vendor-name {
  margin: 0;
  font-size: 18px;
  font-weight: 700;
  color: var(--text-primary);
}

.vendor-website {
  font-size: 13px;
  color: var(--primary-color);
  text-decoration: none;
  cursor: pointer;
  padding: 3px 10px;
  border-radius: 6px;
  transition: all 0.2s;
  font-weight: 500;
  border: 1px solid var(--primary-color);
  background: transparent;
}

.vendor-website:hover {
  color: white;
  background: var(--primary-color);
}

.recommended-badge {
  padding: 4px 10px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 600;
  margin: 0 8px;
}

.version-count {
  font-size: 14px;
  color: var(--text-secondary);
  font-weight: 500;
}

.vendor-stats {
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

/* 版本列表容器 */
.vendor-versions {
  background: var(--bg-secondary);
  padding: 4px 8px;
}

/* JDK 项（在分组内部） */
.jdk-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 16px;
  margin: 2px 0;
  background: var(--bg-secondary);
  border-radius: 6px;
  transition: transform 0.2s, box-shadow 0.2s;
}

.jdk-item:hover {
  transform: translateX(2px);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
}

.jdk-info {
  flex: 1;
}

.jdk-version {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 2px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.jdk-identifier {
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 2px;
}

.jdk-dist {
  font-size: 12px;
  color: var(--text-tertiary);
}

.badge {
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
}

.badge-active {
  background: #dbeafe;
  color: #1e40af;
}

.badge-installed {
  background: #d1fae5;
  color: #065f46;
}

.jdk-actions {
  display: flex;
  gap: 6px;
}

.action-btn {
  padding: 6px 12px;
  border: none;
  border-radius: 4px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.install-btn {
  background: var(--success-color);
  color: white;
}

.install-btn:hover:not(:disabled) {
  background: var(--success-hover);
}

.use-btn {
  background: var(--primary-color);
  color: white;
}

.use-btn:hover:not(:disabled) {
  background: var(--primary-hover);
}

.default-btn {
  background: var(--gray-color);
  color: white;
  cursor: not-allowed;
}

.uninstall-btn {
  background: var(--danger-color);
  color: white;
}

.uninstall-btn:hover:not(:disabled) {
  background: var(--danger-hover);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.progress-container {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 200px;
  flex-shrink: 0;
  flex-grow: 0;
}

.progress-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.progress-bar {
  width: 100%;
  height: 8px;
  background: var(--border-color);
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--primary-color), #3b82f6);
  transition: width 0.3s ease;
  border-radius: 4px;
}

.progress-text {
  font-size: 11px;
  color: var(--text-secondary);
  font-weight: 500;
  min-height: 18px;
  line-height: 18px;
  white-space: nowrap;
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
