import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export interface ProgressInfo {
  type: 'download' | 'install'
  percentage: number
  message: string
}

export interface InstallTask {
  candidate: string
  version: string
  identifier: string
  status: 'downloading' | 'installing' | 'completed' | 'failed'
  progress: ProgressInfo
}

export const useInstallProgressStore = defineStore('installProgress', () => {
  // 当前正在进行的安装任务 Map: identifier -> InstallTask
  const tasks = ref<Map<string, InstallTask>>(new Map())

  // 事件监听器引用
  let unlistenDownload: UnlistenFn | null = null
  let unlistenInstall: UnlistenFn | null = null
  let unlistenComplete: UnlistenFn | null = null

  // 初始化事件监听器（在应用启动时调用一次）
  async function initEventListeners() {
    // 如果已经初始化，先清理
    if (unlistenDownload || unlistenInstall || unlistenComplete) {
      await cleanupEventListeners()
    }

    // 监听下载进度
    unlistenDownload = await listen<any>('download-progress', (event) => {
      const { candidate, version, percentage, downloaded, total } = event.payload
      const identifier = `${candidate}-${version}`

      const task = tasks.value.get(identifier)
      if (task) {
        task.status = 'downloading'
        task.progress = {
          type: 'download',
          percentage: Math.round(percentage),
          message: `${formatBytes(downloaded)} / ${formatBytes(total)} (${Math.round(percentage)}%)`
        }
        // 触发响应式更新
        tasks.value = new Map(tasks.value)
      }
    })

    // 监听安装进度
    unlistenInstall = await listen<any>('install-progress', (event) => {
      const { candidate, version, message } = event.payload
      const identifier = `${candidate}-${version}`

      const task = tasks.value.get(identifier)
      if (task) {
        task.status = 'installing'
        // 安装时保持下载完成的进度条状态，只更新消息
        task.progress = {
          type: 'install',
          percentage: task.progress.percentage, // 保持当前百分比不变
          message: message || 'Installing...'
        }
        // 触发响应式更新
        tasks.value = new Map(tasks.value)
      }
    })

    // 监听安装完成
    unlistenComplete = await listen<any>('install-complete', (event) => {
      const { candidate, version, success, message } = event.payload
      const identifier = `${candidate}-${version}`

      const task = tasks.value.get(identifier)
      if (task) {
        if (success) {
          // 成功时标记为完成状态，但不自动移除任务
          // 让前端组件在数据刷新完成后手动移除，避免UI闪烁
          task.status = 'completed'
          task.progress = {
            type: 'install',
            percentage: 100,
            message: message || 'Completed'
          }
          // 触发响应式更新
          tasks.value = new Map(tasks.value)
        } else {
          task.status = 'failed'
          task.progress = {
            type: 'install',
            percentage: 100,
            message: message || 'Failed'
          }
          // 触发响应式更新
          tasks.value = new Map(tasks.value)

          // 5秒后移除失败的任务
          setTimeout(() => {
            removeTask(identifier)
          }, 5000)
        }
      }
    })
  }

  // 清理事件监听器
  async function cleanupEventListeners() {
    if (unlistenDownload) {
      unlistenDownload()
      unlistenDownload = null
    }
    if (unlistenInstall) {
      unlistenInstall()
      unlistenInstall = null
    }
    if (unlistenComplete) {
      unlistenComplete()
      unlistenComplete = null
    }
  }

  // 开始一个安装任务
  function startTask(candidate: string, version: string) {
    const identifier = `${candidate}-${version}`
    console.log('[InstallProgressStore] Starting task:', identifier)
    tasks.value.set(identifier, {
      candidate,
      version,
      identifier,
      status: 'downloading',
      progress: {
        type: 'download',
        percentage: 0,
        message: 'Starting...'
      }
    })
    // 触发响应式更新
    tasks.value = new Map(tasks.value)
    console.log('[InstallProgressStore] Task started, total tasks:', tasks.value.size)
  }

  // 移除一个任务
  function removeTask(identifier: string) {
    tasks.value.delete(identifier)
    // 触发响应式更新
    tasks.value = new Map(tasks.value)
  }

  // 检查某个版本是否正在操作中
  function isOperating(identifier: string): boolean {
    return tasks.value.has(identifier)
  }

  // 获取某个版本的进度信息
  function getProgress(identifier: string): ProgressInfo | undefined {
    return tasks.value.get(identifier)?.progress
  }

  // 获取某个版本的任务状态
  function getTaskStatus(identifier: string): string | undefined {
    return tasks.value.get(identifier)?.status
  }

  // 格式化字节数
  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
  }

  // 清除所有任务（用于应用重启或重置）
  function clearAllTasks() {
    tasks.value.clear()
    tasks.value = new Map()
  }

  return {
    tasks,
    initEventListeners,
    cleanupEventListeners,
    startTask,
    removeTask,
    isOperating,
    getProgress,
    getTaskStatus,
    clearAllTasks
  }
})
