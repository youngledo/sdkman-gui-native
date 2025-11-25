import { createI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import en from './locales/en.json'
import zh from './locales/zh.json'

// 检测系统语言
function getSystemLanguage(): string {
  const browserLang = navigator.language || (navigator as any).userLanguage

  // 如果是中文（包括 zh, zh-CN, zh-TW 等），返回 'zh'
  if (browserLang.toLowerCase().startsWith('zh')) {
    return 'zh'
  }

  // 默认返回英语
  return 'en'
}

// 获取初始语言设置
const systemLanguage = getSystemLanguage()

const i18n = createI18n({
  legacy: false,
  locale: systemLanguage, // 使用系统语言作为初始值
  fallbackLocale: 'en',
  messages: {
    en,
    zh,
  },
})

// 更新托盘菜单的语言
export async function updateTrayLanguage(locale: string) {
  const messages = i18n.global.messages.value[locale as 'en' | 'zh'] as any
  if (messages && messages.tray) {
    try {
      // 构建托盘菜单项配置
      const menuItems = [
        { id: 'quit', label: messages.tray.quit }
      ]

      await invoke('update_tray_menu', { items: menuItems })
    } catch (error) {
      console.error('Failed to update tray menu:', error)
    }
  }
}

export default i18n
