<script setup lang="ts">
import { computed } from 'vue';
import type { Object3D } from 'three';

interface Props {
  selectedObject: Object3D | null;
}

const props = defineProps<Props>();

const objectName = computed(() => props.selectedObject?.name || 'Unnamed');
const objectType = computed(() => props.selectedObject?.type || 'Unknown');
const objectUuid = computed(() => props.selectedObject?.uuid || 'N/A');

const position = computed(() => {
  if (!props.selectedObject) return null;
  const p = props.selectedObject.position;
  return { x: p.x.toFixed(3), y: p.y.toFixed(3), z: p.z.toFixed(3) };
});

const rotation = computed(() => {
  if (!props.selectedObject) return null;
  const r = props.selectedObject.rotation;
  return {
    x: r.x.toFixed(3),
    y: r.y.toFixed(3),
    z: r.z.toFixed(3),
    xDeg: (r.x * 180 / Math.PI).toFixed(1),
    yDeg: (r.y * 180 / Math.PI).toFixed(1),
    zDeg: (r.z * 180 / Math.PI).toFixed(1)
  };
});

const scale = computed(() => {
  if (!props.selectedObject) return null;
  const s = props.selectedObject.scale;
  return { x: s.x.toFixed(3), y: s.y.toFixed(3), z: s.z.toFixed(3) };
});

const userData = computed(() => props.selectedObject?.userData || {});
const hasGeometry = computed(() => props.selectedObject && 'geometry' in props.selectedObject);
const hasMaterial = computed(() => props.selectedObject && 'material' in props.selectedObject);
</script>

<template>
  <div class="p-4 text-gray-300 font-mono text-sm">
    <div v-if="!selectedObject" class="text-center py-10 px-5 text-gray-500">
      <p>Click an object to view details</p>
    </div>

    <div v-else>
      <!-- Basic Info -->
      <section class="mb-5 border-b border-gray-700 pb-3 last:border-b-0">
        <h3 class="text-sm text-blue-400 mb-3 font-bold">Basic Information</h3>
        <div class="flex justify-between mb-1.5">
          <span class="text-gray-400">Name:</span>
          <span class="text-white text-right">{{ objectName }}</span>
        </div>
        <div class="flex justify-between mb-1.5">
          <span class="text-gray-400">Type:</span>
          <span class="text-white text-right">{{ objectType }}</span>
        </div>
        <div class="flex justify-between mb-1.5">
          <span class="text-gray-400">UUID:</span>
          <span class="text-white text-right text-[10px] break-all">{{ objectUuid }}</span>
        </div>
      </section>

      <!-- Transform -->
      <section class="mb-5 border-b border-gray-700 pb-3 last:border-b-0">
        <h3 class="text-sm text-blue-400 mb-3 font-bold">Transform</h3>
        <div v-if="position" class="mb-3">
          <h4 class="text-xs text-gray-500 mt-2 mb-1">Position</h4>
          <div class="flex flex-col gap-1 pl-2">
            <span class="text-white">X: {{ position.x }}</span>
            <span class="text-white">Y: {{ position.y }}</span>
            <span class="text-white">Z: {{ position.z }}</span>
          </div>
        </div>
        <div v-if="rotation" class="mb-3">
          <h4 class="text-xs text-gray-500 mt-2 mb-1">Rotation</h4>
          <div class="flex flex-col gap-1 pl-2">
            <span class="text-white">X: {{ rotation.x }} ({{ rotation.xDeg }}°)</span>
            <span class="text-white">Y: {{ rotation.y }} ({{ rotation.yDeg }}°)</span>
            <span class="text-white">Z: {{ rotation.z }} ({{ rotation.zDeg }}°)</span>
          </div>
        </div>
        <div v-if="scale" class="mb-3">
          <h4 class="text-xs text-gray-500 mt-2 mb-1">Scale</h4>
          <div class="flex flex-col gap-1 pl-2">
            <span class="text-white">X: {{ scale.x }}</span>
            <span class="text-white">Y: {{ scale.y }}</span>
            <span class="text-white">Z: {{ scale.z }}</span>
          </div>
        </div>
      </section>

      <!-- Geometry Info -->
      <section v-if="hasGeometry" class="mb-5 border-b border-gray-700 pb-3 last:border-b-0">
        <h3 class="text-sm text-blue-400 mb-3 font-bold">Geometry</h3>
        <div class="flex justify-between mb-1.5">
          <span class="text-gray-400">Type:</span>
          <span class="text-white text-right">{{ (selectedObject as any).geometry?.type || 'N/A' }}</span>
        </div>
      </section>

      <!-- Material Info -->
      <section v-if="hasMaterial" class="mb-5 border-b border-gray-700 pb-3 last:border-b-0">
        <h3 class="text-sm text-blue-400 mb-3 font-bold">Material</h3>
        <div class="flex justify-between mb-1.5">
          <span class="text-gray-400">Type:</span>
          <span class="text-white text-right">{{ (selectedObject as any).material?.type || 'N/A' }}</span>
        </div>
      </section>

      <!-- User Data -->
      <section v-if="Object.keys(userData).length > 0" class="mb-5 border-b border-gray-700 pb-3 last:border-b-0">
        <h3 class="text-sm text-blue-400 mb-3 font-bold">Custom Data</h3>
        <pre class="bg-black/50 p-2 rounded overflow-x-auto text-[11px] text-green-500">{{ JSON.stringify(userData, null, 2) }}</pre>
      </section>
    </div>
  </div>
</template>
