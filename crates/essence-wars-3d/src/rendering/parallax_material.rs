//! Parallax Card Material for gem tokens.
//!
//! This module implements a custom material that creates a "window into another dimension"
//! effect for card art displayed on creature gem tokens using parallax offset mapping.

use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

/// Plugin that registers the parallax card material.
pub struct ParallaxMaterialPlugin;

impl Plugin for ParallaxMaterialPlugin {
    fn build(&self, app: &mut App) {
        // Load the shader from assets
        app.add_plugins(MaterialPlugin::<ParallaxCardMaterial>::default());
    }
}

/// Custom material for parallax card art effect.
///
/// This material uses a depth texture to create a 3D parallax illusion,
/// making the card art appear to have depth when viewed from different angles.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ParallaxCardMaterial {
    /// Base color tint (multiplied with texture)
    #[uniform(0)]
    pub base_color: LinearRgba,

    /// Emissive color for glow effects
    #[uniform(0)]
    pub emissive: LinearRgba,

    /// Metallic factor (0.0 = dielectric, 1.0 = metallic)
    #[uniform(0)]
    pub metallic: f32,

    /// Roughness factor (0.0 = smooth, 1.0 = rough)
    #[uniform(0)]
    pub roughness: f32,

    /// Parallax depth intensity (how much the parallax effect displaces)
    #[uniform(0)]
    pub parallax_depth: f32,

    /// Number of parallax sampling layers
    #[uniform(0)]
    pub parallax_layers: f32,

    /// Flags packed into a u32:
    /// - bit 0: use_card_texture (1 = use texture, 0 = solid color)
    /// - bit 1: is_exhausted (dimmed appearance)
    /// - bit 2: is_damaged (red tint)
    #[uniform(0)]
    pub flags: u32,

    /// Padding for alignment
    #[uniform(0)]
    pub _padding: f32,

    /// Card art texture (the actual card image)
    #[texture(1)]
    #[sampler(2)]
    pub card_texture: Option<Handle<Image>>,

    /// Depth map texture (grayscale, white = near, black = far)
    #[texture(3)]
    #[sampler(4)]
    pub depth_texture: Option<Handle<Image>>,
}

impl Default for ParallaxCardMaterial {
    fn default() -> Self {
        Self {
            base_color: LinearRgba::WHITE,
            emissive: LinearRgba::BLACK,
            metallic: 0.0,
            roughness: 0.5,
            parallax_depth: 0.05,
            parallax_layers: 16.0,
            flags: 0,
            _padding: 0.0,
            card_texture: None,
            depth_texture: None,
        }
    }
}

impl ParallaxCardMaterial {
    /// Create a new parallax material with card art and depth textures.
    pub fn new(
        card_texture: Handle<Image>,
        depth_texture: Handle<Image>,
    ) -> Self {
        Self {
            flags: 1, // use_card_texture = true
            card_texture: Some(card_texture),
            depth_texture: Some(depth_texture),
            ..default()
        }
    }

    /// Create a solid color material (no card texture).
    pub fn solid_color(color: Color) -> Self {
        Self {
            base_color: color.into(),
            flags: 0, // use_card_texture = false
            ..default()
        }
    }

    /// Set the exhausted state (dimmed appearance).
    pub fn with_exhausted(mut self, exhausted: bool) -> Self {
        if exhausted {
            self.flags |= 2; // Set bit 1
        } else {
            self.flags &= !2; // Clear bit 1
        }
        self
    }

    /// Set the damaged state (red tint).
    pub fn with_damaged(mut self, damaged: bool) -> Self {
        if damaged {
            self.flags |= 4; // Set bit 2
        } else {
            self.flags &= !4; // Clear bit 2
        }
        self
    }

    /// Set the parallax depth intensity.
    pub fn with_parallax_depth(mut self, depth: f32) -> Self {
        self.parallax_depth = depth;
        self
    }

    /// Set the number of parallax layers.
    pub fn with_parallax_layers(mut self, layers: f32) -> Self {
        self.parallax_layers = layers;
        self
    }

    /// Set metallic and roughness for different gem appearances.
    pub fn with_pbr(mut self, metallic: f32, roughness: f32) -> Self {
        self.metallic = metallic;
        self.roughness = roughness;
        self
    }

    /// Set emissive color for glow effects.
    pub fn with_emissive(mut self, emissive: impl Into<LinearRgba>) -> Self {
        self.emissive = emissive.into();
        self
    }

    /// Check if using card texture.
    pub fn uses_card_texture(&self) -> bool {
        self.flags & 1 != 0
    }

    /// Check if exhausted.
    pub fn is_exhausted(&self) -> bool {
        self.flags & 2 != 0
    }

    /// Check if damaged.
    pub fn is_damaged(&self) -> bool {
        self.flags & 4 != 0
    }
}

impl Material for ParallaxCardMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/parallax_card.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}

/// Resource for caching loaded card textures.
#[derive(Resource, Default)]
pub struct CardTextureCache {
    /// Map from card_id to (card_texture, depth_texture)
    pub textures: std::collections::HashMap<u16, (Handle<Image>, Handle<Image>)>,
}

impl CardTextureCache {
    /// Get or load textures for a card.
    pub fn get_or_load(
        &mut self,
        card_id: u16,
        faction: &str,
        asset_server: &AssetServer,
    ) -> (Handle<Image>, Handle<Image>) {
        if let Some(handles) = self.textures.get(&card_id) {
            return handles.clone();
        }

        // Load card texture and depth map
        let card_path = format!("textures/cards/{}/{}.png", faction, card_id);
        let depth_path = format!("textures/cards/{}/{}_depth.png", faction, card_id);

        let card_texture = asset_server.load(&card_path);
        let depth_texture = asset_server.load(&depth_path);

        let handles = (card_texture, depth_texture);
        self.textures.insert(card_id, handles.clone());
        handles
    }

    /// Clear all cached textures.
    pub fn clear(&mut self) {
        self.textures.clear();
    }
}
