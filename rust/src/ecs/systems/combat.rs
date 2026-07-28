use crate::ecs::prelude::*;
use bevy_ecs::prelude::*;

pub fn attack_system(
    mut commands: Commands,
    mut attackers: Query<(Entity, &mut Attacking, &UnitBattleStats)>,
    attack_orders: Query<&AttackOrder>,
    time: Res<Time>,
) {
    let delta_time = time.delta;
    attackers
        .iter_mut()
        .for_each(|(entity, mut attacking, battle_stats)| {
            if let Ok(attack_order) = attack_orders.get(attacking.order) {
                // 공격 쿨다운 감소
                attacking.cooldown -= delta_time;
                if attacking.cooldown <= 0.0 {
                    commands.trigger(DamageEvent {
                        sender: entity,
                        receiver: attack_order.target,
                        damage: battle_stats.attack_damage,
                    });
                    // 공격 후 쿨다운 초기화
                    attacking.cooldown = battle_stats.attack_cooldown;
                }
            } else {
                // 공격 명령이 존재하지 않으면 Attacking 컴포넌트 제거
                commands.entity(entity).remove::<Attacking>();
            }
        });
}
