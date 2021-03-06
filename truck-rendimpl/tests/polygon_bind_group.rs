mod common;
use glsl_to_spirv::ShaderType;
use image::{DynamicImage, ImageBuffer, Rgba};
use std::sync::{Arc, Mutex};
use truck_platform::*;
use truck_rendimpl::*;
use wgpu::*;

const PICTURE_SIZE: (u32, u32) = (256, 256);

struct BGCheckPolygonInstance<'a> {
    polygon: PolygonInstance,
    fragment_shader: &'a str,
}

impl<'a> Rendered for BGCheckPolygonInstance<'a> {
    derive_render_id!(polygon);
    derive_vertex_buffer!(polygon);
    derive_bind_group_layout!(polygon);
    derive_bind_group!(polygon);
    #[inline(always)]
    fn pipeline(
        &self,
        device_handler: &DeviceHandler,
        layout: &PipelineLayout,
        sample_count: u32,
    ) -> Arc<RenderPipeline> {
        let vertex_shader = include_str!("shaders/mesh-bindgroup.vert");
        let vertex_spirv = common::compile_shader(vertex_shader, ShaderType::Vertex);
        let vertex_module = wgpu::util::make_spirv(&vertex_spirv);
        let fragment_spirv = common::compile_shader(self.fragment_shader, ShaderType::Fragment);
        let fragment_module = wgpu::util::make_spirv(&fragment_spirv);
        self.polygon.pipeline_with_shader(
            vertex_module,
            fragment_module,
            device_handler,
            layout,
            sample_count,
        )
    }
}

const ATTRS_OBJ: &str = "
v -1.0 2.0 -1.0\nv 1.0 2.0 -1.0\nv -1.0 2.0 1.0\nv 1.0 2.0 1.0
vt -1.0 -1.0\nvt 1.0 -1.0\nvt 1.0 1.0\nvt -1.0 1.0
vn -1.0 0.2 -1.0\nvn 1.0 0.2 -1.0\nvn -1.0 0.2 1.0\nvn 1.0 0.2 1.0
";
const TRIS_OBJ: &str = "f 1/1/1 2/2/3 3/4/2\nf 3/4/2 2/2/3 4/3/4\n";
const QUADS_OBJ: &str = "f 1/1/1 2/2/3 4/3/4 3/4/2\n";

fn test_polygons() -> [PolygonMesh; 2] {
    [
        obj::read((ATTRS_OBJ.to_string() + TRIS_OBJ).as_bytes()).unwrap(),
        obj::read((ATTRS_OBJ.to_string() + QUADS_OBJ).as_bytes()).unwrap(),
    ]
}

fn nontex_inst_desc() -> PolygonInstanceDescriptor {
    PolygonInstanceDescriptor {
        instance_state: InstanceState {
            matrix: Matrix4::from_cols(
                [1.0, 2.0, 3.0, 4.0].into(),
                [5.0, 6.0, 7.0, 8.0].into(),
                [9.0, 10.0, 11.0, 12.0].into(),
                [13.0, 14.0, 15.0, 16.0].into(),
            ),
            material: Material {
                albedo: Vector4::new(0.2, 0.4, 0.6, 1.0),
                roughness: 0.31415,
                reflectance: 0.29613,
                ambient_ratio: 0.92,
            },
            texture: None,
            backface_culling: true,
        },
    }
}

fn exec_polygon_bgtest(
    scene: &mut Scene,
    instance: &PolygonInstance,
    shader: &str,
    answer: &Vec<u8>,
    id: usize,
    out_dir: String,
) -> bool {
    let sc_desc = scene.sc_desc();
    let tex_desc = common::texture_descriptor(&sc_desc);
    let texture = scene.device().create_texture(&tex_desc);
    let mut bgc_instance = BGCheckPolygonInstance {
        polygon: instance.clone_instance(),
        fragment_shader: shader,
    };
    common::render_one(scene, &texture, &mut bgc_instance);
    let buffer = common::read_texture(scene.device_handler(), &texture);
    let path = format!("{}polygon-bgtest-{}.png", out_dir, id);
    common::save_buffer(path, &buffer, PICTURE_SIZE);
    common::same_buffer(&answer, &buffer)
}

fn exec_polymesh_nontex_bind_group_test(backend: BackendBit, out_dir: &str) {
    let out_dir = out_dir.to_string();
    std::fs::create_dir_all(&out_dir).unwrap();
    let instance = Instance::new(backend);
    let (device, queue) = common::init_device(&instance);
    let sc_desc = Arc::new(Mutex::new(common::swap_chain_descriptor(PICTURE_SIZE)));
    let handler = DeviceHandler::new(device, queue, sc_desc);
    let mut scene = Scene::new(handler, &Default::default());
    let creator = scene.instance_creator();
    let answer = common::nontex_answer_texture(&mut scene);
    let answer = common::read_texture(scene.device_handler(), &answer);
    let inst_desc = nontex_inst_desc();
    test_polygons()
        .iter()
        .enumerate()
        .for_each(move |(i, polygon)| {
            let instance: PolygonInstance = creator.create_instance(polygon, &inst_desc);
            let shader = include_str!("shaders/mesh-nontex-bindgroup.frag");
            assert!(exec_polygon_bgtest(
                &mut scene,
                &instance,
                shader,
                &answer,
                i,
                out_dir.clone()
            ));
            let shader = include_str!("shaders/anti-mesh-nontex-bindgroup.frag");
            assert!(!exec_polygon_bgtest(
                &mut scene,
                &instance,
                shader,
                &answer,
                i,
                out_dir.clone()
            ));
        })
}

#[test]
fn polymesh_nontex_bind_group_test() {
    common::os_alt_exec_test(exec_polymesh_nontex_bind_group_test)
}

fn exec_polymesh_tex_bind_group_test(backend: BackendBit, out_dir: &str) {
    let out_dir = out_dir.to_string();
    std::fs::create_dir_all(&out_dir).unwrap();
    let instance = Instance::new(backend);
    let (device, queue) = common::init_device(&instance);
    let sc_desc = Arc::new(Mutex::new(common::swap_chain_descriptor(PICTURE_SIZE)));
    let handler = DeviceHandler::new(device, queue, sc_desc);
    let mut scene = Scene::new(handler, &Default::default());
    let creator = scene.instance_creator();
    let answer = common::random_texture(&mut scene);
    let buffer = common::read_texture(scene.device_handler(), &answer);
    let pngpath = out_dir.clone() + "random-texture.png";
    common::save_buffer(pngpath, &buffer, PICTURE_SIZE);
    let mut desc = nontex_inst_desc();
    let image_buffer =
        ImageBuffer::<Rgba<_>, _>::from_raw(PICTURE_SIZE.0, PICTURE_SIZE.1, buffer.clone())
            .unwrap();
    let attach = image2texture::image2texture(
        scene.device_handler(),
        &DynamicImage::ImageRgba8(image_buffer),
    );
    desc.instance_state.texture = Some(Arc::new(attach));
    test_polygons()
        .iter()
        .enumerate()
        .for_each(move |(i, polygon)| {
            let instance: PolygonInstance = creator.create_instance(polygon, &desc);
            let shader = include_str!("shaders/mesh-tex-bindgroup.frag");
            assert!(exec_polygon_bgtest(
                &mut scene,
                &instance,
                shader,
                &buffer,
                i + 3,
                out_dir.clone(),
            ));
            let shader = include_str!("shaders/anti-mesh-tex-bindgroup.frag");
            assert!(!exec_polygon_bgtest(
                &mut scene,
                &instance,
                shader,
                &buffer,
                i + 3,
                out_dir.clone(),
            ));
        })
}

#[test]
fn polymesh_tex_bind_group_test() { common::os_alt_exec_test(exec_polymesh_tex_bind_group_test) }
