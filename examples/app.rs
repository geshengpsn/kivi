use kivi::{MeshMaterial, MonitorTab, Stl};
use nalgebra::matrix;
/// A WebSocket echo server
fn main() {
    let monitor = MonitorTab::new(9876, 5173, 9877);
    monitor.log("/test", Stl::from_path("./box.stl")).unwrap();
    monitor.log("/test", MeshMaterial{color: [255, 0, 0], roughness: 0.5, metalness: 0.5}).unwrap();
    for i in 0..200 {
        monitor.log("/test", matrix![
            1.0, 0.0, 0.0, 0.;
            0.0, 1.0, 0.0, 0.0;
            0.0, 0.0, 1.0, i as f64 * 0.01;
            0.0, 0.0, 0.0, 1.0;
        ]).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    monitor.wait_tab_close();
}
