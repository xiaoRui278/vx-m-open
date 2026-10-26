<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAppStore, GITHUB_URL, SITE_URL } from "./stores/appStore";
import { icons } from "./icons";
import { t } from "./i18n";
import { initTheme } from "./prefs";
import { openUrl } from "./api";
import logo from "./assets/logo.png";
import InstancesView from "./views/InstancesView.vue";
import SettingsView from "./views/SettingsView.vue";

const store = useAppStore();
const active = ref<"instances" | "settings">("instances");

const navItems = computed(() => [
  { key: "instances" as const, label: t("navInstances"), icon: icons.instances },
  { key: "settings" as const, label: t("navSettings"), icon: icons.settings },
]);

const views = {
  instances: InstancesView,
  settings: SettingsView,
};

import { isWindows } from "./platform";
const updateDismissed = ref(false);
const updating = ref(false);

async function updateNow() {
  updating.value = true;
  try {
    await store.updateNow();
  } finally {
    updating.value = false;
  }
}

function openReleasePage() {
  void openUrl(SITE_URL);
}

function openGitHub() {
  void openUrl(GITHUB_URL);
}

// 浏览器预览没有窗口 API，静默降级
function caption(action: "minimize" | "toggleMaximize" | "close") {
  try {
    const w = getCurrentWindow();
    void (action === "minimize" ? w.minimize() : action === "close" ? w.close() : w.toggleMaximize());
  } catch {
    /* 预览环境 */
  }
}

onMounted(() => {
  document.documentElement.dataset.platform = isWindows ? "windows" : "macos";
  initTheme();
  store.init();
});

const platformFoot = computed(
  () => `v${store.appVersion || "1.0.0"} · ${isWindows ? "Windows" : "macOS"}`,
);
</script>

<template>
  <!-- Windows 自绘 Fluent 标题栏 -->
  <div v-if="isWindows" class="titlebar" data-tauri-drag-region>
    <div class="titlebar-app" data-tauri-drag-region>
      <img class="brand-mark" :src="logo" alt="">
      VXMOpen
    </div>
    <div class="titlebar-spacer" data-tauri-drag-region></div>
    <div class="caption-group">
      <button class="caption-btn" aria-label="minimize" @click="caption('minimize')" v-html="icons.captionMin"></button>
      <button class="caption-btn" aria-label="maximize" @click="caption('toggleMaximize')" v-html="icons.captionMax"></button>
      <button class="caption-btn close" :aria-label="t('close')" @click="caption('close')" v-html="icons.captionClose"></button>
    </div>
  </div>

  <div class="app">
    <!-- 侧边栏（macOS）/ NavigationView（Windows） -->
    <aside class="sidebar">
      <div class="brand">
        <img class="brand-mark" :src="logo" alt="">
        <span class="brand-name">VXMOpen</span>
      </div>
      <nav class="nav" :aria-label="t('navInstances')">
        <button
          v-for="item in navItems"
          :key="item.key"
          class="nav-item"
          :class="{ active: active === item.key }"
          @click="active = item.key"
        >
          <span aria-hidden="true" v-html="item.icon"></span>
          {{ item.label }}
        </button>
      </nav>
      <div class="sidebar-foot">
        <span>{{ platformFoot }}</span>
        <button
          class="gh-btn"
          type="button"
          :title="t('github')"
          :aria-label="t('github')"
          @click="openGitHub"
          v-html="icons.github"
        ></button>
      </div>
    </aside>

    <!-- 主区 -->
    <main class="app-main">
      <div v-if="store.updateAvailable && !updateDismissed" class="update-strip">
        <span aria-hidden="true" v-html="icons.alert"></span>
        <span class="head-fill">{{ t("updateBanner", { version: store.updateVersion }) }}</span>
        <button v-if="store.updateViaUpdater" class="btn sm" :disabled="updating || store.busy" @click="updateNow">
          {{ t("updateInstallNow") }}
        </button>
        <button v-else class="btn sm" @click="openReleasePage">{{ t("updateDownload") }}</button>
        <button class="dismiss" :aria-label="t('cancel')" @click="updateDismissed = true" v-html="icons.x"></button>
      </div>
      <component :is="views[active]" />
    </main>

    <!-- Toast -->
    <div
      v-if="store.message"
      class="toast"
      :class="store.messageType"
      @click="store.clearMessage()"
    >
      <span aria-hidden="true" v-html="store.messageType === 'error' ? icons.alert : icons.check"></span>
      {{ store.message }}
    </div>
  </div>
</template>
