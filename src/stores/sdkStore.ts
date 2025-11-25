import {defineStore} from 'pinia'
import {invoke} from '@tauri-apps/api/core'
import {ref} from 'vue'

export interface SdkVersion {
  version: string
  vendor: string
  dist: string
  status: string
  identifier: string
  installed: boolean
  inUse: boolean  // 匹配 Rust 后端的 camelCase 命名
  isDefault: boolean  // 匹配 Rust 后端的 camelCase 命名
  categories: string[]  // JDK分类数组：JDK, JAVAFX, NIK
  candidate: string
}

export interface Sdk {
  candidate: string
  name: string
  description: string
  category: string
  website: string | null
}

export interface Statistics {
  jdk_installed: number
  jdk_available: number
  sdk_installed: number
  sdk_available: number
}

export const useSdkStore = defineStore('sdk', () => {
  // State
  const jdkVersions = ref<SdkVersion[]>([])
  const sdkCandidates = ref<Sdk[]>([])
  const statistics = ref<Statistics>({
    jdk_installed: 0,
    jdk_available: 0,
    sdk_installed: 0,
    sdk_available: 0,
  })
  const installedJdkVersions = ref<string[]>([])
  const currentJdkVersion = ref<string | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 跟踪正在进行的操作，防止重复点击
  const uninstallingVersions = ref<Set<string>>(new Set())

  // Actions
  async function fetchJdkVersions(forceRefresh = false) {
    loading.value = true
    error.value = null
    try {
      console.log('[fetchJdkVersions] Fetching...', forceRefresh ? '(force refresh)' : '(from cache)')
      jdkVersions.value = await invoke<SdkVersion[]>('list_jdk_versions', { forceRefresh })
      console.log('[fetchJdkVersions] Fetched', jdkVersions.value.length, 'versions.',
        'Installed:', jdkVersions.value.filter(v => v.installed).length)
    } catch (e) {
      error.value = String(e)
      console.error('Failed to fetch JDK versions:', e)
    } finally {
      loading.value = false
    }
  }

  async function fetchSdkCandidates(forceRefresh = false) {
    loading.value = true
    error.value = null
    try {
      sdkCandidates.value = await invoke<Sdk[]>('list_sdk_candidates', { forceRefresh })
    } catch (e) {
      error.value = String(e)
      console.error('Failed to fetch SDK candidates:', e)
    } finally {
      loading.value = false
    }
  }

  async function fetchStatistics() {
    loading.value = true
    error.value = null
    try {
      statistics.value = await invoke<Statistics>('get_statistics')
    } catch (e) {
      error.value = String(e)
      console.error('Failed to fetch statistics:', e)
    } finally {
      loading.value = false
    }
  }

  async function scanInstalledJdks() {
    try {
      installedJdkVersions.value = await invoke<string[]>('scan_installed_sdks', {
        candidate: 'java'
      })
    } catch (e) {
      console.error('Failed to scan installed JDKs:', e)
    }
  }

  async function getCurrentJdkVersion() {
    try {
      currentJdkVersion.value = await invoke<string | null>('get_current_sdk_version', {
        candidate: 'java'
      })
    } catch (e) {
      console.error('Failed to get current JDK version:', e)
    }
  }

  async function setDefaultJdkVersion(version: string) {
    try {
      await invoke('set_default_sdk_version', {
        candidate: 'java',
        version
      })
      currentJdkVersion.value = version
      // 刷新 JDK 版本列表以更新状态
      await fetchJdkVersions()
    } catch (e) {
      error.value = String(e)
      console.error('Failed to set default JDK version:', e)
      throw e
    }
  }

  async function downloadAndInstallJdk(version: string) {
    error.value = null
    try {
      await invoke('download_and_install_sdk', {
        candidate: 'java',
        version
      })
      // 刷新数据（不设置loading状态，避免UI阻塞）
      await Promise.all([
        fetchJdkVersions(),
        scanInstalledJdks(),
        fetchStatistics()
      ])
    } catch (e) {
      error.value = String(e)
      console.error('Failed to install JDK:', e)
      throw e
    }
  }

  async function uninstallJdk(version: string) {
    // 防止重复点击
    const key = `java:${version}`
    if (uninstallingVersions.value.has(key)) {
      console.log('Uninstall already in progress for', key)
      return
    }

    error.value = null
    uninstallingVersions.value.add(key)
    // 触发响应式更新：创建新的 Set 引用
    uninstallingVersions.value = new Set(uninstallingVersions.value)
    try {
      console.log('[uninstallJdk] Starting uninstall for:', version)
      await invoke('uninstall_sdk', {
        candidate: 'java',
        version
      })
      console.log('[uninstallJdk] Uninstall completed, refreshing data...')

      // 刷新数据（不设置loading状态，避免UI阻塞）
      await Promise.all([
        fetchJdkVersions(),
        scanInstalledJdks(),
        getCurrentJdkVersion(),
        fetchStatistics()
      ])

      console.log('[uninstallJdk] Data refreshed. Installed versions:',
        jdkVersions.value.filter(v => v.installed).map(v => v.identifier))

      // 等待 Vue 响应式更新完成，避免UI闪烁
      await new Promise(resolve => setTimeout(resolve, 50))
      console.log('[uninstallJdk] Complete')
    } catch (e) {
      error.value = String(e)
      console.error('Failed to uninstall JDK:', e)
      throw e
    } finally {
      uninstallingVersions.value.delete(key)
    }
  }

  async function listInstalledCandidates() {
    try {
      return await invoke<string[]>('list_installed_candidates')
    } catch (e) {
      console.error('Failed to list installed candidates:', e)
      return []
    }
  }

  // 通用的SDK管理方法（用于非Java的SDK）
  async function listSdkVersions(candidate: string, forceRefresh = false): Promise<SdkVersion[]> {
    try {
      return await invoke<SdkVersion[]>('list_sdk_versions', {candidate, forceRefresh})
    } catch (e) {
      console.error(`Failed to list ${candidate} versions:`, e)
      return []
    }
  }

  async function downloadAndInstallSdk(candidate: string, version: string) {
    error.value = null
    try {
      await invoke('download_and_install_sdk', {
        candidate,
        version
      })
    } catch (e) {
      error.value = String(e)
      console.error(`Failed to install ${candidate}:`, e)
      throw e
    }
  }

  async function uninstallSdk(candidate: string, version: string) {
    // 防止重复点击
    const key = `${candidate}:${version}`
    if (uninstallingVersions.value.has(key)) {
      console.log('[uninstallSdk] Uninstall already in progress for', key)
      return
    }

    error.value = null
    uninstallingVersions.value.add(key)
    // 触发响应式更新：创建新的 Set 引用
    uninstallingVersions.value = new Set(uninstallingVersions.value)
    try {
      console.log('[uninstallSdk] Starting uninstall for:', candidate, version)
      await invoke('uninstall_sdk', {
        candidate,
        version
      })
      console.log('[uninstallSdk] Uninstall completed')
    } catch (e) {
      error.value = String(e)
      console.error('[uninstallSdk] Failed to uninstall', candidate, ':', e)
      throw e
    } finally {
      uninstallingVersions.value.delete(key)
      console.log('[uninstallSdk] Removed from uninstallingVersions')
    }
  }

  async function setDefaultSdkVersion(candidate: string, version: string) {
    try {
      await invoke('set_default_sdk_version', {
        candidate,
        version
      })
    } catch (e) {
      error.value = String(e)
      console.error(`Failed to set default ${candidate} version:`, e)
      throw e
    }
  }

  // 检查是否正在卸载某个版本
  function isUninstalling(candidate: string, version: string): boolean {
    return uninstallingVersions.value.has(`${candidate}:${version}`)
  }

  return {
    // State
    jdkVersions,
    sdkCandidates,
    statistics,
    installedJdkVersions,
    currentJdkVersion,
    loading,
    error,

    // Actions
    fetchJdkVersions,
    fetchSdkCandidates,
    fetchStatistics,
    scanInstalledJdks,
    getCurrentJdkVersion,
    setDefaultJdkVersion,
    downloadAndInstallJdk,
    uninstallJdk,
    listInstalledCandidates,
    listSdkVersions,
    downloadAndInstallSdk,
    uninstallSdk,
    setDefaultSdkVersion,
    isUninstalling,
  }
})
