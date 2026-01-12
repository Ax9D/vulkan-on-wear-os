use std::time::Duration;
use std::time::Instant;

use android_activity::{AndroidApp, InputStatus, MainEvent, PollEvent};
use hikari_render::*;
use winit::application::ApplicationHandler;
use winit::event::Event;
use winit::event_loop::ControlFlow;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::EventLoopBuilder;
use winit::window::Window;

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Trace));
    use winit::platform::android::EventLoopBuilderExtAndroid;

    let event_loop = EventLoop::builder()
        .with_android_app(app)
        .build().expect("Failed to create event loop");

    let mut my_app = MyApp {
        graphics: None,
        window: None,
    };

    event_loop.run_app(&mut my_app).expect("Failed to run app");
}

struct GraphicsState {
    gfx: Gfx,
    graph: Graph<(f32, )>,
    time: f32,
    last_update: Instant,
}
impl GraphicsState {
    pub fn new(window: &Window) -> Self {
        let mut gfx = Gfx::new(&window, GfxConfig::default())
                .expect("Failed to initialize Gfx");
                let vs = r"
        #version 450
        layout( push_constant ) uniform constants
        {
            mat4 rotation;
        } pc;
        layout(location = 0) out vec3 vs_color;
        const vec2 vertices[3] = vec2[](
                vec2( 0.0,  0.5),  // top
                vec2(-0.5, -0.5),  // bottom-left
                vec2( 0.5, -0.5)   // bottom-right
            );

        const vec3 colors[3] = vec3[](
                vec3(1.0, 0.0, 0.0),
                vec3(0.0, 1.0, 0.0),
                vec3(0.0, 0.0, 1.0)
            );

        void main() {
        vec4 vertex = vec4(vertices[gl_VertexIndex], 1.0, 1.0);

        gl_Position = pc.rotation * vertex;
        vs_color = colors[gl_VertexIndex];
        }
        ";

        let fs = r"
        #version 450
        layout(location = 0) in vec3 vs_color;
        layout(location = 0) out vec4 color;

        void main() {
            color = vec4(vs_color, 1.0);
        }
        ";

        let mandelbrot_vs = r"
        #version 450

layout(location = 0) out vec2 fs_uv;

// Standard fullscreen triangle
const vec2 vertices[3] = vec2[](
    vec2(-1.0, -1.0),
    vec2( 3.0, -1.0),
    vec2(-1.0,  3.0)
);

void main() {
    vec2 pos = vertices[gl_VertexIndex];
    gl_Position = vec4(pos, 0.0, 1.0);

    fs_uv = pos;   // in [-1, 3], but covers screen correctly
}";
        let mandelbrot_fs = r"#version 450

layout(push_constant) uniform constants {
    float time;
    float aspect;
} pc;

layout(location = 0) in vec2 fs_uv;
layout(location = 0) out vec4 outColor;

// Map clip-space [-1..1] to complex plane
vec2 mapToComplex(vec2 uv, float zoom, vec2 center) {
    return vec2(
        uv.x * pc.aspect * zoom + center.x,
        uv.y * zoom + center.y
    );
}

void main() {
    //----------------------------------------------------------
    // Camera
    //----------------------------------------------------------
    float zoom = exp(-0.12 * pc.time);

    vec2 center = vec2(
        -0.745 + 0.015 * sin(pc.time * 0.3),
         0.110 + 0.015 * cos(pc.time * 0.2)
    );

    vec2 c = mapToComplex(fs_uv, zoom, center);

    //----------------------------------------------------------
    // Mandelbrot iteration
    //----------------------------------------------------------
    int maxIter = 300;
    vec2 z = vec2(0.0);
    int i;

    for (i = 0; i < maxIter; i++) {
        if (dot(z, z) > 4.0) break;

        z = vec2(
            z.x * z.x - z.y * z.y + c.x,
            2.0 * z.x * z.y         + c.y
        );
    }

    //----------------------------------------------------------
    // Interior region
    //----------------------------------------------------------
    if (i == maxIter) {
        outColor = vec4(0.0, 0.0, 0.0, 1.0);
        return;
    }

    //----------------------------------------------------------
    // Smooth iteration count (correct GLSL syntax)
    //----------------------------------------------------------
    float zn = length(z);
    zn = max(zn, 1e-6); // avoid log(0)

    // smooth iteration value: i + 1 - log(log|z|)/log(2)
    float nu = log(log(zn)) / log(2.0);
    float iter = float(i) + 1.0 - nu;

    float t = iter / float(maxIter);
    t = clamp(t, 0.0, 1.0);

    //----------------------------------------------------------
    // Colour palette: blue→cyan→yellow→orange→red
    //----------------------------------------------------------
    vec3 c1 = vec3(0.0, 0.0, 0.4);
    vec3 c2 = vec3(0.0, 0.8, 1.0);
    vec3 c3 = vec3(1.0, 1.0, 0.0);
    vec3 c4 = vec3(1.0, 0.5, 0.0);
    vec3 c5 = vec3(0.8, 0.0, 0.0);

    vec3 col;

    if (t < 0.25)
        col = mix(c1, c2, t / 0.25);
    else if (t < 0.50)
        col = mix(c2, c3, (t - 0.25) / 0.25);
    else if (t < 0.75)
        col = mix(c3, c4, (t - 0.50) / 0.25);
    else
        col = mix(c4, c5, (t - 0.75) / 0.25);

    outColor = vec4(col, 1.0);
}";

        let shader = Shader::builder("triangle")
        .with_stage(ShaderStage::Vertex, ShaderCode { entry_point: "main", data: ShaderData::Glsl(vs.into()) }, &[])
        .with_stage(ShaderStage::Fragment, ShaderCode { entry_point: "main", data: ShaderData::Glsl(fs.into()) }, &[])
        .build(gfx.device(), None).unwrap();

        let shader = Shader::builder("mandelbrot")
        .with_stage(ShaderStage::Vertex, ShaderCode { entry_point: "main", data: ShaderData::Glsl(mandelbrot_vs.into()) }, &[])
        .with_stage(ShaderStage::Fragment, ShaderCode { entry_point: "main", data: ShaderData::Glsl(mandelbrot_fs.into()) }, &[])
        .build(gfx.device(), None).unwrap();

        let (width, height) = gfx.swapchain().unwrap().lock().size();

        let mut graph = GraphBuilder::new(&mut gfx, width, height);
            graph.add_renderpass(Renderpass::new("Triangle", ImageSize::default_xy()).present().cmd(move |cmd, _, info, (&time, ): (&f32, )| {
            cmd.set_shader(&shader);

            // let mut transform = hikari_math::Transform::default();
            // transform.rotation = hikari_math::Quat::from_rotation_z(time);
            // let rotation= transform.get_rotation_matrix().to_cols_array();
            // cmd.push_constants(&rotation, 0);
            // cmd.draw(0..3, 0..1);
            
            #[derive(Clone, Copy)]
            struct PC {
                time: f32, 
                aspect: f32,
            };

            let pc = PC {
                time, 
                aspect: info.framebuffer_width as f32 / info.framebuffer_height as f32
            };

            cmd.push_constants(&pc, 0);
            cmd.draw(0..3, 0..1);
        }));

        let graph = graph.build().unwrap();

        Self {
            gfx,
            graph,
            time: 0.0,
            last_update: Instant::now()
        }
    }
}
struct MyApp {
    graphics: Option<GraphicsState>,
    window: Option<Window>
}
impl ApplicationHandler for MyApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("App resumed — creating window");

        let window = event_loop.create_window(Window::default_attributes())
            .expect("Failed to create window");

        self.graphics = Some(
            GraphicsState::new(&window)
        );

        self.window = Some(window);
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        log::info!("App suspended — dropping gfx");
        self.graphics = None;
        self.window = None;
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        log::info!("Received window event: {:?}", event);

        let Some(window) = &self.window else { return; };
        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::Resized(size) => {
                if let Some(graphics) = &mut self.graphics {
                    graphics.graph.resize(size.width, size.height).expect("Failed to resize graph");
                    graphics.gfx.resize(size.width, size.height)
                        .expect("Failed to resize swapchain");
                }
                window.request_redraw();
            }

            WindowEvent::RedrawRequested => {
                if let Some(graphics) = &mut self.graphics {
                    let current_time = Instant::now();
                    let dt = (current_time - graphics.last_update).as_secs_f32();
                    const SPEED: f32 = 0.75;
                    graphics.time += SPEED * dt;
                    graphics.graph.execute_sync((&graphics.time, ));
                    graphics.gfx.new_frame().expect("Failed to render new frame");
                    graphics.last_update = current_time;
                }
                window.request_redraw();
            }

            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            _ => {}
        }
    }
}