<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useAppStore } from "../stores/appStore";
import * as api from "../api";
import { icons } from "../icons";
import { t } from "../i18n";
import type { LaunchTarget } from "../types";
import logo from "../assets/logo.png";

import { isWindows as sharedIsWindows } from "../platform";

const store = useAppStore();

// 平台分支：Windows 走"本体多开"模型（无副本操作），macOS 走副本模型
const isWindows = sharedIsWindows;

// ================= 弹框（创建 / 改名 / 删除 / 重建确认） =================
const showModal = ref(false);
const modalMode = ref<"create" | "rename" | "instance-remark" | "delete" | "rebuild" | "rebuildall">("create");
const renameTarget = ref<LaunchTarget | null>(null);
const confirmTarget = ref<LaunchTarget | null>(null);
const isMainRename = ref(false);
const nameInput = ref("");
const submitting = ref(false);

const isConfirmMode = computed(
  () => modalMode.value === "delete" || modalMode.value === "rebuild" || modalMode.value === "rebuildall",
);

const modalTitle = computed(() => {
  switch (modalMode.value) {
    case "create":
      return t("createTitle");
    case "rebuildall":
      return t("rebuildAllTitle");
    case "delete":
      return t("deleteTitle", { name: confirmTarget.value?.label ?? "" });
    case "rebuild":
      return t("rebuildTitle", { name: confirmTarget.value?.label ?? "" });
    case "instance-remark":
      return t("instanceRemarkTitle");
    default:
      return isMainRename.value ? t("aliasTitle") : t("renameTitle", { name: renameTarget.value?.label ?? "" });
  }
});

const modalHint = computed(() => {
  switch (modalMode.value) {
    case "create":
      return t("createHint");
    case "rebuildall":
      return t("rebuildHint");
    case "delete":
      return t("deleteHint");
    case "rebuild":
      return t("rebuildHint");
    case "instance-remark":
      return t("instanceRemarkHint");
    default:
      return isMainRename.value ? t("aliasHint") : t("renameHint");
  }
});

const busyLabel = computed(() => {
  switch (modalMode.value) {
    case "create":
      return t("creating");
    case "delete":
      return t("deleting");
    case "rebuild":
    case "rebuildall":
      return t("rebuilding");
    default:
      return t("saving");
  }
});

const confirmLabel = computed(() => {
  switch (modalMode.value) {
    case "create":
      return t("createCopy");
    case "rebuildall":
      return t("rebuildAll");
    case "delete":
      return t("remove");
    case "rebuild":
      return t("rebuild");
    case "rename":
      return t("save");
    case "instance-remark":
      return t("saveRemark");
    default:
      return t("saveRemark");
  }
});

function openCreate() {
  // 默认名：跳过已占用的「副本 N」
  const used = new Set(store.targets.map((x: LaunchTarget) => x.label));
  let i = 2;
  while (used.has(t("defaultCopyName", { n: i })) || used.has(`副本 ${i}`) || used.has(`副本${i}`)) i++;
  modalMode.value = "create";
  renameTarget.value = null;
  confirmTarget.value = null;
  isMainRename.value = false;
  nameInput.value = t("defaultCopyName", { n: i });
  showModal.value = true;
}

function openRename(x: LaunchTarget) {
  modalMode.value = "rename";
  renameTarget.value = x;
  confirmTarget.value = null;
  isMainRename.value = x.isMain;
  nameInput.value = x.label;
  showModal.value = true;
}

/** 打开 Windows 实例备注弹框（跟随进程，pid + 启动时间匹配） */
function openInstanceRemark(x: LaunchTarget) {
  modalMode.value = "instance-remark";
  renameTarget.value = x;
  confirmTarget.value = null;
  isMainRename.value = false;
  nameInput.value = instanceDisplayName(x) ?? "";
  showModal.value = true;
}

function openConfirm(mode: "delete" | "rebuild", x: LaunchTarget) {
  modalMode.value = mode;
  renameTarget.value = null;
  confirmTarget.value = x;
  isMainRename.value = false;
  showModal.value = true;
}

function openRebuildAll() {
  modalMode.value = "rebuildall";
  renameTarget.value = null;
  confirmTarget.value = null;
  isMainRename.value = false;
  showModal.value = true;
}

async function confirmModal() {
  if (submitting.value) return;
  if (!isConfirmMode.value) {
    const name = nameInput.value.trim();
    if (!name) return;
  }
  submitting.value = true;
  try {
    if (modalMode.value === "rebuildall") {
      showModal.value = false;
      await runRebuildAll();
      return;
    }
    if (isConfirmMode.value && confirmTarget.value) {
      if (modalMode.value === "delete") {
        await store.deleteCopy(confirmTarget.value.path);
      } else {
        await store.rebuildCopy(confirmTarget.value.path);
      }
    } else {
      const name = nameInput.value.trim();
      if (!name) return;
      if (modalMode.value === "create") {
        showModal.value = false;
        await runCreateProgress(name);
        return;
      } else if (modalMode.value === "instance-remark" && renameTarget.value?.pid != null && renameTarget.value.startedAt != null) {
        await store.setInstanceRemark(
          renameTarget.value.pid,
          renameTarget.value.startedAt,
          name,
        );
      } else if (isMainRename.value) {
        await store.setMainAlias(name);
      } else if (renameTarget.value) {
        await store.renameCopy(renameTarget.value.path, name);
      }
    }
    showModal.value = false;
  } finally {
    submitting.value = false;
  }
}

// ================= 创建进度弹框（后端分步事件驱动） =================
const progressOpen = ref(false);
const progressName = ref("");
const progressStep = ref(0); // 0=克隆中 1=克隆完成 2=BundleID 完成 3=签名完成 4=完成
const progressFailed = ref(false);

const stepKeys = computed(() => [
  t("stepClone"),
  t("stepBundle"),
  t("stepSign"),
  t("stepDone"),
]);

const progressRatio = computed(() => Math.min(progressStep.value, 4) / 4);

async function runCreateProgress(name: string) {
  progressName.value = name;
  progressStep.value = 0;
  progressFailed.value = false;
  progressOpen.value = true;
  try {
    await api.generateCopy(name);
    progressStep.value = 4;
    await store.refreshAll();
    store.showMessage(t("copyCreated", { name }), "info");
    await new Promise((r) => setTimeout(r, 450));
  } catch (e) {
    progressFailed.value = true;
    store.showMessage(String(e), "error");
    await new Promise((r) => setTimeout(r, 900));
  } finally {
    progressOpen.value = false;
  }
}

// ================= 全部重建进度弹框 =================
const rebuildAllOpen = ref(false);
const rebuildAllDone = ref(0);
const rebuildAllTotal = ref(0);
const rebuildAllLabel = ref("");

const rebuildAllRatio = computed(() =>
  rebuildAllTotal.value ? rebuildAllDone.value / rebuildAllTotal.value : 0,
);

async function runRebuildAll() {
  rebuildAllOpen.value = true;
  rebuildAllDone.value = 0;
  try {
    // total 从第一个事件获取（命令内部过滤落后副本）
    await api.rebuildAll();
    await store.refreshAll();
  } catch (e) {
    store.showMessage(String(e), "error");
  } finally {
    rebuildAllOpen.value = false;
  }
}

// ================= 事件监听 =================
let unlisteners: UnlistenFn[] = [];

/** Esc 关闭通用弹框（进度弹框不响应，防误触） */
function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape" && showModal.value) showModal.value = false;
}

onMounted(() => {
  // 浏览器预览（非 Tauri）没有事件系统，静默跳过
  try {
    void listen<{ step: number; name: string }>("copy-progress", (e) => {
      if (progressOpen.value) progressStep.value = e.payload.step;
    }).then((u) => unlisteners.push(u));

    void listen<{ done: number; total: number; label: string }>(
      "rebuild-all-progress",
      (e) => {
        rebuildAllTotal.value = e.payload.total;
        rebuildAllDone.value = e.payload.done;
        rebuildAllLabel.value = e.payload.label;
      },
    ).then((u) => unlisteners.push(u));
  } catch {
    /* 预览环境 */
  }

  window.addEventListener("keydown", onKeydown);
});

onUnmounted(() => {
  unlisteners.forEach((u) => u());
  unlisteners = [];
  window.removeEventListener("keydown", onKeydown);
});

// ================= 列表渲染辅助 =================
/** 主程序默认 label 为后端标记值 "WeChat"，按界面语言映射；备注名原样显示 */
function displayName(x: LaunchTarget): string {
  return x.isMain && x.label === "WeChat" ? t("wechat") : x.label;
}

/** Windows 实例备注：按 pid + 启动时间匹配，跟随进程 */
function instanceDisplayName(x: LaunchTarget): string | null {
  return (
    store.instanceRemarks.find(
      (r) => r.pid === x.pid && r.startedAt === x.startedAt,
    )?.remark ?? null
  );
}

function closeTarget(x: LaunchTarget) {
  if (x.pid != null) void store.terminate(x.pid);
}

const outdatedCount = computed(
  () => store.targets.filter((x: LaunchTarget) => x.outdated).length,
);

const mainTarget = computed(() => store.targets.find((x: LaunchTarget) => x.isMain));
const winInstances = computed(() => store.targets.filter((x: LaunchTarget) => !x.isMain));

const startTime = (sec: number | null) =>
  sec
    ? new Date(sec * 1000).toLocaleTimeString(localeTimeTag(), {
        hour: "2-digit",
        minute: "2-digit",
      })
    : "";

function localeTimeTag() {
  return localStorage.getItem("vxm.locale") === "en" ? "en-US" : "zh-CN";
}

const anyDialogOpen = computed(
  () => showModal.value || progressOpen.value || rebuildAllOpen.value,
);
</script>

<template>
  <!-- 页头 -->
  <header class="page-head">
    <div class="head-fill">
      <h1 class="page-title">{{ t("instancesTitle") }}</h1>
      <p class="page-sub">{{ isWindows ? t("instancesSubWin") : t("instancesSubMac") }}</p>
    </div>
    <button v-if="!isWindows" class="btn primary lg" :disabled="store.busy || anyDialogOpen" @click="openCreate">
      <span aria-hidden="true" v-html="icons.plus"></span>
      {{ t("createCopy") }}
    </button>
  </header>

  <div class="content">
    <!-- 环境检测条 -->
    <div class="env-strip">
      <span class="env-icon" aria-hidden="true">
        <span v-if="store.info?.installed" v-html="icons.check"></span>
        <span v-else v-html="icons.alert"></span>
      </span>
      <div class="env-body">
        <div class="env-title">
          <template v-if="store.info?.installed">
            {{ t("wechatInstalled") }}<span v-if="store.info.version" class="ver-chip">v{{ store.info.version }}</span>
          </template>
          <template v-else>{{ t("wechatMissing") }}</template>
        </div>
        <div class="env-path">
          <span v-if="store.info?.path" class="path-text">{{ store.info.path }}</span>
          <span v-else class="path-text">{{ isWindows ? t("wechatMissingWinHint") : t("wechatMissingMacHint") }}</span>
        </div>
      </div>
    </div>

    <!-- 版本落后警示（macOS 副本模型） -->
    <div v-if="!isWindows && outdatedCount" class="warn-strip">
      <span aria-hidden="true" v-html="icons.warn"></span>
      <span class="head-fill">{{ t("outdatedWarning", { n: outdatedCount }) }}</span>
      <button class="btn" :disabled="store.busy || anyDialogOpen" @click="openRebuildAll">
        {{ t("rebuildAll") }}
      </button>
    </div>

    <div v-if="store.busy && store.busyText" class="busy-line">
      <span class="spinner"></span>{{ store.busyText }}
    </div>

    <!-- ========== macOS：副本卡片列表 ========== -->
    <template v-if="!isWindows">
      <div v-if="store.targets.length">
        <h2 class="section-label">{{ t("copiesLabel") }}（{{ store.targets.length }}）</h2>
        <article
          v-for="x in store.targets"
          :key="x.path"
          class="instance"
          :data-state="x.running ? 'running' : x.outdated ? 'outdated' : 'stopped'"
        >
          <div class="avatar" :class="{ main: x.isMain }" aria-hidden="true">{{ displayName(x).slice(0, 1) }}</div>
          <div class="ident">
            <div class="ident-name-line">
              <span class="state-dot" aria-hidden="true"></span>
              <span class="ident-name">{{ displayName(x) }}</span>
              <span class="tag" :class="x.isMain ? 'main' : 'copy'">{{ x.isMain ? t("main") : t("copy") }}</span>
            </div>
            <div class="ident-path"><span class="path-text">{{ x.path }}</span></div>
            <div v-if="x.isMain" class="meta-line">{{ t("macModel") }}</div>
            <div v-if="x.running || (!x.running && x.outdated)" class="meta-line">
              <template v-if="x.running">
                <span v-if="x.pid">PID {{ x.pid }}</span>
                <span v-if="x.pid && x.startedAt" class="sep">·</span>
                <span v-if="x.startedAt">{{ t("startedAt", { time: startTime(x.startedAt) }) }}</span>
              </template>
              <template v-else>{{ t("outdatedHint") }}</template>
            </div>
          </div>
          <div class="actions">
            <template v-if="!x.running">
              <button class="btn primary sm" :disabled="store.busy || anyDialogOpen" @click="store.launch(x.path)">
                <span aria-hidden="true" v-html="icons.play"></span>{{ t("launch") }}
              </button>
            </template>
            <template v-else>
              <button class="btn sm" :disabled="store.busy || anyDialogOpen" @click="store.activate(x.path, x.pid)">
                <span aria-hidden="true" v-html="icons.target"></span>{{ t("activate") }}
              </button>
              <button class="btn danger sm" :disabled="store.busy || anyDialogOpen" @click="closeTarget(x)">
                <span aria-hidden="true" v-html="icons.power"></span>{{ t("close") }}
              </button>
            </template>
            <template v-if="!x.isMain">
              <button v-if="x.outdated" class="btn sm" :disabled="store.busy || anyDialogOpen" @click="openConfirm('rebuild', x)">
                <span aria-hidden="true" v-html="icons.refresh"></span>{{ t("rebuild") }}
              </button>
              <button class="btn sm" :disabled="store.busy || anyDialogOpen" @click="openRename(x)">
                <span aria-hidden="true" v-html="icons.pencil"></span>{{ t("rename") }}
              </button>
              <button class="btn danger sm" :disabled="store.busy || anyDialogOpen" @click="openConfirm('delete', x)">
                <span aria-hidden="true" v-html="icons.trash"></span>{{ t("remove") }}
              </button>
            </template>
            <template v-else>
              <button class="btn sm" :disabled="store.busy || anyDialogOpen" @click="openRename(x)">
                <span aria-hidden="true" v-html="icons.pencil"></span>{{ t("remark") }}
              </button>
            </template>
          </div>
        </article>
      </div>
      <div v-else class="empty">
        <img class="empty-logo" :src="logo" alt="">
        <div class="empty-title">{{ t("emptyTitle") }}</div>
        <p class="empty-sub">{{ t("emptySub") }}</p>
        <button class="btn primary" :disabled="store.busy || anyDialogOpen" @click="openCreate">
          {{ t("createCopy") }}
        </button>
      </div>
    </template>

    <!-- ========== Windows：主程序大卡 + 运行实例行 ========== -->
    <template v-else>
      <h2 class="section-label">{{ t("main") }}</h2>
      <div v-if="mainTarget" class="instance" :data-state="mainTarget.running ? 'running' : 'stopped'">
        <div class="avatar main" aria-hidden="true" v-html="icons.wechat"></div>
        <div class="ident">
          <div class="ident-name-line">
            <span class="state-dot" aria-hidden="true"></span>
            <span class="ident-name">{{ displayName(mainTarget) }}</span>
            <span class="tag main">{{ t("main") }}</span>
          </div>
          <div class="ident-path"><span class="path-text">{{ mainTarget.path }}</span></div>
          <div class="meta-line">
            {{ t("winModel") }}
            <template v-if="mainTarget.running"> · {{ t("instancesRunning", { n: mainTarget.instanceCount }) }}</template>
          </div>
        </div>
        <div class="actions">
          <button class="btn primary lg" :disabled="store.busy || anyDialogOpen" @click="store.launch(mainTarget.path)">
            <span v-if="store.busy" class="spinner"></span>
            <span v-else aria-hidden="true" v-html="icons.play"></span>
            {{ store.busy ? t("launching") : t("launchNew") }}
          </button>
        </div>
      </div>

      <template v-if="winInstances.length">
        <h2 class="section-label" style="margin-top: 18px">
          {{ t("runningLabel") }}（{{ winInstances.length }}）
        </h2>
        <article v-for="(x, i) in winInstances" :key="x.pid ?? i" class="instance-row">
          <div class="instance-badge" aria-hidden="true">{{ i + 1 }}</div>
          <div class="row-fill">
            <div class="instance-name">{{ instanceDisplayName(x) ?? `${t("instance")} ${i + 1}` }}</div>
            <div class="instance-meta">
              <span v-if="x.pid" class="meta-pid">PID {{ x.pid }}</span>
              <span v-if="x.pid && x.startedAt" class="dot-sep">·</span>
              <span v-if="x.startedAt">{{ t("startedAt", { time: startTime(x.startedAt) }) }}</span>
            </div>
          </div>
          <div class="actions">
            <button class="btn sm" :disabled="store.busy || anyDialogOpen" @click="openInstanceRemark(x)">
              <span aria-hidden="true" v-html="icons.pencil"></span>{{ t("remark") }}
            </button>
            <button class="btn sm" :disabled="store.busy || anyDialogOpen" @click="store.activate(x.path, x.pid)">
              <span aria-hidden="true" v-html="icons.target"></span>{{ t("activate") }}
            </button>
            <button class="btn danger sm" :disabled="store.busy || anyDialogOpen" @click="closeTarget(x)">
              <span aria-hidden="true" v-html="icons.power"></span>{{ t("close") }}
            </button>
          </div>
        </article>
      </template>
      <div v-else class="empty">
        <img class="empty-logo" :src="logo" alt="">
        <div class="empty-title">{{ t("winEmptyTitle") }}</div>
        <p class="empty-sub">{{ t("winEmptySub") }}</p>
        <button class="btn primary" :disabled="store.busy || anyDialogOpen" @click="store.launch(mainTarget?.path ?? '')">
          {{ t("launchNew") }}
        </button>
      </div>
    </template>
  </div>

  <!-- 通用弹框（改名 / 删除确认 / 重建确认 / 全部重建确认）；点空白不关闭，防误触，Esc 可关 -->
  <div v-if="showModal" class="overlay">
    <div class="dialog" :class="{ 'danger-dialog': modalMode === 'delete' }">
      <div v-if="modalMode === 'delete' || modalMode === 'rebuild' || modalMode === 'rebuildall'" class="dialog-badge" aria-hidden="true" v-html="icons.alert"></div>
      <h3 class="dialog-title">{{ modalTitle }}</h3>
      <p class="dialog-hint">{{ modalHint }}</p>
      <input
        v-if="!isConfirmMode"
        v-model="nameInput"
        class="dialog-input"
        maxlength="36"
        :disabled="submitting"
        :placeholder="isMainRename || modalMode === 'instance-remark' ? t('remarkPlaceholder') : t('createPlaceholder')"
        @keyup.enter="confirmModal"
      />
      <div class="dialog-actions">
        <button class="btn" :disabled="submitting" @click="showModal = false">{{ t("cancel") }}</button>
        <button
          class="btn"
          :class="modalMode === 'delete' ? 'danger-solid' : 'primary'"
          :disabled="submitting || (!isConfirmMode && !nameInput.trim())"
          @click="confirmModal"
        >
          <span v-if="submitting" class="spinner"></span>
          {{ submitting ? busyLabel : confirmLabel }}
        </button>
      </div>
    </div>
  </div>

  <!-- 创建进度弹框（不可关闭） -->
  <div v-if="progressOpen" class="overlay">
    <div class="dialog progress-dialog" role="dialog" aria-modal="true">
      <h3 class="dialog-title">{{ t("progressTitle", { name: progressName }) }}</h3>
      <p class="dialog-hint">{{ t("progressHint") }}</p>
      <div class="progress-track">
        <div class="progress-bar" :style="{ transform: `scaleX(${progressFailed ? 1 : Math.max(progressRatio, 0.06)})` }"></div>
      </div>
      <ul class="steps">
        <li
          v-for="(label, i) in stepKeys"
          :key="label"
          :class="{ done: !progressFailed && progressStep > i, active: !progressFailed && progressStep === i }"
        >
          {{ label }}
        </li>
      </ul>
    </div>
  </div>

  <!-- 全部重建进度弹框（不可关闭） -->
  <div v-if="rebuildAllOpen" class="overlay">
    <div class="dialog progress-dialog" role="dialog" aria-modal="true">
      <h3 class="dialog-title">{{ t("rebuildAllTitle") }}</h3>
      <p class="dialog-hint">
        {{ rebuildAllTotal ? t("rebuildAllProgress", { label: rebuildAllLabel, done: rebuildAllDone, total: rebuildAllTotal }) : t("rebuildAllFinishing") }}
      </p>
      <div class="progress-track">
        <div class="progress-bar" :style="{ transform: `scaleX(${Math.max(rebuildAllRatio, 0.04)})` }"></div>
      </div>
    </div>
  </div>
</template>
