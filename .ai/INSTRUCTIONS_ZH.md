# SDKMAN GUI - AI开发指南

## 项目概述

这是一个基于SDKMAN的跨平台GUI应用程序，使用**Tauri 2.0 + Vue 3 + Rust**构建。

**核心功能**:
- 侧边栏导航：首页、JDK、SDK、设置页面
- 使用自定义CSS变量和主题的现代UI
- 完整的国际化支持（中文和英文）
- 基于Rust后端的异步操作

## 技术栈

- **前端**: Vue 3 + TypeScript + Vite
- **后端**: Rust + Tauri 2.0
- **UI**: 自定义组件 + CSS变量
- **状态管理**: Pinia
- **国际化**: Vue-i18n
- **路由**: Vue Router

## 架构

### 前端 (Vue 3)

```
sdkman-gui-tauri/src/
├── components/           # 可复用UI组件
│   ├── Sidebar.vue       # 导航侧边栏
│   ├── SdkCard.vue       # SDK显示卡片
│   └── ...
├── views/                # 页面组件
│   ├── Home.vue          # 带统计的首页
│   ├── Jdk.vue           # JDK管理页面
│   ├── Sdk.vue           # SDK浏览页面
│   ├── SdkDetail.vue     # SDK版本详情
│   └── Settings.vue      # 设置页面
├── stores/               # Pinia状态管理
│   └── sdkStore.ts       # SDK数据存储
├── locales/              # i18n翻译文件
│   ├── en.json           # 英文翻译
│   └── zh.json           # 中文翻译
├── router/               # Vue Router配置
│   └── index.ts
└── App.vue               # 根组件（主题管理）
```

### 后端 (Rust)

```
sdkman-gui-tauri/src-tauri/src/
├── main.rs               # 应用入口点
├── lib.rs                # 库导出
├── sdkman.rs             # SDKMAN CLI封装
├── config.rs             # 配置管理
└── ...
```

## 🚨 关键开发规则 🚨

### 1. 目录结构规范 - 必须遵循 ⚠️

**黄金规则：所有应用配置和缓存数据必须存储在统一的配置目录！**

#### 应用数据存储位置

**macOS/Linux**:
```
~/.config/sdkman-gui/          # ✅ 统一配置目录
├── config.json                # 应用配置文件
└── cache/                     # 缓存目录
    ├── jdk_versions.json      # JDK版本列表缓存
    ├── sdk_candidates.json    # SDK候选者列表缓存
    └── {candidate}_versions.json  # 各个SDK的版本列表缓存
```

**Windows**:
```
C:\Users\<username>\.config\sdkman-gui\
├── config.json
└── cache\
    ├── jdk_versions.json
    ├── sdk_candidates.json
    └── {candidate}_versions.json
```

#### ❌ 禁止使用的路径

```rust
// ❌ 绝对不要使用这些路径
let path = home.join(".sdkman-gui");           // 禁止！
let path = home.join(".cache/sdkman-gui");     // 禁止！
let path = home.join("Library/Application Support/sdkman-gui");  // 禁止！
```

#### ✅ 正确的路径获取方式

```rust
// ✅ 配置文件路径
use crate::config::AppConfig;
let config = AppConfig::load()?;  // 自动使用 ~/.config/sdkman-gui/config.json

// ✅ 缓存目录路径
use crate::cache;
let cache_dir = cache::get_cache_dir()?;  // ~/.config/sdkman-gui/cache/
```

#### 添加新缓存文件时的步骤

1. 在 `src-tauri/src/cache.rs` 中添加文件名常量：
```rust
const NEW_CACHE_FILE: &str = "new_cache.json";
```

2. 实现缓存函数（必须使用 `get_cache_dir()`）：
```rust
pub fn cache_new_data(data: &[SomeType]) -> Result<()> {
    let cache_dir = get_cache_dir()?;  // ✅ 使用统一的目录获取函数
    let cache_file = cache_dir.join(NEW_CACHE_FILE);
    // 实现缓存逻辑...
}

pub fn read_new_data_cache() -> Result<Option<Vec<SomeType>>> {
    let cache_dir = get_cache_dir()?;  // ✅ 使用统一的目录获取函数
    let cache_file = cache_dir.join(NEW_CACHE_FILE);
    // 实现读取逻辑...
}
```

#### 缓存策略规范

所有需要从网络获取的数据都应该支持缓存：

```rust
#[tauri::command]
pub async fn get_some_data(
    force_refresh: bool,  // ✅ 必须支持此参数
    client: State<'_, Arc<Mutex<SdkmanApiClient>>>
) -> Result<Vec<SomeType>, String> {
    // 优先使用缓存
    if !force_refresh {
        if let Ok(Some(cached)) = cache::read_some_data_cache() {
            println!("Using cached data");
            return Ok(cached);
        }
    }

    // 从API获取并缓存
    println!("Fetching from API");
    let data = fetch_from_api(&client).await?;
    let _ = cache::cache_some_data(&data);  // 缓存结果
    Ok(data)
}
```

前端调用约定：
```typescript
// ✅ 初始加载使用缓存
onMounted(async () => {
    await fetchData(false)  // forceRefresh = false
})

// ✅ 刷新按钮强制更新
async function onRefresh() {
    await fetchData(true)   // forceRefresh = true
}
```

### 2. 国际化 (I18n) - 必须遵循

**黄金规则：所有面向用户的文本必须使用i18n。禁止硬编码字符串！**

#### Vue模板用法

```vue
<template>
  <!-- ❌ 错误 -->
  <h1>欢迎使用SDKMAN</h1>
  <button>安装</button>

  <!-- ✅ 正确 -->
  <h1>{{ $t('home.welcome') }}</h1>
  <button>{{ $t('jdk.action.install') }}</button>
</template>
```

#### 添加新翻译

1. 添加到 `src/locales/en.json`:
```json
{
  "home": {
    "welcome": "Welcome to SDKMAN"
  },
  "jdk": {
    "action": {
      "install": "Install"
    }
  }
}
```

2. 添加到 `src/locales/zh.json`:
```json
{
  "home": {
    "welcome": "欢迎使用SDKMAN"
  },
  "jdk": {
    "action": {
      "install": "安装"
    }
  }
}
```

### 2. 代码风格与约定

#### TypeScript命名
- 组件: `PascalCase` (SdkCard.vue, HomeView.vue)
- 函数: `camelCase` (loadStatistics, setupTheme)
- 变量: `camelCase` (sdkList, isLoading)
- 常量: `UPPER_SNAKE_CASE` (DEFAULT_LOCALE, MAX_RETRY)

#### 中英文间距
**中英文字符之间不要有空格**（用户的明确要求）

```typescript
// ✅ 正确
const message = "欢迎使用SDKMAN"

// ❌ 错误
const message = "欢迎使用 SDKMAN"
```

#### Vue组件结构

```vue
<template>
  <!-- 模板内容 -->
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

// 响应式状态
const isLoading = ref(false)

// 函数
async function loadData() {
  // ...
}

// 生命周期
onMounted(() => {
  loadData()
})
</script>

<style scoped>
/* 组件样式 */
</style>
```

### 3. Tauri命令集成

#### 定义Rust命令

```rust
// src-tauri/src/lib.rs
#[tauri::command]
async fn get_sdk_list() -> Result<Vec<SdkInfo>, String> {
    // 实现
}

// 在builder中注册
.invoke_handler(tauri::generate_handler![get_sdk_list])
```

#### 从Vue调用

```typescript
import { invoke } from '@tauri-apps/api/core'

async function fetchSdkList() {
  try {
    const list = await invoke<SdkInfo[]>('get_sdk_list')
    return list
  } catch (e) {
    console.error('获取SDK列表失败:', e)
    throw e
  }
}
```

### 4. 主题管理

应用支持三种主题模式：亮色、暗色和自动。

#### CSS变量

```css
:root {
  --bg-primary: #ffffff;
  --bg-secondary: #f5f5f5;
  --text-primary: #1a1a1a;
  --text-secondary: #666666;
  --primary-color: #007aff;
  /* ... 更多变量 */
}

[data-theme="dark"] {
  --bg-primary: #1a1a1a;
  --bg-secondary: #2d2d2d;
  --text-primary: #ffffff;
  --text-secondary: #a0a0a0;
  --primary-color: #0a84ff;
}
```

#### 使用主题变量

```vue
<style scoped>
.card {
  background: var(--bg-secondary);
  color: var(--text-primary);
}
</style>
```

### 5. Pinia状态管理

```typescript
// stores/sdkStore.ts
import { defineStore } from 'pinia'

export const useSdkStore = defineStore('sdk', {
  state: () => ({
    sdkList: [] as SdkInfo[],
    isLoading: false,
    error: null as string | null,
  }),

  actions: {
    async fetchSdkList() {
      this.isLoading = true
      try {
        this.sdkList = await invoke<SdkInfo[]>('get_sdk_list')
      } catch (e) {
        this.error = String(e)
      } finally {
        this.isLoading = false
      }
    }
  }
})
```

### 6. 错误处理

#### 前端

```typescript
try {
  const result = await invoke('some_command')
  // 处理成功
} catch (e) {
  console.error('操作失败:', e)
  // 显示用户友好的错误通知
}
```

#### 后端 (Rust)

```rust
#[tauri::command]
async fn install_sdk(candidate: String, version: String) -> Result<(), String> {
    // 使用Result类型进行正确的错误处理
    execute_sdkman_command(&format!("sdk install {} {}", candidate, version))
        .map_err(|e| e.to_string())
}
```

### 7. SDKMAN CLI集成

所有SDKMAN命令都通过Rust后端执行：

```rust
// 执行SDKMAN命令
fn execute_sdkman_command(command: &str) -> Result<String, Box<dyn Error>> {
    let full_command = format!(
        "source ~/.sdkman/bin/sdkman-init.sh && {}",
        command
    );

    let output = Command::new("bash")
        .args(["-c", &full_command])
        .output()?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
```

常用SDKMAN命令：
- `sdk list` - 列出所有候选
- `sdk list java` - 列出Java版本
- `sdk install java 21` - 安装Java 21
- `sdk default java 21` - 设置默认Java
- `sdk uninstall java 21` - 卸载Java 21

## 开发工作流

### 运行开发模式

```bash
cd sdkman-gui-tauri
npm install
npm run tauri dev
```

### 构建生产版本

```bash
npm run tauri build
```

### 项目结构

```
sdkman-gui/
├── sdkman-gui-tauri/          # 主要的Tauri应用
│   ├── src/                   # Vue 3前端
│   ├── src-tauri/             # Rust后端
│   ├── package.json           # Node.js依赖
│   └── vite.config.ts         # Vite配置
├── docs/                      # 文档和图片
├── README.md                  # 英文README
├── README_ZH.md               # 中文README
└── AI_GUIDE_ZH.md             # 本文件
```

## 常见反模式要避免

### ❌ 硬编码文本
```vue
<template>
  <span>欢迎</span>  <!-- 绝对不要这样做 -->
</template>
```

### ❌ 使用同步操作阻塞UI
```typescript
// 错误 - 阻塞UI
const result = someHeavyOperation()
```

### ❌ 忽略错误处理
```typescript
// 错误 - 没有错误处理
const data = await invoke('get_data')
```

### ❌ 在生产环境使用console.log
```typescript
console.log("调试信息")  // 生产环境中删除
```

## 关键文件参考

**配置文件：**
- `sdkman-gui-tauri/tauri.conf.json` - Tauri配置
- `sdkman-gui-tauri/package.json` - Node.js依赖
- `sdkman-gui-tauri/vite.config.ts` - Vite构建配置
- `sdkman-gui-tauri/tsconfig.json` - TypeScript配置

**主入口点：**
- `sdkman-gui-tauri/src/main.ts` - Vue应用入口
- `sdkman-gui-tauri/src-tauri/src/main.rs` - Rust入口点

**关键组件：**
- `src/App.vue` - 根组件（主题管理）
- `src/views/Home.vue` - 首页
- `src/views/Jdk.vue` - JDK管理
- `src/views/Settings.vue` - 设置页面

---

**记住：在处理此项目时应参考此AI_GUIDE_ZH.md文件。始终遵循这些规则以保持代码质量和一致性！**
