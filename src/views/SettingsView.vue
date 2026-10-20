<script setup lang="ts">
import { useAppStore } from "../stores/appStore";

const store = useAppStore();
</script>

<template>
  <section class="card">
    <h3 class="card-title">常规</h3>
    <div class="setting-row">
      <span>微信路径</span>
      <span class="path">{{ store.info?.path ?? "未检测到" }}</span>
    </div>
    <div class="setting-row">
      <span>副本位置</span>
      <span class="path">/Applications（无写入权限时自动回退 ~/Applications）</span>
    </div>
    <div class="setting-row">
      <span>主题</span>
      <span>跟随系统明暗色</span>
    </div>
  </section>

  <section class="card">
    <h3 class="card-title">关于</h3>
    <div class="setting-row">
      <span>版本</span>
      <span>v0.1.0（M1）</span>
    </div>
    <div class="setting-row">
      <span>定位</span>
      <span>开源工具，不修改微信本体、不注入、不碰网络协议</span>
    </div>
    <div class="setting-row">
      <span>声明</span>
      <span>非官方第三方工具，与腾讯无关，请遵守微信用户协议</span>
    </div>
  </section>
</template>
