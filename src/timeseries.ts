import { ref } from "vue";

const clip_library = ref<Clip[]>([]);

const clip_index = ref(0);

class Clip {
    frames: Frame[];
    name: string;
    constructor(name: string) {
        this.name = name;
        this.frames = [];
    }
}

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

function parse_data(type: DataTypeIndex, data: Uint8Array): any {
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
        this.data = parse_data(this.type_index, u8_array.slice(20 + path_length));
    }
}

