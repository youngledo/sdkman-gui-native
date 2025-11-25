import { createRouter, createWebHistory } from 'vue-router'
import Home from '../views/Home.vue'
import JdkManagement from '../views/JdkManagement.vue'
import SdkBrowser from '../views/SdkBrowser.vue'
import SdkDetail from '../views/SdkDetail.vue'
import Settings from '../views/Settings.vue'

const routes = [
  {
    path: '/',
    name: 'Home',
    component: Home
  },
  {
    path: '/jdk',
    name: 'JdkManagement',
    component: JdkManagement
  },
  {
    path: '/sdk',
    name: 'SdkBrowser',
    component: SdkBrowser
  },
  {
    path: '/sdk/:candidate',
    name: 'SdkDetail',
    component: SdkDetail,
    props: true
  },
  {
    path: '/settings',
    name: 'Settings',
    component: Settings
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
