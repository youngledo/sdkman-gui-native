<template>
  <div class="sidebar">
    <!-- Logo 区域 -->
    <div class="sidebar-header">
      <img
        :src="sdkmanLogo"
        alt="SDKMAN Logo"
        class="logo"
        @click="navigateTo('/')"
      />
    </div>

    <!-- 导航菜单 -->
    <nav class="sidebar-nav">
      <router-link to="/" class="nav-button" active-class="nav-button-active">
        <svg class="nav-icon icon-home" width="28" height="28" viewBox="0 0 24 24" fill="currentColor">
          <path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"/>
        </svg>
        <span>{{ $t('nav.home') }}</span>
      </router-link>

      <router-link to="/jdk" class="nav-button" active-class="nav-button-active">
        <svg class="nav-icon icon-jdk" width="28" height="28" viewBox="0 0 24 24" fill="currentColor">
          <path d="M2,21H20V19H2M20,8H18V5H20M20,3H4V13A4,4 0 0,0 8,17H14A4,4 0 0,0 18,13V10H20A2,2 0 0,0 22,8V5C22,3.89 21.1,3 20,3Z"/>
        </svg>
        <span>{{ $t('nav.jdk') }}</span>
      </router-link>

      <router-link to="/sdk" class="nav-button" active-class="nav-button-active">
        <svg class="nav-icon icon-sdk" width="28" height="28" viewBox="0 0 24 24" fill="currentColor">
          <path d="M21 16.5c0 .38-.21.71-.53.88l-7.9 4.44c-.16.12-.36.18-.57.18-.21 0-.41-.06-.57-.18l-7.9-4.44A.99.99 0 0 1 3 16.5v-9c0-.38.21-.71.53-.88l7.9-4.44c.16-.12.36-.18.57-.18.21 0 .41.06.57.18l7.9 4.44c.32.17.53.5.53.88v9M12 4.15L6.04 7.5 12 10.85l5.96-3.35L12 4.15M5 15.91l6 3.38v-6.71L5 9.21v6.7m14 0v-6.7l-6 3.37v6.71l6-3.38z"/>
        </svg>
        <span>{{ $t('nav.sdk') }}</span>
      </router-link>

      <div class="spacer"></div>

      <div class="sidebar-separator"></div>

      <router-link to="/settings" class="nav-button" active-class="nav-button-active">
        <svg class="nav-icon icon-settings" width="28" height="28" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 15.5A3.5 3.5 0 0 1 8.5 12 3.5 3.5 0 0 1 12 8.5a3.5 3.5 0 0 1 3.5 3.5 3.5 3.5 0 0 1-3.5 3.5m7.43-2.53c.04-.32.07-.64.07-.97 0-.33-.03-.66-.07-1l2.11-1.63c.19-.15.24-.42.12-.64l-2-3.46c-.12-.22-.39-.31-.61-.22l-2.49 1c-.52-.39-1.06-.73-1.69-.98l-.37-2.65A.506.506 0 0 0 14 2h-4c-.25 0-.46.18-.5.42l-.37 2.65c-.63.25-1.17.59-1.69.98l-2.49-1c-.22-.09-.49 0-.61.22l-2 3.46c-.13.22-.07.49.12.64L4.57 11c-.04.34-.07.67-.07 1 0 .33.03.65.07.97l-2.11 1.66c-.19.15-.25.42-.12.64l2 3.46c.12.22.39.3.61.22l2.49-1.01c.52.4 1.06.74 1.69.99l.37 2.65c.04.24.25.42.5.42h4c.25 0 .46-.18.5-.42l.37-2.65c.63-.26 1.17-.59 1.69-.99l2.49 1.01c.22.08.49 0 .61-.22l2-3.46c.12-.22.07-.49-.12-.64l-2.11-1.66z"/>
        </svg>
        <span>{{ $t('nav.settings') }}</span>
      </router-link>
    </nav>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import sdkmanLogo from '@/assets/images/sdkman-logo.png'

const router = useRouter()

const navigateTo = (path: string) => {
  router.push(path)
}
</script>

<style scoped>
.sidebar {
  width: 220px;
  min-width: 220px;
  max-width: 220px;
  height: 100vh;
  background: var(--bg-secondary);
  color: var(--text-primary);
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border-color);
}

.sidebar-header {
  padding: 30px 20px 20px;
  display: flex;
  justify-content: center;
  align-items: center;
}

.logo {
  height: 96px;
  cursor: pointer;
  transition: transform 0.2s;
}

.logo:hover {
  transform: scale(1.05);
}

.sidebar-nav {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 10px 12px;
  gap: 4px;
}

.nav-button {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 18px;
  border-radius: 8px;
  color: var(--text-secondary);
  text-decoration: none;
  font-size: 17px;
  font-weight: 500;
  transition: all 0.2s;
  cursor: pointer;
}

.nav-button:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.nav-icon {
  opacity: 0.9;
}

.nav-icon.icon-home {
  color: #6366f1;
}

.nav-icon.icon-jdk {
  color: #f59e0b;
}

.nav-icon.icon-sdk {
  color: #10b981;
}

.nav-icon.icon-settings {
  color: #6b7280;
}

.spacer {
  flex: 1;
}

.sidebar-separator {
  height: 1px;
  background: var(--border-color);
  margin: 10px 0;
}
</style>
