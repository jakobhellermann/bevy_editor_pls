use bevy::{render::camera::ActiveCameras, utils::HashSet};

#[derive(Default)]
pub struct PersistentActiveCameras {
    pub inactive_cameras: HashSet<String>,
    pub active_cameras: HashSet<String>,
}

impl PersistentActiveCameras {
    pub fn update(&mut self, active_cameras: &ActiveCameras) {
        for camera in active_cameras.iter() {
            self.active_cameras.insert(camera.name.clone());
        }
    }

    pub fn enable(&mut self, camera: String, active_cameras: &mut ActiveCameras) {
        active_cameras.add(&camera);
        self.inactive_cameras.remove(&camera);
        self.active_cameras.insert(camera);
    }
    pub fn disable(&mut self, camera: String, active_cameras: &mut ActiveCameras) {
        active_cameras.remove(&camera);
        self.active_cameras.remove(&camera);
        self.inactive_cameras.insert(camera);
    }
    pub fn set_active(&mut self, camera: String, active: bool, active_cameras: &mut ActiveCameras) {
        match active {
            true => self.enable(camera, active_cameras),
            false => self.disable(camera, active_cameras),
        }
    }

    pub fn disable_all(&mut self, active_cameras: &mut ActiveCameras) {
        let all_cameras: Vec<_> = active_cameras.iter().map(|cam| cam.name.clone()).collect();

        for camera in all_cameras {
            active_cameras.remove(&camera);
            self.inactive_cameras.insert(camera);
        }
        self.active_cameras.clear();
    }

    pub fn enable_all(&mut self, active_cameras: &mut ActiveCameras) {
        for camera in self.inactive_cameras.drain() {
            active_cameras.add(&camera);
            self.active_cameras.insert(camera);
        }
    }

    pub fn all_sorted(&self) -> impl IntoIterator<Item = (&str, bool)> + '_ {
        let mut cameras = Vec::new();
        cameras.extend(
            self.inactive_cameras
                .iter()
                .map(|name| (name.as_str(), false)),
        );
        cameras.extend(self.active_cameras.iter().map(|name| (name.as_str(), true)));
        cameras.sort();

        cameras
    }
}
