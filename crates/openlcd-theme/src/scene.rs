use serde::{Deserialize, Serialize};

use crate::{BackgroundLayer, ImageLayer, Layer, LayerId, TextLayer};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub layers: Vec<Layer>,

    #[serde(skip)]
    next_layer_id: LayerId,
}

impl Scene {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            layers: Vec::new(),
            next_layer_id: 1,
        }
    }

    pub fn add_image(&mut self, mut layer: ImageLayer) -> LayerId {
        if layer.id == 0 {
            layer.id = self.next_layer_id();
        } else {
            self.next_layer_id = self.next_layer_id.max(layer.id + 1);
        }

        let id = layer.id;

        self.layers.push(Layer::Image(layer));

        id
    }

    pub fn next_layer_id(&mut self) -> LayerId {
        let id = self.next_layer_id;

        self.next_layer_id += 1;

        id
    }

    pub fn add_background(&mut self, color: [u8; 4]) -> LayerId {
        let id = self.next_layer_id();

        self.layers
            .push(Layer::Background(BackgroundLayer::new(id, color)));

        id
    }

    pub fn add_text(&mut self, mut layer: TextLayer) -> LayerId {
        if layer.id == 0 {
            layer.id = self.next_layer_id();
        } else {
            self.next_layer_id = self.next_layer_id.max(layer.id + 1);
        }

        let id = layer.id;

        self.layers.push(Layer::Text(layer));

        id
    }

    pub fn layer(&self, id: LayerId) -> Option<&Layer> {
        self.layers.iter().find(|layer| layer.id() == id)
    }

    pub fn layer_mut(&mut self, id: LayerId) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|layer| layer.id() == id)
    }

    pub fn remove_layer(&mut self, id: LayerId) -> bool {
        let old_len = self.layers.len();

        self.layers.retain(|layer| layer.id() != id);

        self.layers.len() != old_len
    }

    pub fn rebuild_layer_ids(&mut self) {
        self.next_layer_id = self.layers.iter().map(Layer::id).max().unwrap_or(0) + 1;
    }
}

#[cfg(test)]
mod tests {
    use openlcd_core::LogicalPosition;

    use super::*;

    #[test]
    fn creates_incremental_layer_ids() {
        let mut scene = Scene::new("test");

        let background = scene.add_background([0, 0, 0, 255]);

        let text = scene.add_text(TextLayer::new(
            0,
            "CPU",
            LogicalPosition::new(100.0, 200.0),
            48.0,
            [255, 255, 255, 255],
        ));

        assert_eq!(background, 1);
        assert_eq!(text, 2);
    }

    #[test]
    fn removes_layer() {
        let mut scene = Scene::new("test");

        let id = scene.add_background([0, 0, 0, 255]);

        assert!(scene.remove_layer(id));

        assert!(scene.layers.is_empty());
    }

    #[test]
    fn rebuilds_next_layer_id() {
        let mut scene = Scene::new("test");

        scene
            .layers
            .push(Layer::Background(BackgroundLayer::new(10, [0, 0, 0, 255])));

        scene.rebuild_layer_ids();

        assert_eq!(scene.next_layer_id(), 11,);
    }
}
