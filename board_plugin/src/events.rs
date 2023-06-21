use crate::components::Coordinates;

#[derive(Debug, Clone, Copy)]
pub struct TileTriggerEvent(pub Coordinates);

#[derive(Debug, Copy, Clone)]
pub struct BoardCompletedEvent;

#[derive(Debug, Copy, Clone)]
pub struct BombExplosionEvent;

#[derive(Debug, Copy, Clone)]
pub struct TileMarkEvent(pub Coordinates);