use std::{fs::{self, File}, io::Read, ops::Range};

use wgpu::util::DeviceExt;

pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

pub trait DrawModel<'a> {
    fn draw_mesh(&mut self, mesh: &'a Mesh);
    fn draw_mesh_instanced(&mut self, mesh: &'a Mesh, instances: Range<u32>);
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    // pub materials: Vec<Material>,
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    // pub material: usize,
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a> where 'b: 'a, {
    fn draw_mesh(&mut self, mesh: &'b Mesh) {
        self.draw_mesh_instanced(mesh, 0..1);
    }

    fn draw_mesh_instanced(
        &mut self, 
        mesh: &'b Mesh, 
        instances: Range<u32>
    ) {
        self.set_vertex_buffer(
            0, 
            mesh.vertex_buffer.slice(..)
        );
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }
}

fn f32_from_vec_offset(vec: &Vec<u8>, offset: usize) -> f32 {
    f32::from_le_bytes([
        vec[offset],
        vec[offset + 1],
        vec[offset + 2],
        vec[offset + 3]
    ])
}

const STL_HEADER_SIZE: u64 = 80;
pub async fn load_stl(
    file_path: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> Model {
    let mut file = File::open(file_path).unwrap();
    // Read the file header -> 80 bytes
    // Read the number of triangles -> u32

    let mut header_buffer = Vec::with_capacity(STL_HEADER_SIZE as usize);
    _ = file.by_ref()
        .take(STL_HEADER_SIZE)
        .read_to_end(&mut header_buffer).unwrap();

    // Get the number of iterations
    let mut size_buffer: Vec<u8> = Vec::with_capacity(std::mem::size_of::<u32>());
    _ = file.by_ref()
        .take(std::mem::size_of::<u32>() as u64)
        .read_to_end(&mut size_buffer).unwrap();

    let num_tris = u32::from_le_bytes([size_buffer[0], size_buffer[1], size_buffer[2], size_buffer[3]]);

    let mut verts: Vec<ModelVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut current_index: u32 = 0;
    let triangle_size: u64 = (12 * 4)+2;
    let f32_size = std::mem::size_of::<f32>();
    let color = [1.0, 1.0, 1.0];
    let tex_coords = [0.0, 0.0];
    for _ in 0..num_tris {
        let mut tri_buffer: Vec<u8> = Vec::with_capacity(triangle_size as usize);
        _ = file.by_ref()
            .take(triangle_size)
            .read_to_end(&mut tri_buffer);
        

        // TODO: Roll this into a new function
        let normal = [
            f32_from_vec_offset(&tri_buffer, 0),
            f32_from_vec_offset(&tri_buffer, f32_size),
            f32_from_vec_offset(&tri_buffer, 2 * f32_size)
        ];

        let pos1 = [
            f32_from_vec_offset(&tri_buffer, 3 * f32_size),
            f32_from_vec_offset(&tri_buffer, 4 * f32_size),
            f32_from_vec_offset(&tri_buffer, 5 * f32_size)
        ];

        let pos2 = [
            f32_from_vec_offset(&tri_buffer, 6 * f32_size),
            f32_from_vec_offset(&tri_buffer, 7 * f32_size),
            f32_from_vec_offset(&tri_buffer, 8 * f32_size)
        ];

        let pos3 = [
            f32_from_vec_offset(&tri_buffer, 9 * f32_size),
            f32_from_vec_offset(&tri_buffer, 10 * f32_size),
            f32_from_vec_offset(&tri_buffer, 11 * f32_size)
        ];

        let mut tri_verts = vec![
            ModelVertex {
                position: pos1,
                color,
                normal,
                tex_coords
            },
            ModelVertex {
                position: pos2,
                color,
                normal,
                tex_coords
            },
            ModelVertex {
                position: pos3,
                color,
                normal,
                tex_coords
            }
        ];
        
        verts.append(&mut tri_verts);

        let mut tri_indices = vec![current_index, current_index+1, current_index+2];
        indices.append(&mut tri_indices);
        current_index += 3;
    }

    // build all the mesh shenanagains
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{:?} Vertex Buffer", file_path)), // TODO: Use just the file name instead of the full path
        contents: bytemuck::cast_slice(&verts),
        usage: wgpu::BufferUsages::VERTEX
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{:?} Index Buffer", file_path)),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX
    });

    Model {
        meshes: vec![
            Mesh {
                name: format!("{:?}", file_path),
                vertex_buffer,
                index_buffer,
                num_elements: indices.len() as u32,
            }
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { // Position
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute { // Color
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ]
        }
    }
}