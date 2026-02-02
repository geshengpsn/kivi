import { ref } from "vue";
import type { Frame } from "./frame";

class Clip {
    frames: Frame[];
    name: string;
    constructor() {
        this.name = new Date().toLocaleString();
        this.frames = [];
    }

    push_frame(frame: Frame) {
        this.frames.push(frame);
        clip_length.value = this.frames.length;
    }
    
    is_timestamp_sorted() {
        return this.frames.every((frame, i) => i === 0 || frame.timestamp >= this.frames[i - 1]!.timestamp);
    }

    clip_duration() {
        if (this.frames.length <= 1) {
            return 0;
        }
        return this.frames[this.frames.length - 1]!.timestamp - this.frames[0]!.timestamp;
    }
}

export const main_clip = new Clip();
export const clip_length = ref(0);
