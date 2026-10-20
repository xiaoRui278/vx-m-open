<script setup lang="ts">
import { computed, ref } from "vue";
import { useAppStore, GITHUB_URL } from "../stores/appStore";
import { locale, setLocale, t, type Locale } from "../i18n";
import { setTheme, theme, type Theme } from "../prefs";
import { openUrl } from "../api";

const store = useAppStore();
const isWindows = navigator.userAgent.includes("Windows");

const langOptions = computed(() => [
  { value: "zh" as Locale, label: "中文" },
  { value: "en" as Locale, label: "English" },
]);

const themeOptions = computed(() => [
  { value: "system" as Theme, label: t("themeSystem") },
  { value: "light" as Theme, label: t("themeLight") },
  { value: "dark" as Theme, label: t("themeDark") },
]);

// ---- 检查更新 ----
const checking = ref(false);
const checkResult = ref<"" | "ok" | "new" | "fail">("");

async function checkUpdate() {
  checking.value = true;
  checkResult.value = "";
  const hasNew = await store.checkUpdate();
  checkResult.value = hasNew ? "new" : "ok";
  checking.value = false;
}

function openGitHub() {
  void openUrl(GITHUB_URL);
}

function openReleasePage() {
  void openUrl(`${GITHUB_URL}/releases/latest`);
}

const versionText = computed(() => {
  const suffix = isWindows ? t("versionSuffixWin") : t("versionSuffixMac");
  return `v${store.appVersion || "1.0.0"}${suffix}`;
});
</script>

<template>
  <!-- 页头 -->
  <header class="page-head">
    <div class="head-fill">
      <h1 class="page-title">{{ t("settingsTitle") }}</h1>
      <p class="page-sub">{{ t("settingsSub") }}</p>
    </div>
  </header>

  <div class="content">
    <!-- 常规 -->
    <div class="settings-group">
      <div class="setting-row">
        <span class="setting-key">{{ t("wechatPath") }}</span>
        <span class="setting-val"><span class="path-text">{{ store.info?.path ?? "—" }}</span></span>
      </div>
      <div v-if="!isWindows" class="setting-row">
        <span class="setting-key">{{ t("copyLocation") }}</span>
        <span class="setting-val">{{ t("copyLocationVal") }}</span>
      </div>
      <div v-else class="setting-row">
        <span class="setting-key">{{ t("detectMethod") }}</span>
        <span class="setting-val">{{ t("detectMethodVal") }}</span>
      </div>
      <div v-if="isWindows" class="setting-row">
        <span class="setting-key">{{ t("dataLocation") }}</span>
        <span class="setting-val">{{ t("dataLocationVal") }}</span>
      </div>
      <div class="setting-row">
        <span class="setting-key">{{ t("language") }}</span>
        <span class="setting-val">
          <select class="setting-select" :value="locale" @change="setLocale(($event.target as HTMLSelectElement).value as Locale)">
            <option v-for="opt in langOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
          </select>
        </span>
      </div>
      <div class="setting-row">
        <span class="setting-key">{{ t("theme") }}</span>
        <span class="setting-val">
          <select class="setting-select" :value="theme" @change="setTheme(($event.target as HTMLSelectElement).value as Theme)">
            <option v-for="opt in themeOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
          </select>
        </span>
      </div>
    </div>

    <!-- 关于 -->
    <div class="settings-group">
      <div class="setting-row">
        <span class="setting-key">{{ t("version") }}</span>
        <span class="setting-val">{{ versionText }}</span>
      </div>
      <div class="setting-row">
        <span class="setting-key">{{ t("checkUpdate") }}</span>
        <span class="setting-val">
          <template v-if="checking"><span class="spinner" style="display: inline-block; vertical-align: -2px"></span> {{ t("checking") }}</template>
          <template v-else-if="store.updateAvailable">
            {{ t("updateAvailable", { version: store.updateVersion }) }}
            <button class="btn sm" style="margin-left: 8px" @click="openReleasePage">{{ t("updateDownload") }}</button>
          </template>
          <template v-else-if="checkResult === 'ok'">{{ t("upToDate") }}</template>
          <template v-else-if="checkResult === 'fail'">{{ t("checkFailed") }}</template>
          <button class="btn sm" style="margin-left: 8px" :disabled="checking" @click="checkUpdate">
            {{ t("checkUpdate") }}
          </button>
        </span>
      </div>
      <div class="setting-row">
        <span class="setting-key">{{ t("github") }}</span>
        <span class="setting-val">
          <button class="link-btn" @click="openGitHub">{{ GITHUB_URL }}</button>
        </span>
      </div>
      <div class="setting-row">
        <span class="setting-key">{{ t("positioning") }}</span>
        <span class="setting-val">{{ isWindows ? t("positioningWinVal") : t("positioningMacVal") }}</span>
      </div>
      <div class="setting-row">
        <span class="setting-key">{{ t("disclaimer") }}</span>
        <span class="setting-val">{{ t("disclaimerVal") }}</span>
      </div>
    </div>
  </div>
</template>
