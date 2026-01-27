<script setup lang="ts">
    import { ref, onMounted, onUnmounted } from 'vue';
    import { build_robot_renderer } from './renderer';

    const canvas = ref<HTMLDivElement | null>(null);
    const wsConnected = ref(false);
    const wsConnection = ref<WebSocket | null>(null);
    const wsPort = ref(9876);
    const Clip: Frame[] = [];
    
    const DataType = {
        ScalarF64: 0,
        Box3: 1,
        BoxLine3: 2,
        Sphere: 3,
        Cylinder: 4,
        Capsule: 5,
        Stl: 6,
        MeshMaterial: 7,
        Matrix4: 8,
        Arrow3: 9,
    } as const;

    type DataTypeIndex = (typeof DataType)[keyof typeof DataType];

    function parse_vec_f64(data: Uint8Array, length: number): Float64Array {
        return new Float64Array(data.buffer, 0, length);
    }

    function get_path(buffer: Uint8Array): string {
        return new TextDecoder('utf-8').decode(buffer);
    }
    
    function get_type(p0: number): DataTypeIndex {
        return p0 as DataTypeIndex;
    }

    class Frame {
        timestamp: bigint;
        path: string;
        type_index: DataTypeIndex;
        data: any;
        constructor(buffer: ArrayBuffer) {
            const data_view = new DataView(buffer);
            const u8_array = new Uint8Array(buffer);
            this.timestamp = data_view.getBigUint64(8, true) << 64n | data_view.getBigUint64(0, true);
            const path_length = data_view.getUint16(16, true);
            this.path = get_path(u8_array.slice(18, 18 + path_length));
            this.type_index = get_type(data_view.getUint16(18 + path_length, true));
            this.data = this.parse_data(this.type_index, u8_array.slice(20 + path_length));
        }

        parse_data(type: DataTypeIndex, data: Uint8Array): any {
            switch (type) {
                case DataType.ScalarF64:
                    return parse_vec_f64(data, 1);
                case DataType.Box3:
                    return parse_vec_f64(data, 3);
                case DataType.BoxLine3:
                    return parse_vec_f64(data, 3);
                case DataType.Sphere:
                    return parse_vec_f64(data, 1);
                case DataType.Cylinder:
                    return parse_vec_f64(data, 2);
                case DataType.Capsule:
                    return parse_vec_f64(data, 2);
                case DataType.Stl:
                    return data;
                case DataType.MeshMaterial:
                    return {
                        color: data.slice(0, 3),
                        roughness: new DataView(data.buffer).getFloat64(3, true),
                        metalness: new DataView(data.buffer).getFloat64(11, true),
                    };
                case DataType.Matrix4:
                    return parse_vec_f64(data, 16);
                case DataType.Arrow3:
                    return {
                        start_end: parse_vec_f64(data, 6),
                        color: data.slice(12, 15),
                    };
                default:
                    throw new Error(`Unknown data type: ${type}`);
            }
        }
    }

    function showData() {
        // console.log(clip);
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
                // console.log('Received message:', event);
                (event.data as Blob).arrayBuffer().then((buffer: ArrayBuffer) => {
                    const frame = new Frame(buffer);
                    console.log(frame);
                    // TimeSeries.push(frame);
                });
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
