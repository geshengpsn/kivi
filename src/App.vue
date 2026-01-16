<script setup lang="ts">
    import { ref, onMounted, onUnmounted } from 'vue';
    import { build_robot_renderer } from './renderer';

    const canvas = ref<HTMLDivElement | null>(null);
    const wsConnected = ref(false);
    const wsConnection = ref<WebSocket | null>(null);
    const reconnectInterval = 1000; // 1秒重连间隔
    const wsPort = ref(9876);

    // WebSocket连接函数
    const connectWebSocket = () => {
        const wsUrl = `ws://localhost:${wsPort.value}/ws`; // 可以根据需要修改WebSocket地址

        try {
            const ws = new WebSocket(wsUrl);
            
            wsConnection.value = ws;

            ws.onopen = () => {
                // console.log('WebSocket connected successfully');
                wsConnected.value = true;
            };

            ws.onmessage = (event) => {
                console.log('Received message:', event.data);
                // 处理接收到的消息
            };

            ws.onclose = () => {
                // console.log('WebSocket connection closed');
                wsConnected.value = false;
                wsConnection.value = null;

                // 如果连接断开，尝试重连
                setTimeout(() => {
                    connectWebSocket();
                }, reconnectInterval);
            };

            ws.onerror = (_error) => {
                // console.error('WebSocket error:', error);
                wsConnected.value = false;
            };

        } catch (error) {
            // console.error('Failed to create WebSocket connection:', error);
            // 如果创建连接失败，也尝试重连
            setTimeout(() => {
                connectWebSocket();
            }, reconnectInterval);
        }
    };

    onMounted(() => {
        if (canvas.value !== null) {
            build_robot_renderer(canvas.value);
        } else {
            console.error('canvas not found');
        }

        // 页面渲染完成后开始连接WebSocket
        connectWebSocket();
    });

    onUnmounted(() => {
        // 清理WebSocket连接
        if (wsConnection.value) {
            wsConnection.value.close();
            wsConnection.value = null;
        }
    });
</script>

<template>
    <div ref="canvas"></div>
</template>

<style scoped>
</style>
