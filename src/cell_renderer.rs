/*
 * Dylan Gilson
 * dylan.gilson@outlook.com
 * January 19, 2023
 */

use bevy::{
    core_pipeline::Transparent3d,
    ecs::{
        query::QueryItem,
        system::{lifetimeless::*, SystemParamItem}
    },
    math::prelude::*,
    pbr::{MeshPipeline, MeshPipelineKey, MeshUniform, SetMeshBindGroup, SetMeshViewBindGroup},
    prelude::*,
    render::{
        mesh::{GpuBufferInfo, MeshVertexBufferLayout},
        render_asset::RenderAssets,
        render_component::{ExtractComponent, ExtractComponentPlugin},
        render_phase::{AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase, SetItemPipeline, TrackedRenderPass},
        render_resource::*,
        renderer::RenderDevice,
        view::{ExtractedView, Msaa},
        RenderApp,
        RenderStage
    }
};

use bytemuck::{Pod, Zeroable};

use crate::utils;

pub struct CellStatesChangedEvent;

const CHUNK_SIZE: usize = 32;
pub const CHUNK_CELL_COUNT: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

// convert index to chunk index
pub fn index_to_chunk_index(index: usize) -> usize {
    index / CHUNK_CELL_COUNT
}

// convert index to chunk offset
pub fn index_to_chunk_offset(index: usize) -> usize {
    index % CHUNK_CELL_COUNT
}

pub struct Chunk<Cell>(pub Vec<Cell>);

// implement Default trait for Chunk
impl<Cell: Default> Default for Chunk<Cell> {
    fn default() -> Self {
        let cells = (0..CHUNK_CELL_COUNT).map(|_| Cell::default()).collect::<Vec<_>>();

        Chunk(cells)
    }
}

impl<Cell> Chunk<Cell> {
    // wrapper function to convert index to xyz position
    pub fn index_to_position(index: usize) -> IVec3 {
        utils::index_to_position(index, CHUNK_SIZE as i32)
    }

    // wrapper function to
    pub fn position_to_index(position: IVec3) -> usize {
        utils::position_to_index(position, CHUNK_SIZE as i32)
    }

    // returns true if xyz position touches the border ; false otherwise
    pub fn is_border_position(position: IVec3, offset: i32) -> bool {
        position.x - offset <= 0 || position.x + offset >= CHUNK_SIZE as i32 - 1
            || position.y - offset <= 0 || position.y + offset >= CHUNK_SIZE as i32 - 1
            || position.z - offset <= 0 || position.z + offset >= CHUNK_SIZE as i32 - 1
    }
}

pub struct Chunks<Cell> {
    pub chunks: Vec<Chunk<Cell>>,
    pub chunk_radius: usize,
    pub chunk_count: usize
}

impl<Cell> Chunks<Cell> {
    // create new Chunks
    pub fn new() -> Chunks<Cell> {
        Chunks {
            chunks: vec![],
            chunk_radius: 0,
            chunk_count: 0
        }
    }

    // get bounds
    pub fn bounds(&self) -> i32 {
        (self.chunk_radius * CHUNK_SIZE) as i32
    }

    // helper function to convert index to xyz position
    fn index_to_position_ex(index: usize, chunk_radius: usize) -> IVec3 {
        let chunk = index_to_chunk_index(index);
        let offset = index_to_chunk_offset(index);
        let chunk_vector = utils::index_to_position(chunk, chunk_radius as i32);
        let offset_vector = Chunk::<Cell>::index_to_position(offset);

        (CHUNK_SIZE as i32 * chunk_vector) + offset_vector
    }

    // helper function to convert xyz position to index
    fn position_to_index_ex(vector: IVec3, chunk_radius: usize) -> usize {
        let chunk_vector = vector / CHUNK_SIZE as i32;
        let offset_vector = vector % CHUNK_SIZE as i32;
        let chunk = utils::position_to_index(chunk_vector, chunk_radius as i32);
        let offset = Chunk::<Cell>::position_to_index(offset_vector);

        chunk * CHUNK_CELL_COUNT + offset
    }

    // convert index to xyz position
    pub fn index_to_position(&self, index: usize) -> IVec3 {
        Chunks::<Cell>::index_to_position_ex(index, self.chunk_radius)
    }

    // convert xyz position to index
    pub fn position_to_index(&self, position: IVec3) -> usize {
        Chunks::<Cell>::position_to_index_ex(position, self.chunk_radius)
    }
}

impl<Cell: Default> Chunks<Cell> {
    // set bounds and update self
    pub fn set_bounds(&mut self, new_bounds: i32) -> i32 {
        let radius = (new_bounds as usize + CHUNK_SIZE - 1) / CHUNK_SIZE;

        if radius != self.chunk_radius {
            let count = radius * radius * radius;

            self.chunks.resize_with(count, || Chunk::default());
            self.chunk_radius = radius;
            self.chunk_count = count;
        }

        self.bounds()
    }
}

#[derive(Component)]
pub struct InstanceMaterialData(pub Vec<InstanceData>);

impl ExtractComponent for InstanceMaterialData {
    type Query = &'static InstanceMaterialData;
    type Filter = ();

    fn extract_component(item: QueryItem<Self::Query>) -> Self {
        InstanceMaterialData(item.0.clone())
    }
}

pub struct CellMaterialPlugin;

impl Plugin for CellMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ExtractComponentPlugin::<InstanceMaterialData>::default());
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<CellPipeline>()
            .init_resource::<SpecializedMeshPipelines<CellPipeline>>()
            .add_system_to_stage(RenderStage::Queue, queue_custom)
            .add_system_to_stage(RenderStage::Prepare, prepare_instance_buffers);
    }
}

/* the remainder of the code in this file is WebGPU setup */

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct InstanceData {
    pub position: Vec3,
    pub scale: f32,
    pub colour: [f32; 4]
}

#[allow(clippy::too_many_arguments)]
fn queue_custom(transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>, custom_pipeline: Res<CellPipeline>,
        msaa: Res<Msaa>, mut pipelines: ResMut<SpecializedMeshPipelines<CellPipeline>>, mut pipeline_cache: ResMut<RenderPipelineCache>,
        meshes: Res<RenderAssets<Mesh>>, material_meshes: Query<(Entity, &MeshUniform, &Handle<Mesh>), (With<Handle<Mesh>>, With<InstanceMaterialData>)>,
        mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent3d>)>) {
    let draw_custom = transparent_3d_draw_functions.read().get_id::<DrawCustom>().unwrap();

    let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples);

    for (view, mut transparent_phase) in views.iter_mut() {
        let view_matrix = view.transform.compute_matrix();
        let view_row_2 = view_matrix.row(2);

        for (entity, mesh_uniform, mesh_handle) in material_meshes.iter() {
            if let Some(mesh) = meshes.get(mesh_handle) {
                let key = msaa_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology);
                let pipeline = pipelines.specialize(&mut pipeline_cache, &custom_pipeline, key, &mesh.layout).unwrap();

                transparent_phase.add(Transparent3d {
                    entity,
                    pipeline,
                    draw_function: draw_custom,
                    distance: view_row_2.dot(mesh_uniform.transform.col(3))
                });
            }
        }
    }
}

#[derive(Component)]
pub struct InstanceBuffer {
    buffer: Buffer,
    length: usize
}

fn prepare_instance_buffers(mut commands: Commands, query: Query<(Entity, &InstanceMaterialData)>, render_device: Res<RenderDevice>) {
    for (entity, instance_data) in query.iter() {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: bytemuck::cast_slice(instance_data.0.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST
        });

        commands.entity(entity).insert(InstanceBuffer {
            buffer,
            length: instance_data.0.len()
        });
    }
}

pub struct CellPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline
}

impl FromWorld for CellPipeline {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        asset_server.watch_for_changes().unwrap();
        let shader = asset_server.load("shaders/cell.wgsl");
        let mesh_pipeline = world.get_resource::<MeshPipeline>().unwrap();

        CellPipeline {
            shader,
            mesh_pipeline: mesh_pipeline.clone()
        }
    }
}

impl SpecializedMeshPipeline for CellPipeline {
    type Key = MeshPipelineKey;

    fn specialize(&self, key: Self::Key, layout: &MeshVertexBufferLayout) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;

        descriptor.vertex.shader = self.shader.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3 // locations 0-2 are taken up by position, normal and uv attributes
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: VertexFormat::Float32x4.size(),
                    shader_location: 4
                }
            ]
        });

        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        descriptor.layout = Some(vec![
            self.mesh_pipeline.view_layout.clone(),
            self.mesh_pipeline.mesh_layout.clone()
        ]);

        Ok(descriptor)
    }
}

type DrawCustom = (SetItemPipeline, SetMeshViewBindGroup<0>, SetMeshBindGroup<1>, DrawMeshInstanced);

pub struct DrawMeshInstanced;

impl EntityRenderCommand for DrawMeshInstanced {
    type Param = (SRes<RenderAssets<Mesh>>, SQuery<Read<Handle<Mesh>>>, SQuery<Read<InstanceBuffer>>);

    #[inline]
    fn render<'w>(_view: Entity, item: Entity, (meshes, mesh_query, instance_buffer_query):SystemParamItem<'w, '_, Self::Param>,
            pass: &mut TrackedRenderPass<'w>) -> RenderCommandResult {
        let mesh_handle = mesh_query.get(item).unwrap();
        let instance_buffer = instance_buffer_query.get(item).unwrap();
        let gpu_mesh = match meshes.into_inner().get(mesh_handle) {
            Some(gpu_mesh) => gpu_mesh,
            None => return RenderCommandResult::Failure
        };

        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

        match &gpu_mesh.buffer_info {
            GpuBufferInfo::Indexed {
                buffer,
                index_format,
                count
            } => {
                pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                pass.draw_indexed(0..*count, 0, 0..instance_buffer.length as u32);
            }

            GpuBufferInfo::NonIndexed {vertex_count} => {
                pass.draw(0..*vertex_count, 0..instance_buffer.length as u32);
            }
        }

        RenderCommandResult::Success
    }
}
