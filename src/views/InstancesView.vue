<script setup lang="ts">
import { computed, ref } from "vue";
import { useAppStore } from "../stores/appStore";
import type { LaunchTarget } from "../types";

const store = useAppStore();

// 平台分支：Windows 走"本体多开"模型（无副本操作），macOS 走副本模型
const isWindows = navigator.userAgent.includes("Windows");

// ---- 弹框（创建 / 重命名 / 删除确认 / 重建确认，共用一个） ----
const showModal = ref(false);
const modalMode = ref<"create" | "rename" | "delete" | "rebuild">("create");
const renameTarget = ref<LaunchTarget | null>(null);
const confirmTarget = ref<LaunchTarget | null>(null);
const isMainRename = ref(false);
const nameInput = ref("");

const modalTitle = computed(() => {
  switch (modalMode.value) {
    case "create":
      return "创建副本";
    case "delete":
      return `删除「${confirmTarget.value?.label ?? ""}」`;
    case "rebuild":
      return `重建「${confirmTarget.value?.label ?? ""}」`;
    default:
      return isMainRename.value ? "设置主程序备注" : "重命名副本";
  }
});

const modalHint = computed(() => {
  switch (modalMode.value) {
    case "create":
      return "克隆主程序并重签名，生成独立数据目录的副本，创建后可在启动台 / Dock 直接看到此名字";
    case "delete":
      return "副本应用会移到废纸篓（可恢复），该账号的数据仍保留在独立容器中，不会丢失。";
    case "rebuild":
      return "将重新克隆当前版本的微信，名字与账号数据全部保留，无需重新登录。";
    default:
      return isMainRename.value
        ? "备注名仅在本工具内显示，不会修改微信本体"
        : "名字会同步显示在 Finder / 启动台 / Dock，账号数据不受影响";
  }
});

const isConfirmMode = computed(
  () => modalMode.value === "delete" || modalMode.value === "rebuild",
);

// 弹框提交中：保持弹框打开，按钮转圈
const submitting = ref(false);

const submittingLabels: Record<string, string> = {
  create: "创建中…",
  rename: "保存中…",
  delete: "删除中…",
  rebuild: "重建中…",
};

const normalLabels: Record<string, string> = {
  create: "创建",
  rename: "保存",
  delete: "删除",
  rebuild: "重建",
};

const buttonLabel = computed(
  () => (submitting.value ? submittingLabels : normalLabels)[modalMode.value] ?? "确定",
);

function openCreate() {
  // 默认名：跳过已占用的「副本 N」
  const used = new Set(store.targets.map((t: LaunchTarget) => t.label));
  let i = 2;
  while (used.has(`副本 ${i}`) || used.has(`副本${i}`)) i++;
  modalMode.value = "create";
  renameTarget.value = null;
  confirmTarget.value = null;
  isMainRename.value = false;
  nameInput.value = `副本 ${i}`;
  showModal.value = true;
}

function openRename(t: LaunchTarget) {
  modalMode.value = "rename";
  renameTarget.value = t;
  confirmTarget.value = null;
  isMainRename.value = t.isMain;
  nameInput.value = t.label;
  showModal.value = true;
}

function openDelete(t: LaunchTarget) {
  modalMode.value = "delete";
  confirmTarget.value = t;
  showModal.value = true;
}

function openRebuild(t: LaunchTarget) {
  modalMode.value = "rebuild";
  confirmTarget.value = t;
  showModal.value = true;
}

async function confirmModal() {
  if (submitting.value) return;
  submitting.value = true;
  try {
    if (isConfirmMode.value) {
      if (confirmTarget.value) {
        if (modalMode.value === "delete") {
          await store.deleteCopy(confirmTarget.value.path);
        } else {
          await store.rebuildCopy(confirmTarget.value.path);
        }
      }
    } else {
      const name = nameInput.value.trim();
      if (!name) return;
      if (modalMode.value === "create") {
        await store.generateCopy(name);
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

function closeTarget(t: LaunchTarget) {
  if (t.pid != null) void store.terminate(t.pid);
}

const outdatedCount = computed(
  () => store.targets.filter((t: LaunchTarget) => t.outdated).length,
);

const startTime = (sec: number | null) =>
  sec
    ? new Date(sec * 1000).toLocaleTimeString("zh-CN", {
        hour: "2-digit",
        minute: "2-digit",
      })
    : "";
</script>

<template>
  <!-- 环境检测 -->
  <section class="card">
    <h3 class="card-title">环境检测</h3>
    <div v-if="store.info?.installed" class="row">
      <span class="dot ok"></span>
      <div>
        <div>
          微信已安装
          <span class="ver">v{{ store.info.version }}</span>
        </div>
        <div class="path">{{ store.info.path }}</div>
      </div>
    </div>
    <div v-else class="row">
      <span class="dot err"></span>
      <div>
        <div>未检测到微信</div>
        <div class="path">
          {{ isWindows ? "请确认微信已安装（默认安装目录或注册表指定路径）" : "请确认微信已安装到 /Applications 目录" }}
        </div>
      </div>
    </div>
  </section>

  <!-- 实例列表 -->
  <section class="card">
    <div class="card-head">
      <h3 class="card-title">实例列表</h3>
      <button v-if="!isWindows" class="btn primary" :disabled="store.busy" @click="openCreate">
        + 创建副本
      </button>
    </div>

    <div v-if="store.busy" class="banner-inline info">
      <span class="spinner"></span>{{ store.busyText }}
    </div>
    <div v-if="outdatedCount" class="banner-inline warn">
      微信已升级，{{ outdatedCount }} 个副本版本落后，建议重建（账号数据保留）
    </div>
    <div v-if="!store.targets.length" class="empty">未检测到任何实例</div>
    <div v-for="t in store.targets" :key="t.isMain ? t.path : `${t.path}#${t.pid}`" class="list-row">
      <span class="dot" :class="t.running ? 'ok' : 'idle'"></span>
      <span class="tag" :class="t.isMain ? 'main' : 'copy'">
        {{ t.isMain ? "主程序" : isWindows ? "实例" : "副本" }}
      </span>
      <div class="target-info">
        <div class="target-name">
          {{ t.label }}
          <span v-if="t.outdated" class="ver warn-text">⚠ 版本落后</span>
          <span v-if="t.running" class="ver">
            <template v-if="isWindows && t.isMain">{{ t.instanceCount }} 个实例运行中</template>
            <template v-else>运行中 · PID {{ t.pid }} · {{ startTime(t.startedAt) }} 启动</template>
          </span>
          <span v-else-if="!t.outdated" class="ver">未运行</span>
        </div>
        <div v-if="!isWindows || t.isMain" class="path">{{ t.path }}</div>
      </div>
      <div class="row" style="gap: 6px">
        <!-- 启动：macOS 仅未运行时显示；Windows 主程序始终可点（每次点击多开一个） -->
        <template v-if="!t.running || (isWindows && t.isMain)">
          <button class="btn primary" :disabled="store.busy" @click="store.launch(t.path)">
            {{ isWindows && t.isMain ? "启动新实例" : "启动" }}
          </button>
        </template>
        <template v-else>
          <button class="btn" :disabled="store.busy" @click="store.activate(t.path, t.pid)">
            激活
          </button>
          <button class="btn danger" :disabled="store.busy" @click="closeTarget(t)">
            关闭
          </button>
        </template>
        <template v-if="!t.isMain && !isWindows">
          <button
            v-if="t.outdated"
            class="btn"
            :disabled="store.busy"
            @click="openRebuild(t)"
          >
            重建
          </button>
          <button class="btn ghost" :disabled="store.busy" @click="openRename(t)">改名</button>
          <button class="btn ghost danger-text" :disabled="store.busy" @click="openDelete(t)">
            删除
          </button>
        </template>
        <template v-else-if="t.isMain && !isWindows">
          <button class="btn ghost" :disabled="store.busy" @click="openRename(t)">备注</button>
        </template>
      </div>
    </div>
  </section>

  <!-- 创建 / 重命名 / 删除确认 / 重建确认 弹框 -->
  <div v-if="showModal" class="overlay" @click.self="showModal = false">
    <div class="dialog">
      <h3 class="dialog-title">{{ modalTitle }}</h3>
      <p class="dialog-hint">{{ modalHint }}</p>
      <input
        v-if="!isConfirmMode"
        v-model="nameInput"
        class="dialog-input"
        maxlength="36"
        :disabled="submitting"
        placeholder="输入名字，如：工作号"
        @keyup.enter="confirmModal"
      />
      <div class="dialog-actions">
        <button class="btn" :disabled="submitting" @click="showModal = false">取消</button>
        <button
          class="btn"
          :class="modalMode === 'delete' ? 'danger-solid' : 'primary'"
          :disabled="submitting || (!isConfirmMode && !nameInput.trim())"
          @click="confirmModal"
        >
          <span v-if="submitting" class="spinner"></span>
          {{ buttonLabel }}
        </button>
      </div>
    </div>
  </div>
</template>
