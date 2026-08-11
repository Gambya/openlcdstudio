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

    pub fn move_layer_up(&mut self, id: LayerId) -> bool {
        let Some(index) = self.layers.iter().position(|layer| layer.id() == id) else {
            return false;
        };

        if matches!(self.layers[index], Layer::Background(_)) {
            return false;
        }

        if index + 1 >= self.layers.len() {
            return false;
        }

        self.layers.swap(index, index + 1);

        true
    }

    pub fn move_layer_down(&mut self, id: LayerId) -> bool {
        let Some(index) = self.layers.iter().position(|layer| layer.id() == id) else {
            return false;
        };

        if index == 0 || matches!(self.layers[index], Layer::Background(_)) {
            return false;
        }

        let target = index - 1;

        // Mantemos Background preso ao fundo.
        if matches!(self.layers[target], Layer::Background(_)) {
            return false;
        }

        self.layers.swap(index, target);

        true
    }

    pub fn is_background(&self, id: LayerId) -> bool {
        self.layer(id)
            .is_some_and(|layer| matches!(layer, Layer::Background(_)))
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
