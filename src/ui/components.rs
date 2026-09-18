use bevy::prelude::*;

#[derive(Component)]
pub struct HpBarRoot {
    pub owner: Entity,
    pub visual_height: f32,
}

#[derive(Component)]
pub struct HpBarFill;

#[derive(Component)]
pub struct HpBarRef {
    pub root: Entity,
    pub fill: Entity,
}
#[derive(Component, Default)]
pub struct DragSelection {
    pub start: Vec2,
    pub current: Vec2,
    pub active: bool,
}

pub enum UiTweenState {
    Moving,
    Finished,
}

pub struct TweenBehavior {
    pub from: f32,
    pub to: f32,
    pub unit: Val,
}

impl TweenBehavior {
    pub const ZERO: Self = TweenBehavior {
        from: 0.0,
        to: 0.0,
        unit: Val::Px(1.0),
    };
    pub fn new(from: f32, to: f32, unit: Val) -> Self {
        TweenBehavior { from, to, unit }
    }

    pub fn from_vals(start: Val, end: Val) -> Self {
        if start.try_add(end).is_err() {
            panic!("Cannot tween between different Val types: {:?} and {:?}", start, end);
        }
        let from = match start {
            Val::Px(px) => px,
            Val::Percent(pct) => pct,
            Val::Vw(vw) => vw,
            Val::Vh(vh) => vh,
            Val::VMin(vmin) => vmin,
            Val::VMax(vmax) => vmax,
            _ => panic!("Unsupported Val type for tweening"),
        };
        let to = match end {
            Val::Px(px) => px,
            Val::Percent(pct) => pct,
            Val::Vw(vw) => vw,
            Val::Vh(vh) => vh,
            Val::VMin(vmin) => vmin,
            Val::VMax(vmax) => vmax,
            _ => panic!("Unsupported Val type for tweening"),
        };
        let unit = {
            let target = if start == Val::ZERO { end } else { start };
            match target {
                Val::Px(_) => Val::Px(1.0),
                Val::Percent(_) => Val::Percent(1.0),
                Val::Vw(_) => Val::Vw(1.0),
                Val::Vh(_) => Val::Vh(1.0),
                Val::VMin(_) => Val::VMin(1.0),
                Val::VMax(_) => Val::VMax(1.0),
                _ => panic!("Unsupported Val type for tweening"),
            }
        };
        TweenBehavior { from, to, unit }
    }

    pub fn from_diff(start: Val, diff: f32, unit_opt: Option<Val>) -> Self {
        if start == Val::ZERO && unit_opt.is_none() {
            panic!("Cannot tween from Val::ZERO with a diff, use from_vals instead");
        }
        let from = match start {
            Val::Px(px) => px,
            Val::Percent(pct) => pct,
            Val::Vw(vw) => vw,
            Val::Vh(vh) => vh,
            Val::VMin(vmin) => vmin,
            Val::VMax(vmax) => vmax,
            _ => panic!("Unsupported Val type for tweening"),
        };
        let to = from + diff;
        let unit = if let Some(u) = unit_opt {u} else {
            match start {
                Val::Px(_) => Val::Px(1.0),
                Val::Percent(_) => Val::Percent(1.0),
                Val::Vw(_) => Val::Vw(1.0),
                Val::Vh(_) => Val::Vh(1.0),
                Val::VMin(_) => Val::VMin(1.0),
                Val::VMax(_) => Val::VMax(1.0),
                _ => panic!("Unsupported Val type for tweening"),
            }
        };
        TweenBehavior { from, to, unit }
    }

    pub fn lerp(&self, t: f32) -> Val {
        let value = self.from + (self.to - self.from) * t;
        self.unit * value
    }
}

#[derive(Component)]
pub struct UiTween {
    pub state: UiTweenState,
    pub timer: Timer,
    pub target_left: TweenBehavior,
    pub target_up: TweenBehavior,
}
