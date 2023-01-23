/*
 * Dylan Gilson
 * dylan.gilson@outlook.com
 * January 22, 2023
 */

use bevy::{
    prelude::{App, Input, KeyCode, Plugin, Res, ResMut, Query},
    tasks::{AsyncComputeTaskPool, TaskPool}
};

use crate::{
    cell_renderer::{InstanceData, InstanceMaterialData},
    rule::Rule
};

pub trait Simulation: Send + Sync {
    fn update(&mut self, input: &Input<KeyCode>, rule: &Rule, task_pool: &TaskPool);
    fn render(&self, rule: &Rule, data: &mut Vec<InstanceData>);
    fn reset(&mut self, rule: &Rule);
    fn cell_count(&self) -> usize;
}

pub struct Simulations {
    simulations: Vec<(&'static str, Box<dyn Simulation>)>,
    active_simulation: Option<usize>
}

impl Simulations {
    pub fn new() -> Simulations {
        Simulations {
            simulations: vec![],
            active_simulation: None
        }
    }

    pub fn add_simulation(&mut self, name: &'static str, simulation: Box<dyn Simulation>) {
        self.simulations.push((name, simulation));
    }
}

pub fn update(mut this: ResMut<Simulations>, rule: Res<Rule>, input: Res<Input<KeyCode>>, mut query: Query<&mut InstanceMaterialData>,
              task_pool: Res<AsyncComputeTaskPool>) {
    let mut new_active = None;

    // default to simulation 1 on launch
    if this.active_simulation == None {
        new_active = Some(0);
    }

    // reset simulation
    if input.just_pressed(KeyCode::Key1) {
        new_active = Some(0);
    }

    if let Some(new_active) = new_active {
        this.active_simulation = Some(new_active);
        this.simulations[new_active].1.reset(&rule);
    }

    if let Some(active) = this.active_simulation {
        let simulation = &mut this.simulations[active].1;

        simulation.update(&input, &rule, &task_pool.0);

        let mut instance_data = query.iter_mut().next().unwrap();

        instance_data.0.clear();

        simulation.render(&rule, &mut instance_data.0);
    }
}

pub struct SimulationsPlugin;

impl Plugin for SimulationsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Simulations::new()).add_system(update);
    }
}
