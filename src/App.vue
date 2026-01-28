<script setup lang="ts">
    import { ref, onMounted, onUnmounted } from 'vue';
    import { build_robot_renderer, robot_renderer } from './renderer';
    import ObjectInfoPanel from './components/ObjectInfoPanel.vue';
    import type { Object3D } from 'three';

    const canvas = ref<HTMLDivElement | null>(null);
    const canvasContainer = ref<HTMLDivElement | null>(null);
    const wsConnected = ref(false);
    const wsConnection = ref<WebSocket | null>(null);
    const wsPort = ref(9876);
    const selectedObject = ref<Object3D | null>(null);
    const isPanelVisible = ref(true);
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
        console.log(robot_renderer);
    }

    function togglePanel() {
        isPanelVisible.value = !isPanelVisible.value;
    }

    function handleObjectSelection(object: Object3D | null) {
        selectedObject.value = object;
    }
    
    // WebSocket连接函数
    const connectWebSocket = () => {
        const wsUrl = `ws://localhost:${wsPort.value}/ws`; // 可以根据需要修改WebSocket地址

        try {
            const ws = new WebSocket(wsUrl);
            
            wsConnection.value = ws;

            ws.onopen = (ev: Event) => {
                console.log('WebSocket connected successfully');
                console.log(new Date().toLocaleString());
                // start a new clip
                console.log(ev);
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

            if (robot_renderer && canvasContainer.value) {
                // Setup selection callback
                robot_renderer.setObjectSelectionCallback(handleObjectSelection);

                // Setup resize observer
                robot_renderer.setupResizeObserver(canvasContainer.value);
            }
        } else {
            console.error('canvas not found');
        }
    });

    onUnmounted(() => {
        if (robot_renderer) {
            robot_renderer.cleanup();
        }
        // 清理WebSocket连接
        if (wsConnection.value) {
            wsConnection.value.close();
            wsConnection.value = null;
        }
    });
</script>

<template>
    <div class="flex h-screen w-screen overflow-hidden">
        <div class="absolute top-2.5 left-2.5 z-[1000] bg-black/50 p-2.5 rounded">
            <label for="wsPort">WebSocket Port:</label>
            <input type="number" v-model="wsPort" id="wsPort" class="ml-2 px-2 py-1 bg-gray-800 text-white border border-gray-600 rounded" />
            <button @click="connectWebSocket" v-if="!wsConnected" class="ml-2 px-3 py-1 bg-blue-600 hover:bg-blue-700 text-white rounded">Connect</button>
            <button @click="showData" class="ml-2 px-3 py-1 bg-gray-700 hover:bg-gray-600 text-white rounded">show data</button>
        </div>
        <div class="flex-1 relative overflow-hidden" ref="canvasContainer">
            <div ref="canvas"></div>
        </div>
        <div v-if="isPanelVisible" class="w-80 bg-black/85 overflow-y-auto border-l border-gray-700">
            <ObjectInfoPanel :selectedObject="selectedObject" />
        </div>
        <button
            class="absolute top-1/2 -translate-y-1/2 z-[1001] bg-black/70 text-white border border-gray-700 px-3 py-2 cursor-pointer rounded-l hover:bg-black/90 transition-all duration-300"
            @click="togglePanel"
            :style="{ right: isPanelVisible ? '320px' : '0' }"
        >
            {{ isPanelVisible ? '>' : '<' }}
        </button>
    </div>
</template>
