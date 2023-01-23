/*
 * Dylan Gilson
 * dylan.gilson@outlook.com
 * January 22, 2023
 */

use bevy::{
    input::Input,
    math::{ivec3, IVec3},
    prelude::KeyCode,
    tasks::TaskPool
};

use futures_lite::future;

use crate::{
    cell_renderer::InstanceData,
    rule::Rule,
    simulation::Simulation,
    utils::{self}
};

const CHUNK_SIZE: usize = 32;
const CHUNK_CELL_COUNT: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

fn index_to_chunk_index(index: usize) -> usize {
    index / CHUNK_CELL_COUNT
}

fn index_to_chunk_offset(index: usize) -> usize {
    index % CHUNK_CELL_COUNT
}

struct Chunk<Cell>(Vec<Cell>);

impl<Cell: Default> Default for Chunk<Cell> {
    fn default() -> Self {
        let cells = (0..CHUNK_CELL_COUNT).map(|_| Cell::default()).collect::<Vec<_>>();

        Chunk(cells)
    }
}

impl<Cell> Chunk<Cell> {
    fn index_to_position(index: usize) -> IVec3 {
        utils::index_to_position(index, CHUNK_SIZE as i32)
    }

    fn position_to_index(position: IVec3) -> usize {
        utils::position_to_index(position, CHUNK_SIZE as i32)
    }

    fn is_border_position(position: IVec3, offset: i32) -> bool {
        position.x - offset <= 0 || position.x + offset >= CHUNK_SIZE as i32 - 1
            || position.y - offset <= 0 || position.y + offset >= CHUNK_SIZE as i32 - 1
            || position.z - offset <= 0 || position.z + offset >= CHUNK_SIZE as i32 - 1
    }
}

#[derive(Clone, Copy, Default)]
struct Cell {
    value: u8,
    neighbours: u8
}

impl Cell {
    fn is_dead(self) -> bool {
        self.value == 0
    }
}

pub struct MultiThreaded {
    chunks: Vec<Chunk<Cell>>,
    chunk_radius: usize,
    chunk_count: usize
}

impl MultiThreaded {
    pub fn new() -> Self {
        MultiThreaded {
            chunks: vec![],
            chunk_radius: 0,
            chunk_count: 0
        }
    }

    pub fn set_size(&mut self, new_size: usize) -> usize {
        let radius = (new_size + CHUNK_SIZE - 1) / CHUNK_SIZE;

        if radius != self.chunk_radius {
            let count = radius * radius * radius;

            self.chunks.resize_with(count, || Chunk::default());
            self.chunk_radius = radius;
            self.chunk_count  = count;
        }

        self.size()
    }

    pub fn size(&self) -> usize {
        self.chunk_radius * CHUNK_SIZE
    }

    pub fn center(&self) -> IVec3 {
        let center = (self.size() / 2) as i32;

        ivec3(center, center, center)
    }

    pub fn cell_count(&self) -> usize {
        let mut count = 0;

        for chunk in &self.chunks {
            for cell in chunk.0.iter() {
                if !cell.is_dead() {
                    count += 1;
                }
            }
        }

        count
    }

    fn index_to_vector(&self, index: usize) -> IVec3 {
        let chunk = index_to_chunk_index(index);
        let offset = index_to_chunk_offset(index);
        let chunk_vector = utils::index_to_position(chunk, self.chunk_radius as i32);
        let offset_vector = Chunk::<Cell>::index_to_position(offset);

        (CHUNK_SIZE as i32 * chunk_vector) + offset_vector
    }

    fn vector_to_index(&self, vector: IVec3) -> usize {
        let chunk_vector = vector / CHUNK_SIZE as i32;
        let offset_vector = vector % CHUNK_SIZE as i32;

        let chunk = utils::position_to_index(chunk_vector, self.chunk_radius as i32);
        let offset = Chunk::<Cell>::position_to_index(offset_vector);

        chunk * CHUNK_CELL_COUNT + offset
    }

    fn wrap(&self, position: IVec3) -> IVec3 {
        utils::wrap(position, self.size() as i32)
    }

    fn update_neighbours_chunk(chunk: &mut Chunk<Cell>, rule: &Rule, offset: usize, increment: bool) {
        let position = Chunk::<Cell>::index_to_position(offset);

        for dir in rule.neighbour_method.get_neighbour_iter() {
            let neighbour_position = position + *dir;

            let index = Chunk::<Cell>::position_to_index(neighbour_position);

            if increment {
                chunk.0[index].neighbours += 1;
            } else {
                chunk.0[index].neighbours -= 1;
            }
        }
    }

    fn update_neighbours(&self, chunks: &mut Vec<Chunk<Cell>>, rule: &Rule, index: usize, increment: bool) {
        let position = self.index_to_vector(index);

        for dir in rule.neighbour_method.get_neighbour_iter() {
            let neighbour_position = self.wrap(position + *dir);
            let index = self.vector_to_index(neighbour_position);
            let chunk = index_to_chunk_index(index);
            let offset = index_to_chunk_offset(index);

            if increment {
                chunks[chunk].0[offset].neighbours += 1;
            } else {
                chunks[chunk].0[offset].neighbours -= 1;
            }
        }
    }

    fn update_values_chunk(chunk: &mut Chunk<Cell>, chunk_index: usize, rule: &Rule,
                           chunk_spawns: &mut Vec<usize>, spawns: &mut Vec<usize>,
                           chunk_deaths: &mut Vec<usize>, deaths: &mut Vec<usize>) {
        for (offset, cell) in chunk.0.iter_mut().enumerate() {
            if cell.is_dead() {
                if rule.birth_rule.in_range(cell.neighbours) {
                    cell.value = rule.states;

                    if Chunk::<Cell>::is_border_position(Chunk::<Cell>::index_to_position(offset), 0) {
                        spawns.push(chunk_index * CHUNK_CELL_COUNT + offset);
                    } else {
                        chunk_spawns.push(offset);
                    }
                }
            } else {
                if cell.value < rule.states || !rule.survival_rule.in_range(cell.neighbours) {
                    if cell.value == rule.states {
                        if Chunk::<Cell>::is_border_position(Chunk::<Cell>::index_to_position(offset), 0) {
                            deaths.push(chunk_index * CHUNK_CELL_COUNT + offset);
                        } else {
                            chunk_deaths.push(offset);
                        }
                    }

                    cell.value -= 1;
                }
            }
        }
    }

    pub fn update(&mut self, rule: &Rule, tasks: &TaskPool) {
        self.set_size(rule.bounding_size as usize);

        let mut chunks = std::mem::take(&mut self.chunks);

        // update values
        let mut value_tasks = vec![];
        for (chunk_index, mut chunk) in chunks.into_iter().enumerate() {
            let rule = rule.clone();
            let mut chunk_spawns = vec![];
            let mut chunk_deaths = vec![];
            let mut spawns = vec![];
            let mut deaths = vec![];

            value_tasks.push(tasks.spawn(async move {
                Self::update_values_chunk(&mut chunk, chunk_index, &rule, &mut chunk_spawns, &mut spawns, &mut chunk_deaths, &mut deaths);

                (chunk, chunk_spawns, spawns, chunk_deaths, deaths)
            }));
        }

        // collect spawns + deaths
        chunks = vec![];

        let mut chunk_spawns = vec![];
        let mut chunk_deaths = vec![];
        let mut spawns = vec![];
        let mut deaths = vec![];

        for task in value_tasks {
            let (chunk, in_spawns, out_spawns, in_deaths, out_deaths) = future::block_on(task);

            chunks.push(chunk);
            chunk_spawns.push(in_spawns);
            chunk_deaths.push(in_deaths);
            spawns.extend(out_spawns);
            deaths.extend(out_deaths);
        }

        // update neighbours in parallel
        let mut neighbour_tasks = vec![];

        for ((mut chunk, spawns), deaths) in chunks.into_iter().zip(chunk_spawns).zip(chunk_deaths) {
            let rule = rule.clone();

            neighbour_tasks.push(tasks.spawn(async move {
                for offset in spawns {
                    Self::update_neighbours_chunk(&mut chunk, &rule, offset, true);
                }

                for offset in deaths {
                    Self::update_neighbours_chunk(&mut chunk, &rule, offset, false);
                }

                chunk
            }));
        }

        // collect chunks
        chunks = vec![];

        for task in neighbour_tasks {
            let chunk = future::block_on(task);

            chunks.push(chunk);
        }

        // update neighbours in serial
        for index in spawns {
            self.update_neighbours(&mut chunks, rule, index, true);
        }

        for index in deaths {
            self.update_neighbours(&mut chunks, rule, index, false);
        }

        self.chunks = chunks;
    }

    pub fn spawn_noise(&mut self, rule: &Rule) {
        let mut chunks = std::mem::take(&mut self.chunks);

        utils::spawn_noise_default(self.center(), |position| {
            let index = self.vector_to_index(self.wrap(position));
            let chunk = index_to_chunk_index(index);
            let offset = index_to_chunk_offset(index);
            let cell = &mut chunks[chunk].0[offset];

            if cell.is_dead() {
                cell.value = rule.states;
                self.update_neighbours(&mut chunks, rule, index, true);
            }
        });

        self.chunks = chunks;
    }
}

impl Simulation for MultiThreaded {
    fn update(&mut self, input: &Input<KeyCode>, rule: &Rule, task_pool: &TaskPool) {
        self.set_size(rule.bounding_size as usize);

        if !input.pressed(KeyCode::P) {
            return;
        }

        self.spawn_noise(rule);
        self.update(rule, task_pool);
    }

    fn render(&self, rule: &Rule, data: &mut Vec<InstanceData>) {
        for (chunk_index, chunk) in self.chunks.iter().enumerate() {
            for (index, cell) in chunk.0.iter().enumerate() {
                if cell.is_dead() {
                    continue;
                }

                let position = self.index_to_vector(chunk_index * CHUNK_CELL_COUNT + index);
                data.push(InstanceData {
                    position: (position - self.center()).as_vec3(),
                    scale: 1.0,
                    colour: rule.colour_method.colour(rule.states, cell.value, cell.neighbours, utils::distance_to_center(position, &rule)).as_rgba_f32()
                });
            }
        }
    }

    fn reset(&mut self, _rule: &Rule) {
        *self = MultiThreaded::new();
    }

    fn cell_count(&self) -> usize {
        self.cell_count()
    }
}
