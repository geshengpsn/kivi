import { AmbientLight, Color, DirectionalLight, Fog, GridHelper, PerspectiveCamera, Scene, WebGLRenderer } from "three"
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { ref } from "vue";

export let ik_time = ref("");

export class RobotRenderer {
    renderer: WebGLRenderer
    scene: Scene
    camera: PerspectiveCamera
    constructor(
        renderer: WebGLRenderer,
        scene: Scene,
        camera: PerspectiveCamera,
    ) {
        this.renderer = renderer;
        this.scene = scene;
        this.camera = camera;
    }

    render() {
        requestAnimationFrame(this.render.bind(this));
        this.renderer.render(this.scene, this.camera);
    }
}

export function build_robot_renderer(container: HTMLDivElement) {
    // create renderer
    let renderer = new WebGLRenderer({
        // antialias
        antialias: true
    })

    // basic settings
    renderer.setPixelRatio(window.devicePixelRatio);
    renderer.setSize(window.innerWidth, window.innerHeight);
    container.appendChild(renderer.domElement);

    // create scene
    let scene = new Scene();
    // background color = gray
    scene.background = new Color(0x242424);
    scene.fog = new Fog(0x242424, 0.1, 100);

    // Perspective Camera & settings
    let camera = new PerspectiveCamera(50, window.innerWidth / window.innerHeight, 0.001, 100);
    camera.position.set(1.5, 1.5, 1.5);
    camera.lookAt(0, 0, 0);
    new OrbitControls(camera, renderer.domElement);

    // AmbientLight
    let l1 = new AmbientLight(0xFFFFFF, 1);

    // DirectionalLight
    let l2 = new DirectionalLight(0xFFFFFF, 1);
    l2.position.set(0, 1.5, 1);

    // add lights to scene
    scene.add(l1);
    scene.add(l2);

    // add grid
    let grid = new GridHelper(10, 10, 0x555555, 0x555555);
    scene.add(grid);

    robot_renderer = new RobotRenderer(renderer, scene, camera);
    robot_renderer.render();
}

export let robot_renderer: RobotRenderer | null = null;

window.onresize = function () {
    if (robot_renderer !== null) {
        robot_renderer.camera.aspect = window.innerWidth / window.innerHeight;
        robot_renderer.camera.updateProjectionMatrix();
        robot_renderer.renderer.setSize(window.innerWidth, window.innerHeight);
    }
};

