use godot::prelude::*;

mod ecs;
mod thing_renderer;
mod unit_manager;
// 1. GDExtension 라이브러리 초기화
struct GameBackend;

#[gdextension]
unsafe impl ExtensionLibrary for GameBackend {}
