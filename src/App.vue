<script setup lang="ts">
    import { ref, onMounted, onUnmounted } from 'vue';
    import { build_robot_renderer } from './renderer';

    const canvas = ref<HTMLDivElement | null>(null);
    const wsConnected = ref(false);
    const wsConnection = ref<WebSocket | null>(null);
    const wsPort = ref(9876);
    const TimeSeries: Frame[] = [];
    
    const DataType = {
        F32: 0,
        F64: 1,
        VecF32: 2,
        VecF64: 3,
    } as const;
    type DataType = (typeof DataType)[keyof typeof DataType];
    
    function get_path(buffer: Uint8Array): string {
        return new TextDecoder('utf-8').decode(buffer);
    }
    
    function get_type(p0: number): DataType {
        return p0 as DataType;
    }

    class Frame {
        timestamp: bigint;
        path: string;
        type: DataType;
        data: Uint8Array;
        constructor(buffer: ArrayBuffer) {
            const u8_array = new Uint8Array(buffer);
            const u64_array = new BigUint64Array(buffer);
            this.timestamp = (u64_array[0] as bigint << 64n) | (u64_array[1] as bigint);
            const path_length = (u8_array[16] as number) << 8 | (u8_array[17] as number);
            this.path = get_path(u8_array.slice(18, 18 + path_length));
            this.type = get_type(u8_array[18 + path_length] as number);
            this.data = u8_array.slice(19 + path_length);
        }
    }

    function showData() {
        console.log(TimeSeries);
    }
    
    // WebSocket连接函数
    const connectWebSocket = () => {
        const wsUrl = `ws://localhost:${wsPort.value}/ws`; // 可以根据需要修改WebSocket地址

        try {
            const ws = new WebSocket(wsUrl);
            
            wsConnection.value = ws;

            ws.onopen = () => {
                console.log('WebSocket connected successfully');
                wsConnected.value = true;
            };

            ws.onmessage = (event) => {
                console.log('Received message:', event);
                // (event.data as Blob).arrayBuffer().then((buffer: ArrayBuffer) => {
                //     const frame = new Frame(buffer);
                //     TimeSeries.push(frame);
                // });
            };

            ws.onclose = () => {
                console.log('WebSocket connection closed');
                wsConnected.value = false;
                wsConnection.value = null;
            };

            ws.onerror = (error) => {
                console.error('WebSocket error:', error);
                wsConnected.value = false;
            };

        } catch (error) {
            wsConnected.value = false;
            wsConnection.value = null;
            console.error('Failed to create WebSocket connection:', error);
        }
    };

    onMounted(() => {
        if (canvas.value !== null) {
            build_robot_renderer(canvas.value);
        } else {
            console.error('canvas not found');
        }
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
    <div style="position: absolute; background-color: rgba(0, 0, 0, 0.5); z-index: 1000;">
        <label for="wsPort">WebSocket Port:</label>
        <input type="number" v-model="wsPort" id="wsPort" />
        <button @click="connectWebSocket" v-if="!wsConnected">Connect</button>
        <button @click="showData">show data</button>
    </div>
    <div ref="canvas"></div>

</template>

<style scoped>
</style>
