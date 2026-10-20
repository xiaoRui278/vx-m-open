<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useAppStore } from "./stores/appStore";
import InstancesView from "./views/InstancesView.vue";
import SettingsView from "./views/SettingsView.vue";

const store = useAppStore();
const active = ref<"instances" | "settings">("instances");

const navItems = [
  { key: "instances", label: "实例管理" },
  { key: "settings", label: "设置" },
] as const;

const views = {
  instances: InstancesView,
  settings: SettingsView,
};

onMounted(() => store.init());
</script>

<template>
  <div class="app">
    <aside class="sidebar">
      <div class="logo">vx-m-open</div>
      <button
        v-for="item in navItems"
        :key="item.key"
        class="nav-item"
        :class="{ active: active === item.key }"
        @click="active = item.key"
      >
        {{ item.label }}
      </button>
    </aside>
    <main class="content">
      <component :is="views[active]" />
    </main>
    <div
      v-if="store.message"
      class="banner"
      :class="store.messageType"
      @click="store.clearMessage()"
    >
      {{ store.message }}
    </div>
  </div>
</template>
