use godot::prelude::*;

mod ecs;
mod godot_bridge;
mod utils;
// 1. GDExtension 라이브러리 초기화
struct GameBackend;

#[gdextension]
unsafe impl ExtensionLibrary for GameBackend {}
