<script setup lang="ts">
import { computed, ref } from "vue";
import { useAppStore, SITE_URL } from "../stores/appStore";
import { locale, setLocale, t, type Locale } from "../i18n";
import { setTheme, theme, type Theme } from "../prefs";
import { openUrl } from "../api";
import { icons } from "../icons";
import { isWindows as sharedIsWindows } from "../platform";
import wechatQrImg from "../assets/donate-wechat.png";
import alipayQrImg from "../assets/donate-alipay.png";

const store = useAppStore();
const isWindows = sharedIsWindows;

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
  const result = await store.checkUpdate();
  checkResult.value = result;
  checking.value = false;
}

function openSite() {
  void openUrl(SITE_URL);
}

const versionText = computed(() => {
  const suffix = isWindows ? t("versionSuffixWin") : t("versionSuffixMac");
  return `v${store.appVersion || "1.0.0"}${suffix}`;
});

// ---- 支持作者弹框 ----
const donateOpen = ref(false);
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
            <button class="btn sm" style="margin-left: 8px" @click="openSite">{{ t("updateDownload") }}</button>
          </template>
          <template v-else-if="checkResult === 'ok'">{{ t("upToDate") }}</template>
          <template v-else-if="checkResult === 'fail'">{{ t("checkFailed") }}</template>
          <button class="btn sm" style="margin-left: 8px" :disabled="checking" @click="checkUpdate">
            {{ t("checkUpdate") }}
          </button>
        </span>
      </div>
      <div class="setting-row">
        <span class="setting-key">{{ t("website") }}</span>
        <span class="setting-val">
          <button class="link-btn" @click="openSite">{{ SITE_URL }}</button>
        </span>
      </div>
      <div class="setting-row">
        <span class="setting-key">{{ t("positioning") }}</span>
        <span class="setting-val">{{ isWindows ? t("positioningWinVal") : t("positioningMacVal") }}</span>
      </div>
      <div class="setting-row">
        <span class="setting-key">{{ t("donateTitle") }}</span>
        <span class="setting-val">
          <button class="link-btn" @click="donateOpen = true">
            <span class="donate-btn-icon" aria-hidden="true" v-html="icons.wechat"></span>{{ t("donateAction") }}
          </button>
        </span>
      </div>
      <div class="setting-row">
        <span class="setting-key">{{ t("disclaimer") }}</span>
        <span class="setting-val">{{ t("disclaimerVal") }}</span>
      </div>
    </div>
  </div>

  <!-- 支持作者弹框 -->
  <div v-if="donateOpen" class="overlay" @click.self="donateOpen = false">
    <div class="dialog">
      <h3 class="dialog-title">{{ t("donateTitle") }}</h3>
      <p class="dialog-hint">{{ t("donateSub") }}</p>
      <div class="donate-qrs">
        <figure class="donate-item">
          <img class="donate-qr" :src="wechatQrImg" :alt="t('donateWechat')" draggable="false" />
          <figcaption>{{ t("donateWechat") }}</figcaption>
        </figure>
        <figure class="donate-item">
          <img class="donate-qr" :src="alipayQrImg" :alt="t('donateAlipay')" draggable="false" />
          <figcaption>{{ t("donateAlipay") }}</figcaption>
        </figure>
      </div>
      <div class="dialog-actions">
        <button class="btn" @click="donateOpen = false">{{ t("cancel") }}</button>
      </div>
    </div>
  </div>
</template>
