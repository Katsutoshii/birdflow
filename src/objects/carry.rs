use crate::prelude::*;
use bevy::prelude::*;

/// Plugin for picking up items and carrying them.
/// 1) When one entity begins carrying the other, their velocity is zeroed out.
/// 2) At each step, we guarantee that all carrying entities have the same acceleration and velocity.
pub struct CarryPlugin;
impl Plugin for CarryPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CarryEvent>().add_systems(
            FixedUpdate,
            (CarryEvent::update, Carrier::update, CarriedBy::update)
                .in_set(SystemStage::PostCompute)
                .chain(),
        );
    }
}

#[derive(Event, Debug)]
pub struct CarryEvent {
    pub carrier: Entity,
    pub carried: Entity,
}

impl CarryEvent {
    /// Set up the carry.
    pub fn update(
        mut events: EventReader<Self>,
        mut query: Query<Option<&mut CarriedBy>>,
        mut commands: Commands,
    ) {
        for event in events.read() {
            dbg!(event);
            if let Some(mut carried_by) = query.get_mut(event.carried).unwrap() {
                carried_by.push(event.carrier);
            } else {
                info!("Add child");
                commands
                    .entity(event.carried)
                    .insert(CarriedBy::new(event.carrier))
                    .add_child(event.carrier);
            }
            commands
                .entity(event.carrier)
                .insert(Carrier::new(event.carried));
        }
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Carrier {
    pub entity: Entity,
}
impl Carrier {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
    /// Cleanup invalid carriers.
    pub fn update(
        carriers: Query<(Entity, &Carrier)>,
        carried: Query<&CarriedBy>,
        mut commands: Commands,
    ) {
        for (entity, carrier) in &carriers {
            if carried.get(carrier.entity).is_err() {
                commands.entity(entity).remove::<Self>();
            }
        }
    }
}

#[derive(Component, Deref, DerefMut, Default)]
pub struct CarriedBy(pub Vec<Entity>);
impl CarriedBy {
    pub fn new(entity: Entity) -> Self {
        Self(vec![entity])
    }
    /// Accululate acceleration from all carriers.
    pub fn update(
        mut carried: Query<(Entity, &mut Self, &mut Acceleration), Without<Carrier>>,
        mut carriers_query: Query<&mut Acceleration, With<Carrier>>,
        mut commands: Commands,
    ) {
        for (entity, mut carriers, mut acceleration) in &mut carried {
            let mut valid_carriers = Vec::default();
            for &carrier in carriers.iter() {
                if let Ok(mut carrier_acceleration) = carriers_query.get_mut(carrier) {
                    valid_carriers.push(carrier);
                    *acceleration += *carrier_acceleration;
                    *carrier_acceleration = Acceleration::ZERO;
                }
            }
            carriers.0 = valid_carriers;
            if carriers.is_empty() {
                commands.entity(entity).remove::<Self>();
            }
        }
    }
}
