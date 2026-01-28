import { AmbientLight, AxesHelper, Color, DirectionalLight, Fog, GridHelper, Group, Matrix4, PerspectiveCamera, Scene, WebGLRenderer, Raycaster, Vector2, Object3D } from "three"
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { ref } from "vue";

export let ik_time = ref("");

export class RobotRenderer {
    renderer: WebGLRenderer
    scene: Scene
    camera: PerspectiveCamera
    raycaster: Raycaster
    mouse: Vector2
    onObjectSelected?: (object: Object3D | null) => void
    private clickStartPos: { x: number; y: number } | null = null

    constructor(
        renderer: WebGLRenderer,
        scene: Scene,
        camera: PerspectiveCamera,
    ) {
        this.renderer = renderer;
        this.scene = scene;
        this.camera = camera;
        this.raycaster = new Raycaster();
        this.mouse = new Vector2();
        this.setupRaycasting();
    }

    setupRaycasting() {
        this.renderer.domElement.addEventListener('mousedown', this.handleMouseDown.bind(this));
        this.renderer.domElement.addEventListener('mouseup', this.handleMouseUp.bind(this));
    }

    handleMouseDown(event: MouseEvent) {
        this.clickStartPos = { x: event.clientX, y: event.clientY };
    }

    handleMouseUp(event: MouseEvent) {
        // Only trigger selection if mouse didn't move much (not a drag)
        if (this.clickStartPos) {
            const dx = event.clientX - this.clickStartPos.x;
            const dy = event.clientY - this.clickStartPos.y;
            const distance = Math.sqrt(dx * dx + dy * dy);

            if (distance < 5) { // 5px threshold
                this.handleClick(event);
            }
        }
        this.clickStartPos = null;
    }

    handleClick(event: MouseEvent) {
        const rect = this.renderer.domElement.getBoundingClientRect();
        this.mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
        this.mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;

        this.raycaster.setFromCamera(this.mouse, this.camera);

        const mainScene = this.scene.getObjectByName('main_scene');
        if (!mainScene) return;

        const intersects = this.raycaster.intersectObjects(this.scene.children, true);

        if (intersects.length > 0) {
            this.onObjectSelected?.(intersects[0].object);
        } else {
            this.onObjectSelected?.(null);
        }
    }

    setObjectSelectionCallback(callback: (object: Object3D | null) => void) {
        this.onObjectSelected = callback;
    }

    setupResizeObserver(container: HTMLElement) {
        const resizeObserver = new ResizeObserver((entries) => {
            for (const entry of entries) {
                const { width, height } = entry.contentRect;
                this.camera.aspect = width / height;
                this.camera.updateProjectionMatrix();
                this.renderer.setSize(width, height);
            }
        });
        resizeObserver.observe(container);
    }

    cleanup() {
        this.renderer.domElement.removeEventListener('mousedown', this.handleMouseDown.bind(this));
        this.renderer.domElement.removeEventListener('mouseup', this.handleMouseUp.bind(this));
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
    camera.up.set(0, 0, 1); // Set Z as up direction
    camera.position.set(1.5, 1.5, 1.5);
    camera.lookAt(0, 0, 0);
    new OrbitControls(camera, renderer.domElement);

    // AmbientLight
    let l1 = new AmbientLight(0xFFFFFF, 1);

    // DirectionalLight
    let l2 = new DirectionalLight(0xFFFFFF, 1);
    l2.position.set(0, 1, 1.5); // Adjusted for Z-up orientation

    // add lights to scene
    scene.add(l1);
    scene.add(l2);
    // add main
    // add grid (rotated to XY plane for Z-up)
    let grid = new GridHelper(10, 10, 0x666666, 0x666666);
    grid.rotation.x = Math.PI / 2; // Rotate 90 degrees to make grid on XY plane
    // grid.position.x = 0.001;
    grid.position.z = -0.001;
    scene.add(grid);

    // add axes helper
    let axes = new AxesHelper(1.5); // Slightly larger to be more visible
    scene.add(axes);

    let main_scene = new Group();
    main_scene.name = "main_scene";
    scene.add(main_scene);

    robot_renderer = new RobotRenderer(renderer, scene, camera);
    robot_renderer.render();
}

export let robot_renderer: RobotRenderer | null = null;

