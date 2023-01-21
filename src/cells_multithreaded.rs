/*
 * Dylan Gilson
 * dylan.gilson@outlook.com
 * January 20, 2023
 */

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use::bevy::{
    input::Input,
    math::{ivec3, vec3, IVec3},
    prelude::{Color, KeyCode, Plugin, Query, Res, ResMut},
    tasks::{AsyncComputeTaskPool, Task}
};

use futures_lite::future;

use crate::{
    cell_renderer::{InstanceData, InstanceMaterialData},
    neighbours::MOORE_NEIGHBOURS,
    rule::Rule,
    utils::{self, keep_in_bounds},
    State
};

pub enum StateChange {
    Decay,
    Spawn
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProcessStep {
    Ready,
    CalculateNeighbours,
    AwaitNeighbours,
    CalculateChanges,
    AwaitChanges // this state also applies the changes
}

impl ProcessStep {
    pub fn advance_to_next_step(&mut self) {
        match self {
            ProcessStep::Ready => *self = ProcessStep::CalculateNeighbours,
            ProcessStep::CalculateNeighbours => *self = ProcessStep::AwaitNeighbours,
            ProcessStep::AwaitNeighbours => *self = ProcessStep::CalculateChanges,
            ProcessStep::CalculateChanges => *self = ProcessStep::AwaitChanges,
            ProcessStep::AwaitChanges => *self = ProcessStep::Ready
        }
    }
}

struct CellsMultithreaded {
    states: Arc<RwLock<HashMap<IVec3, State>>>,

    // cached data used for calculating state
    neighbours: Arc<RwLock<HashMap<IVec3, u8>>>,
    changes: HashMap<IVec3, StateChange>,

    neighbour_jobs: Vec<Option<Task<Vec<IVec3>>>>,
    change_jobs: Vec<Option<Task<Vec<(IVec3, StateChange)>>>>,
    process_step: ProcessStep,

    instance_material_data: Option<Vec<InstanceData>> // instance buffer data
}

impl CellsMultithreaded {
    pub fn new(rule: &Rule) -> Self {
        let s = CellsMultithreaded {
            states: Arc::new(RwLock::new(HashMap::new())),
            neighbours: Arc::new(RwLock::new(HashMap::new())),
            changes: HashMap::new(),
            neighbour_jobs: Vec::new(),
            change_jobs: Vec::new(),
            process_step: ProcessStep::CalculateNeighbours,
            instance_material_data: None
        };

        utils::spawn_noise(&mut s.states.write().unwrap(), rule);

        s
    }

    pub fn ready(&mut self) {
        if self.process_step == ProcessStep::Ready {
            self.process_step.advance_to_next_step()
        }
    }

    pub fn calculate_neighbours(&mut self, rule: &Rule, task_pool: Res<AsyncComputeTaskPool>) {
        let (x_range, y_range, z_range) = rule.get_bounding_ranges();

        for z in z_range.clone() {
            for y in y_range.clone() {
                for x in x_range.clone() {
                    // prepare data for thread
                    let state_rc_clone = self.states.clone();
                    let rule_states = rule.states;
                    let rule_bounding = rule.bounding;
                    let cell_position = ivec3(x, y, z);

                    let neighbour_task = task_pool.spawn(async move {
                        let states = state_rc_clone.read().unwrap();
                        let mut results: Vec<IVec3> = vec![];

                        if let Some(cell) = states.get(&cell_position) {
                            // count as neighbour if new
                            if cell.value == rule_states {
                                // get neighbouring cells and increment
                                for dir in MOORE_NEIGHBOURS.iter() {
                                    let mut neighbour_position = cell_position + *dir;

                                    keep_in_bounds(rule_bounding, &mut neighbour_position);

                                    results.push(neighbour_position);
                                }
                            }
                        }

                        results
                    });

                    self.neighbour_jobs.push(Some(neighbour_task));
                }
            }
        }
    }

    pub fn calculate_changes(&mut self, rule: &Rule, task_pool: Res<AsyncComputeTaskPool>) {
        let (x_range, y_range, z_range) = rule.get_bounding_ranges();

        for z in z_range.clone() {
            for y in y_range.clone() {
                for x in x_range.clone() {
                    // prepare data for thread
                    let cell_position = ivec3(x, y, z);
                    let state_rc_clone = self.states.clone();
                    let neighbours_rc_clone = self.neighbours.clone();
                    let rule_survival_rule = rule.survival_rule.clone();
                    let rule_birth_rule = rule.birth_rule.clone();
                    let rule_start_state_value = rule.start_state_value;
                    let rule_bounding = rule.bounding;

                    let changes_task = task_pool.spawn(async move {
                        let states = state_rc_clone.read().unwrap();
                        let mut changes = Vec::new();
                        let neighbours = match neighbours_rc_clone.read().unwrap().get(&cell_position) {
                            Some(n) => *n,
                            None => 0
                        };

                        match states.get(&cell_position) {
                            Some(cell) => {
                                if !(rule_survival_rule.in_range(neighbours) && cell.value == rule_start_state_value) {
                                    changes.push((cell_position, StateChange::Decay));
                                }
                            },
                            None => {
                                // check if cell should spawn
                                if rule_birth_rule.in_range(neighbours) {
                                    if cell_position.x >= -rule_bounding && cell_position.x <= rule_bounding
                                        && cell_position.y >= -rule_bounding && cell_position.y <= rule_bounding
                                        && cell_position.z >= -rule_bounding && cell_position.z <= rule_bounding {
                                            changes.push((cell_position, StateChange::Spawn));
                                    }
                                }
                            }
                        }

                        changes
                    });

                    self.change_jobs.push(Some(changes_task));
                }
            }
        }
    }

    pub fn apply_changes(&mut self, rule: &Rule) {
        let mut states = self.states.write().unwrap();

        // apply new spawns
        for (cell_position, state_change) in self.changes.iter() {
            match state_change {
                StateChange::Decay => {
                    let mut cell = states.get_mut(cell_position).unwrap();

                    // DECAY by 1
                    let value = cell.value as i32 - 1;
                    let value = i32::min(value, rule.states as i32);

                    cell.value = value as u8;
                },
                StateChange::Spawn => {
                    states.insert(*cell_position, State::new(rule.start_state_value));
                }
            }
        }

        states.retain(|_, c| c.value > 0); // remove dead cells

        let mut instance_data = Vec::new();

        // update instance buffer
        for cell in states.iter() {
            let position = cell.0;

            instance_data.push(InstanceData {
                position: vec3(position.x as f32, position.y as f32, position.z as f32),
                scale: 1.0,
                colour: Color::rgba(cell.1.value as f32 / rule.states as f32, 0.0, 0.0, 1.0).as_rgba_f32()
            });
        }

        self.instance_material_data = Some(instance_data);

        // all calculations are complete, reset cache data
        self.changes.clear();
        self.neighbours.write().unwrap().clear();
    }

    pub fn tick(&mut self, rule: &Rule, task_pool: Res<AsyncComputeTaskPool>) {
        let advance = match self.process_step {
            ProcessStep::Ready => false,
            ProcessStep::CalculateNeighbours => {
                self.calculate_neighbours(rule, task_pool);
                true
            },
            ProcessStep::AwaitNeighbours => {
                for job in self.neighbour_jobs.iter_mut() {
                    let mut task = job.take().unwrap();

                    match future::block_on(future::poll_once(&mut task)) {
                        Some(results) => {
                            let mut neighbours = self.neighbours.write().unwrap();

                            for neighbour_position in results.into_iter() {
                                if !neighbours.contains_key(&neighbour_position) {
                                    neighbours.insert(neighbour_position, 0);
                                }

                                let neighbour = neighbours.get_mut(&neighbour_position).unwrap();

                                *neighbour += 1;
                            }
                        },
                        None => *job = Some(task) // failed to retrieve data, continue
                    }
                }

                self.neighbour_jobs.retain(|job| job.is_some()); // remove completed tasks

                self.neighbour_jobs.is_empty() // no jobs -> advance process step ; some jobs left -> stay with this process step
            },
            ProcessStep::CalculateChanges => {
                self.calculate_changes(rule, task_pool);

                true
            },
            ProcessStep::AwaitChanges => {
                for job in self.change_jobs.iter_mut() {
                    let mut task = job.take().unwrap();

                    match future::block_on(future::poll_once(&mut task)) {
                        Some(state_changes) => {
                            let mut states = self.states.write().unwrap();

                            for (cell_position, state_change) in state_changes.into_iter() {
                                match state_change {
                                    StateChange::Decay => {
                                        let mut cell = states.get_mut(&cell_position).unwrap();

                                        cell.value -= 1;
                                    },
                                    StateChange::Spawn => {
                                        states.insert(cell_position, State::new(rule.start_state_value));
                                    }
                                }
                            }
                        },
                        None => *job = Some(task) // failed to retrieve data, continue
                    }
                }

                // remove completed tasks
                self.change_jobs.retain(|job| job.is_some());

                let done = self.change_jobs.is_empty();

                if done {
                    self.apply_changes(rule);
                }

                done
            }
        };

        if advance {
            self.process_step.advance_to_next_step();
        }
    }
}

fn tick_cell(rule: Res<Rule>, mut cells: ResMut<CellsMultithreaded>, keyboard_input: Res<Input<KeyCode>>, task_pool: Res<AsyncComputeTaskPool>) {
    cells.tick(&rule, task_pool);

    if keyboard_input.pressed(KeyCode::E) {
        cells.ready();

        return;
    }
}

fn spawn_noise(rule: Res<Rule>, mut cells: ResMut<CellsMultithreaded>, keyboard_input: Res<Input<KeyCode>>) {
    if !keyboard_input.just_pressed(KeyCode::P) {
        return;
    }
}

fn prepare_cell_data(mut cells: ResMut<CellsMultithreaded>, mut query: Query<&mut InstanceMaterialData>) {
    if let Some(mut instance_material_data) = cells.instance_material_data.take() {
        let mut instance_data = query.iter_mut().next().unwrap();

        instance_data.0.clear();
        instance_data.0.append(&mut instance_material_data);
    }
}

pub struct CellsMultiThreadedPlugin;

impl Plugin for CellsMultiThreadedPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let rule = app.world.get_resource::<Rule>().unwrap();
        let cells_multithreaded = CellsMultithreaded::new(&rule);

        app.insert_resource(cells_multithreaded).add_system(prepare_cell_data).add_system(spawn_noise).add_system(tick_cell);
    }
}
