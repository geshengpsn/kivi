<script lang="ts" setup>
import { ref } from 'vue';

const togglePlayback = () => {
    console.log('togglePlayback');
}
const currentFrameIndex = ref(0);
const is_playing = ref(false);
const clip_length = ref(0);
const seekToFrame = (index: number) => {
    console.log('seekToFrame', index);
}
</script>

<template>
    <!-- Playback Control Bar -->

    <div v-if="true" class="fixed bottom-0 left-0 right-0 z-[1000]">
        <div class="flex items-center gap-4 max-w-screen-xl mx-auto bg-black/80 backdrop-blur-lg p-6 rounded-full m-4">
            <!-- Play/Pause Button -->
            <button
                @click="togglePlayback"
                class="bg-green-500 hover:bg-green-600 text-white text-xl w-10 h-10 rounded-full flex items-center justify-center transition-colors cursor-pointer border-none"
            >
                {{ is_playing ? '⏸' : '▶' }}
            </button>

            <!-- Frame Counter -->
            <span v-if="clip_length > 0" class="text-white font-mono text-sm min-w-[100px]">
                {{ currentFrameIndex + 1 }} / {{ clip_length }}
            </span>

            <!-- Timeline Slider -->
            <input
                type="range"
                class="flex-1 h-1.5 rounded-full outline-none appearance-none bg-white/20 cursor-pointer
                       [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:h-4
                       [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-green-500 [&::-webkit-slider-thumb]:cursor-pointer
                       [&::-moz-range-thumb]:w-4 [&::-moz-range-thumb]:h-4 [&::-moz-range-thumb]:rounded-full
                       [&::-moz-range-thumb]:bg-green-500 [&::-moz-range-thumb]:cursor-pointer [&::-moz-range-thumb]:border-0"
                :min="0"
                :max="clip_length - 1"
                v-model.number="currentFrameIndex"
                @input="seekToFrame(currentFrameIndex)"
            />
            <div class=""></div>
        </div>
    </div>

</template>