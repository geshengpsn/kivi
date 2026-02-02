import { ArrowHelper, Color, Matrix4, Mesh, MeshStandardMaterial, Object3D, Scene, Vector3 } from "three";
import type { RobotRenderer } from "./renderer";
import { STLLoader } from "three/examples/jsm/Addons.js";

const DataType = {
    NullData: 0,
    Stl: 1,
    MeshMaterial: 2,
    Matrix4: 3,
    Arrow3: 4,
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
        case DataType.NullData:
            return null;
        case DataType.Stl:
            return new TextDecoder('utf-8').decode(data);
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

function apply_by_data(object: Mesh, data_index: DataTypeIndex, data: any) {
    switch (data_index) {
        case DataType.NullData:
            object.parent?.remove(object);
            object.geometry.dispose();
            break;
        case DataType.Stl:
            new STLLoader().loadAsync("http://localhost:9876/userfile/" + data as string).then((geom) => {
                object.geometry = geom;
            });
            break;
        case DataType.MeshMaterial:
            object.material = new MeshStandardMaterial({
                color: new Color(data.color[0] / 255, data.color[1] / 255, data.color[2] / 255),
                roughness: data.roughness,
                metalness: data.metalness,
            });
            break;
        case DataType.Matrix4:
            const matrix = new Matrix4().fromArray(data);
            console.log(matrix);
            matrix.decompose(object.position, object.quaternion, new Vector3());
            // object.matrix.set(data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15]);
            break;
        case DataType.Arrow3:
            let old_arrow = object.children.find((child) => child.name === "/ARROW3");
            if (old_arrow) {
                let start = new Vector3(data.start_end[0], data.start_end[1], data.start_end[2]);
                let end = new Vector3(data.start_end[3], data.start_end[4], data.start_end[5]);
                let length = start.distanceTo(end);
                let dir = end.clone().sub(start).normalize();
                (old_arrow as ArrowHelper).setColor(new Color(data.color[0] / 255, data.color[1] / 255, data.color[2] / 255));
                (old_arrow as ArrowHelper).setDirection(dir);
                (old_arrow as ArrowHelper).setLength(length);
                (old_arrow as ArrowHelper).position.copy(start);
            } else {
                let start = new Vector3(data.start_end[0], data.start_end[1], data.start_end[2]);
                let end = new Vector3(data.start_end[3], data.start_end[4], data.start_end[5]);
                let length = start.distanceTo(end);
                let dir = end.clone().sub(start).normalize();
                let arrow = new ArrowHelper(
                    dir,
                    start,
                    length,
                    new Color(data.color[0] / 255, data.color[1] / 255, data.color[2] / 255),
                );
                arrow.name = "/ARROW3";
                object.add(arrow);
            }
            break;
        default:
            throw new Error(`Unknown data type: ${data_index}`);
    }
}

export class Frame {
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

    apply(scene: Scene) {
        if (this.path.length === 0) {
            return;
        }

        let object: any = scene;
        for (let i = 0; i < this.path.length; i++) {
            const tag = this.path[i]!;
            const child = (object as Object3D).children.find((child) => child.name === tag);
            let is_last = i === this.path.length - 1;

            if (child) {
                // child found
                object = child;
            } else {
                // child not found
                const new_object = new Mesh();
                new_object.name = tag;
                new_object.material = new MeshStandardMaterial({
                    color: new Color(0xffffff)
                });
                object.add(new_object);
                object = new_object;
            }
            if (is_last) {
                apply_by_data(object, this.type_index, this.data);
            }
        }
    }
}

export function apply_frame(frame: Frame, renderer: RobotRenderer) {
    console.log(frame);
    frame.apply(renderer.scene);
}