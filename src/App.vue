<script setup lang="ts">
    import { ref, onMounted, onUnmounted } from 'vue';
    import { build_robot_renderer, robot_renderer } from './renderer';
    import { apply_frame, Frame } from './frame';

    const canvas = ref<HTMLDivElement | null>(null);
    // const currentFrameIndex = ref(0);
    // const is_playing = ref(false);
    // let playbackAnimationId: number | null = null;
    // let clock = new Clock();
    // let lastFrameTime = 0;
    // let scene_frame_index = 0;
    // const playback_timestamp = ref(0);
    // const frameInterval = 1000 / 30; // 30 FPS
    // const hasFrames = computed(() => clip_length.value > 0);

    // Playback control functions
    // const togglePlayback = () => {
    //     if (is_playing.value) {
    //         pausePlayback();
    //     } else {
    //         startPlayback();
    //     }
    // };

    // const startPlayback = () => {
    //     if (!hasFrames.value) return;
    //     is_playing.value = true;
    //     lastFrameTime = performance.now();

    //     const animate = (currentTime: number) => {
    //         if (!is_playing.value) return;

    //         const elapsed = currentTime - lastFrameTime;

    //         if (elapsed >= frameInterval) {
    //             if (currentFrameIndex.value < totalFrames.value - 1) {
    //                 currentFrameIndex.value++;
    //                 applyFrame(currentFrameIndex.value);
    //                 lastFrameTime = currentTime - (elapsed % frameInterval);
    //             } else {
    //                 pausePlayback();
    //                 return;
    //             }
    //         }

    //         playbackAnimationId = requestAnimationFrame(animate);
    //     };

    //     playbackAnimationId = requestAnimationFrame(animate);
    // };

    // const pausePlayback = () => {
    //     is_playing.value = false;
    //     if (playbackAnimationId !== null) { 
    //         cancelAnimationFrame(playbackAnimationId);
    //         playbackAnimationId = null;
    //     }
    // };

    // const seekToFrame = (index: number) => {
    //     currentFrameIndex.value = Math.max(0, Math.min(index, clip_length.value - 1));
    //     applyFrame(currentFrameIndex.value);
    // };

    // const applyFrame = (index: number) => {
    //     if (!robot_renderer || !hasFrames.value) return;
    //     const frame = main_clip.frames[index];
    //     if (frame) {
    //         frame.apply(robot_renderer);
    //     }
    // };

    // WebSocket连接函数
    const connectWebSocket = (port: string) => {
        const wsUrl = `ws://localhost:${port}/ws`;

        try {
            const ws = new WebSocket(wsUrl);

            ws.onopen = (ev: Event) => {
                console.log('WebSocket connected successfully');
                console.log(new Date().toLocaleString());
                console.log(ev);
            };

            ws.onmessage = (event) => {
                (event.data as Blob).arrayBuffer().then((buffer: ArrayBuffer) => {
                    const frame = new Frame(buffer);
                    apply_frame(frame, robot_renderer!);
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

            const urlParams = new URLSearchParams(window.location.search);
            const addr = urlParams.get('addr');
            if (addr) {
                connectWebSocket(addr);
            }
        } else {
            console.error('canvas not found');
        }
    });

    onUnmounted(() => {
        // pausePlayback();
        if (robot_renderer) {
            // robot_renderer.cleanup();
        }
    });
</script>

<template>
    <div ref="canvas" class="w-full h-full"></div>
</template>
