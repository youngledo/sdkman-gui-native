<template>
  <div class="main-layout">
    <Sidebar />
    <main class="content-area">
      <router-view />
    </main>
  </div>
</template>

<script setup lang="ts">
import Sidebar from './Sidebar.vue'
</script>

<style scoped>
.main-layout {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

.content-area {
  flex: 1;
  overflow-y: auto;
  background: var(--bg-primary);
}
</style>
