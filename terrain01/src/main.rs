use bevy::{
    pbr::NotShadowCaster,
    prelude::*,
    render::camera::ScalingMode,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

use noise::{NoiseFn, Perlin};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "terrain01".into(),
                    ..default()
                }),
                ..default()
            }),
        )
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Terrain::new(512))
        .add_systems(Startup, setup)
        .add_systems(Update, (camera, keyboard))
        .run();
}

pub fn setup(
    mut terrain: ResMut<Terrain>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    terrain.generate();

    commands.spawn((
        Camera3dBundle {
            projection: OrthographicProjection {
                scale: 3.0,
                scaling_mode: ScalingMode::WindowSize(255.0),
                ..default()
            }
            .into(),
            transform: Transform::from_xyz(-64.0, 64.0, -64.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        MainCamera,
    ));

    commands.spawn((DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 32000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 64.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    },));

    let texture: Handle<Image> = asset_server.load("uv-test-map.png");

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(create_mesh(terrain.as_ref())),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(texture),
                ..default()
            }),
            ..default()
        },
        NotWireframe,
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(create_mesh_frame(terrain.as_ref())),
            material: materials.add(Color::BLACK.into()),
            ..default()
        },
        Wireframe,
        NotShadowCaster,
    ));
}

/// Handle various keyboard shortcuts
pub fn keyboard(
    keyboard_keys: Res<Input<KeyCode>>,
    mut query_wireframe: Query<
        (&mut Visibility, &Wireframe),
        (With<Wireframe>, Without<NotWireframe>),
    >,
    mut query_notwireframe: Query<
        (&mut Visibility, &NotWireframe),
        (With<NotWireframe>, Without<Wireframe>),
    >,
) {
    let (mut wireframe, _) = query_wireframe.single_mut();
    let (mut notwireframe, _) = query_notwireframe.single_mut();

    if keyboard_keys.just_pressed(KeyCode::T) {
        *wireframe = match *wireframe {
            Visibility::Visible => Visibility::Hidden,
            Visibility::Hidden => Visibility::Visible,

            // This is the initial state, which is `Visible`.
            Visibility::Inherited => Visibility::Hidden,
        };
    }

    if keyboard_keys.just_pressed(KeyCode::Y) {
        *notwireframe = match *notwireframe {
            Visibility::Visible => Visibility::Hidden,
            Visibility::Hidden => Visibility::Visible,

            // This is the initial state, which is `Visible`.
            Visibility::Inherited => Visibility::Hidden,
        };
    }
}

pub fn camera(
    time: Res<Time>,
    keyboard_keys: Res<Input<KeyCode>>,
    mut query_camera: Query<(&mut Transform, &mut Projection), With<MainCamera>>,
) {
    let (mut transform, mut projection) = query_camera.single_mut();

    let scroll_speed = 32.0;

    let scroll_up = Vec3::new(1.0, 0.0, 1.0);
    let scroll_down = Vec3::new(-1.0, 0.0, -1.0);
    let scroll_left = Vec3::new(1.0, 0.0, -1.0);
    let scroll_right = Vec3::new(-1.0, 0.0, 1.0);

    // Keyboard scrolling
    if keyboard_keys.pressed(KeyCode::W) {
        transform.translation += scroll_up * scroll_speed * time.delta_seconds();
    }

    if keyboard_keys.pressed(KeyCode::S) {
        transform.translation += scroll_down * scroll_speed * time.delta_seconds();
    }

    if keyboard_keys.pressed(KeyCode::A) {
        transform.translation += scroll_left * scroll_speed * time.delta_seconds();
    }

    if keyboard_keys.pressed(KeyCode::D) {
        transform.translation += scroll_right * scroll_speed * time.delta_seconds();
    }

    let zoom_speed = 4.0;

    if let Projection::Orthographic(p) = projection.as_mut() {
        if keyboard_keys.pressed(KeyCode::Z) {
            p.scale += zoom_speed * time.delta_seconds();
        }

        if keyboard_keys.pressed(KeyCode::X) {
            p.scale -= zoom_speed * time.delta_seconds();
        }

        p.scale = p.scale.clamp(2.0, 40.0);
    }
}

fn create_mesh_frame(terrain: &Terrain) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::LineList);

    let side = terrain.side;
    let grid = side + 1;
    let size = grid * grid;

    let normals: Vec<[f32; 3]> = vec![[0.0f32, 1.0f32, 0.0f32]; size];
    let uvs: Vec<[f32; 2]> = vec![[0.0f32, 0.0f32]; size];

    let mut indices: Vec<u32> = Vec::new();

    for x in 0..side {
        for y in 0..side {
            indices.extend([
                (y * grid + x) as u32,
                (y * grid + x + 1) as u32,
                (y * grid + x) as u32,
                ((y + 1) * grid + x) as u32,
                (y * grid + x + 1) as u32,
                ((y + 1) * grid + x + 1) as u32,
                ((y + 1) * grid + x) as u32,
                ((y + 1) * grid + x + 1) as u32,
            ]);
        }
    }

    assert!(indices.len() / 2 == 4 * side * side);

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, terrain.vertices.to_vec());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(indices)));

    mesh
}

fn create_mesh(terrain: &Terrain) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let side = terrain.side;
    let grid = side + 1;
    let size = grid * grid;

    let mut normals: Vec<[f32; 3]> = vec![[0.0f32, 1.0f32, 0.0f32]; size];
    let mut uvs: Vec<[f32; 2]> = vec![[0.0f32, 0.0f32]; size];

    let mut indices: Vec<u32> = Vec::new();

    for x in 0..side {
        for y in 0..side {
            indices.extend([
                (y * grid + x + 1) as u32,
                ((y + 1) * grid + x + 1) as u32,
                (y * grid + x) as u32,
            ]);

            indices.extend([
                ((y + 1) * grid + x + 1) as u32,
                ((y + 1) * grid + x) as u32,
                (y * grid + x) as u32,
            ]);
        }
    }

    assert!(indices.len() / 3 == 2 * side * side);

    for i in (0..indices.len()).step_by(3) {
        let (vi0, vi1, vi2) = (
            indices[i] as usize,
            indices[i + 1] as usize,
            indices[i + 2] as usize,
        );

        let (v0, v1, v2) = (
            terrain.vertices[vi0],
            terrain.vertices[vi1],
            terrain.vertices[vi2],
        );

        let (v, w) = (
            Vec3::from(v2) - Vec3::from(v0),
            Vec3::from(v1) - Vec3::from(v0),
        );

        let c = w.cross(v);

        normals[vi0] = (Vec3::from(normals[vi0]) + c).into();
        normals[vi1] = (Vec3::from(normals[vi1]) + c).into();
        normals[vi2] = (Vec3::from(normals[vi2]) + c).into();
    }

    for i in 0..normals.len() {
        normals[i] = Vec3::from(normals[i]).normalize().into();
    }

    for i in 0..uvs.len() {
        uvs[i] = [terrain.vertices[i][1] / 16.0, 0.0];
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, terrain.vertices.to_vec());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(indices)));

    mesh
}

#[derive(Resource)]
pub struct Terrain {
    side: usize,
    grid: usize,
    dirty: bool,

    // general-data
    pub elevation: Vec<f32>,
    pub kind: Vec<f32>,

    // mesh-related
    pub vertices: Vec<[f32; 3]>,
}

impl Terrain {
    pub fn new(side: usize) -> Self {
        // to store x*y squares we store (x+1,y+1) vertices and we need to know
        // the height of each of them
        let grid = side + 1;

        Self {
            side,
            grid,
            dirty: true,
            elevation: vec![0.0; grid * grid],
            kind: vec![0.0; grid * grid],
            vertices: vec![[0.0f32, 0.0f32, 0.0f32]; grid * grid],
        }
    }

    pub fn elevation_at_xz(&self, x: f32, y: f32) -> Option<f32> {
        if x < 0.0 || y < 0.0 {
            return None;
        }

        if x > self.side as f32 || y > self.side as f32 {
            return None;
        }

        let (xp, yp) = (x.floor() as usize, y.floor() as usize);

        Some(self.elevation[xp + self.grid * yp])
    }

    fn resize(&mut self, side: usize) {
        self.side = side;
        self.grid = side + 1;
        self.dirty = true;

        self.elevation.resize(self.grid * self.grid, 0.0);
        self.vertices.resize(self.grid * self.grid, [0.0, 0.0, 0.0]);

        self.dirty = true
    }

    pub fn generate(&mut self) {
        self.resize(self.side);

        let perlin = Perlin::new(1);

        let mut index = 0;
        for x in 0..self.grid {
            for y in 0..self.grid {
                let h = perlin.get([x as f64 / 32.0, y as f64 / 32.0, 1.8]) as f32;
                self.elevation[index] = h * 32.0;
                self.vertices[index] = [x as f32, h * 32.0, y as f32];
                index += 1;
            }
        }

        self.dirty = false
    }
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct Wireframe;

#[derive(Component)]
pub struct NotWireframe;
