<script setup lang="ts">
    import { ref, onMounted, onUnmounted } from 'vue';
    import { build_robot_renderer, robot_renderer } from './renderer';
    // import ObjectInfoPanel from './components/ObjectInfoPanel.vue';
    // import type { Object3D } from 'three';
    import { Frame } from './frame';

    const canvas = ref<HTMLDivElement | null>(null);
    const canvasContainer = ref<HTMLDivElement | null>(null);
    // const wsConnected = ref(false);
    // const wsConnection = ref<WebSocket | null>(null);
    // const wsPort = ref(9876);

    let wsPort = 9876;

    function showData() {
        console.log(robot_renderer);
    }
    
    // WebSocket连接函数
    const connectWebSocket = () => {
        const wsUrl = `ws://localhost:${wsPort}/ws`; // 可以根据需要修改WebSocket地址

        try {
            const ws = new WebSocket(wsUrl);

            ws.onopen = (ev: Event) => {
                console.log('WebSocket connected successfully');
                console.log(new Date().toLocaleString());
                // start a new clip
                console.log(ev);
            };

            ws.onmessage = (event) => {
                (event.data as Blob).arrayBuffer().then((buffer: ArrayBuffer) => {
                    const frame = new Frame(buffer);
                    console.log(frame);
                });
            };

            ws.onclose = () => {
                console.log('WebSocket connection closed');
            };

            ws.onerror = (error) => {
                console.error('WebSocket error:', error);
            };

        } catch (error) {
            console.error('Failed to create WebSocket connection:', error);
        }
    };

    onMounted(() => {
        if (canvas.value !== null) {
            build_robot_renderer(canvas.value);

            if (robot_renderer && canvasContainer.value) {
                robot_renderer.setupResizeObserver(canvasContainer.value);
            }

            // get url params
            const urlParams = new URLSearchParams(window.location.search);
            console.log(urlParams.get('addr'));
        } else {
            console.error('canvas not found');
        }
    });

    onUnmounted(() => {
        if (robot_renderer) {
            robot_renderer.cleanup();
        }
    });
</script>

<template>
    <div ref="canvasContainer">
        <div ref="canvas"></div>
    </div>
</template>
