import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import i18n, { updateTrayLanguage } from './i18n'
import './style.css'
import App from './App.vue'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)
app.use(i18n)
app.mount('#app')

// 初始化托盘菜单语言
updateTrayLanguage(i18n.global.locale.value)
