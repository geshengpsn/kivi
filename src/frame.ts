import type { Object3D } from "three";
import type { RobotRenderer } from "./renderer";

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

// class Box3 {
//     size: [number, number, number];
//     constructor(size: [number, number, number]) {
//         this.size = size;
//     }
// }

// class BoxLine3 {
//     size: [number, number, number];
//     constructor(size: [number, number, number]) {
//         this.size = size;
//     }
// }

// class Sphere {
//     radius: number;
//     constructor(radius: number) {
//         this.radius = radius;
//     }
// }

// class Cylinder {
//     radius: number;
//     height: number;
//     constructor(radius: number, height: number) {
//         this.radius = radius;
//         this.height = height;
//     }
// }

// class Capsule {
//     radius: number;
//     height: number;
//     constructor(radius: number, height: number) {
//         this.radius = radius;
//         this.height = height;
//     }
// }

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

function apply_by_data(object: Object3D, data_index: DataTypeIndex, data: any) {
    switch (data_index) {
        case DataType.ScalarF64:
            object.userData = data[0];
            break;
        case DataType.Box3:
            object.scale.set(data[0], data[1], data[2]);
            break;
        case DataType.BoxLine3:
            object.scale.set(data[0], data[1], data[2]);
            break;
        case DataType.Sphere:
            object.scale.set(data[0], data[0], data[0]);
            break;
        case DataType.Cylinder:
            object.scale.set(data[0], data[0], data[1]);
            break;
        case DataType.Capsule:
            object.scale.set(data[0], data[0], data[1]);
            break;
        case DataType.Stl:
            // object.geometry = new BufferGeometry();
            // object.geometry.setAttribute('position', new Float32BufferAttribute(data, 3));
            break;
        case DataType.MeshMaterial:
            // object.material = new MeshBasicMaterial({ color: new Color(data[0], data[1], data[2]) });
            break;
        case DataType.Matrix4:
            object.matrix.set(data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15]);
            break;
        case DataType.Arrow3:
            break;
        default:
            throw new Error(`Unknown data type: ${data_index}`);
    }
}

class Frame {
    timestamp: bigint;
    path: string[];
    type_index: DataTypeIndex;
    data: any;
    constructor(buffer: ArrayBuffer) {
        const data_view = new DataView(buffer);
        const u8_array = new Uint8Array(buffer);
        this.timestamp = data_view.getBigUint64(8, true) << 64n | data_view.getBigUint64(0, true);
        const path_length = data_view.getUint16(16, true);
        this.path = get_path(u8_array.slice(18, 18 + path_length)).split('/').filter((p) => p !== '');
        this.type_index = get_type(data_view.getUint16(18 + path_length, true));
        this.data = parse_data(this.type_index, u8_array.slice(20 + path_length));
    }
    apply(renderer: RobotRenderer) {
        if (this.path.length === 0) {

        }
        for (const path of this.path) {

        }
    }

}