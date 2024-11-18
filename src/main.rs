use kiss3d::{
    camera::{FirstPerson, Camera},
    light::Light,
    nalgebra::{Point3, Translation3, UnitQuaternion, Vector3, Matrix4},
    window::Window,
    scene::SceneNode,
    event::{Action, Key, WindowEvent},
};
use std::f32::consts::PI;
use std::rc::Rc;

const WARP_POINTS: [(f32, f32, f32); 6] = [
    (0.0, 0.0, 0.0),      // Sol
    (3.0, 0.0, 3.0),      // Mercurio
    (5.0, 0.0, 5.0),      // Venus
    (7.0, 0.0, 7.0),      // Tierra
    (9.0, 0.0, 9.0),      // Marte
    (12.0, 0.0, 12.0),    // Júpiter
];

struct Planeta {
    radio: f32,
    distancia_sol: f32,
    velocidad_orbital: f32,
    color: Point3<f32>,
    angulo: f32,
    nodo: SceneNode,
    orbita: Vec<SceneNode>,
}

struct Nave {
    nodo: SceneNode,
    velocidad: Vector3<f32>,
}

struct Estado {
    warping: bool,
    warp_inicio: Point3<f32>,
    warp_destino: Point3<f32>,
    warp_progreso: f32,
}

impl Planeta {
    fn new(
        window: &mut Window,
        radio: f32,
        distancia_sol: f32,
        velocidad_orbital: f32,
        color: Point3<f32>,
    ) -> Self {
        let mut nodo = window.add_sphere(radio);
        nodo.set_color(color.x, color.y, color.z);
        
        // Crear puntos de la órbita
        let mut orbita = Vec::new();
        let segments = 100;
        for i in 0..segments {
            let angulo = (i as f32) * 2.0 * PI / (segments as f32);
            let mut punto = window.add_sphere(0.05);
            punto.set_color(0.5, 0.5, 0.5);
            punto.set_local_translation(Translation3::new(
                distancia_sol * angulo.cos(),
                0.0,
                distancia_sol * angulo.sin(),
            ));
            orbita.push(punto);
        }

        Planeta {
            radio,
            distancia_sol,
            velocidad_orbital,
            color,
            angulo: 0.0,
            nodo,
            orbita,
        }
    }

    fn actualizar(&mut self) {
        self.angulo += self.velocidad_orbital;
        if self.angulo >= 2.0 * PI {
            self.angulo = 0.0;
        }
        let pos = self.obtener_posicion();
        self.nodo.set_local_translation(Translation3::new(pos.x, pos.y, pos.z));
    }

    fn obtener_posicion(&self) -> Point3<f32> {
        Point3::new(
            self.distancia_sol * self.angulo.cos(),
            0.0,
            self.distancia_sol * self.angulo.sin(),
        )
    }
}

fn crear_skybox(window: &mut Window) {
    let size = 1000.0;
    let mut skybox = window.add_cube(size, size, size);
    
    // Textura de estrellas (representada con color por simplicidad)
    skybox.set_color(0.0, 0.0, 0.1);
    skybox.set_points_size(2.0);
    
    // Agregar "estrellas" como puntos blancos
    for _ in 0..1000 {
        let mut estrella = window.add_sphere(0.1);
        let x = rand::random::<f32>() * size - size/2.0;
        let y = rand::random::<f32>() * size - size/2.0;
        let z = rand::random::<f32>() * size - size/2.0;
        estrella.set_local_translation(Translation3::new(x, y, z));
        estrella.set_color(1.0, 1.0, 1.0);
    }
}

fn crear_nave(window: &mut Window) -> Nave {
    let mut nodo = window.add_sphere(0.5); // Base de la nave
    nodo.set_color(0.7, 0.7, 0.7);
    
    // Agregar detalles a la nave como objetos separados
    let mut ala_izq = window.add_cube(0.5, 0.1, 0.25);
    ala_izq.set_color(0.6, 0.6, 0.6);
    ala_izq.set_local_translation(Translation3::new(-0.5, 0.0, 0.0));
    
    let mut ala_der = window.add_cube(0.5, 0.1, 0.25);
    ala_der.set_color(0.6, 0.6, 0.6);
    ala_der.set_local_translation(Translation3::new(0.5, 0.0, 0.0));
    
    Nave {
        nodo,
        velocidad: Vector3::new(0.0, 0.0, 0.0),
    }
}

fn main() {
    let mut window = Window::new("Sistema Solar");
    window.set_light(Light::StickToCamera);
    
    let mut estado = Estado {
        warping: false,
        warp_inicio: Point3::origin(),
        warp_destino: Point3::origin(),
        warp_progreso: 0.0,
    };

    crear_skybox(&mut window);

    // Crear el sol
    let mut sol = window.add_sphere(2.0);
    sol.set_color(1.0, 0.7, 0.0);

    // Crear planetas
    let mut planetas = vec![
        Planeta::new(&mut window, 0.4, 3.0, 0.03, Point3::new(0.7, 0.7, 0.7)),
        Planeta::new(&mut window, 0.9, 5.0, 0.02, Point3::new(0.9, 0.7, 0.5)),
        Planeta::new(&mut window, 1.0, 7.0, 0.015, Point3::new(0.2, 0.5, 1.0)),
        Planeta::new(&mut window, 0.8, 9.0, 0.012, Point3::new(1.0, 0.4, 0.2)),
        Planeta::new(&mut window, 1.8, 12.0, 0.008, Point3::new(0.8, 0.6, 0.4)),
    ];

    let mut nave = crear_nave(&mut window);

    // Configurar cámara
    let eye = Point3::new(0.0, 10.0, -20.0);
    let at = Point3::origin();
    let mut primera_persona = FirstPerson::new(eye, at);
    primera_persona.set_move_step(0.5);

    let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);

    let mut last_pos = eye;

    while window.render_with_camera(&mut primera_persona) {
        sol.append_rotation(&rot);

        for planeta in planetas.iter_mut() {
            planeta.actualizar();
        }

        // Obtener la posición actual de la cámara
        let pos_camara = primera_persona.eye();
        
        // Verificar colisiones y actualizar posición si es necesario
        for planeta in &planetas {
            let pos_planeta = planeta.obtener_posicion();
            let distancia = (pos_camara - pos_planeta).norm();
            if distancia < planeta.radio + 1.0 {
                // En lugar de set_eye, mantenemos la última posición válida
                primera_persona = FirstPerson::new(last_pos, pos_planeta);
                primera_persona.set_move_step(0.5);
                break;
            }
        }

        // Actualizar última posición válida
        last_pos = pos_camara;

        // Actualizar posición de la nave
        nave.nodo.set_local_translation(Translation3::new(
            pos_camara.x,
            pos_camara.y - 2.0,
            pos_camara.z + 3.0,
        ));

        // Manejo de warping
        if window.get_key(Key::Space) == Action::Press && !estado.warping {
            let pos_actual = primera_persona.eye();
            let mut min_dist = f32::MAX;
            let mut punto_destino = WARP_POINTS[0];
            
            for punto in WARP_POINTS.iter() {
                let dist = ((punto.0 - pos_actual.x).powi(2) + 
                          (punto.1 - pos_actual.y).powi(2) + 
                          (punto.2 - pos_actual.z).powi(2)).sqrt();
                if dist < min_dist {
                    min_dist = dist;
                    punto_destino = *punto;
                }
            }

            estado.warping = true;
            estado.warp_inicio = pos_actual;
            estado.warp_destino = Point3::new(punto_destino.0, punto_destino.1, punto_destino.2);
            estado.warp_progreso = 0.0;
        }

        if estado.warping {
            estado.warp_progreso += 0.02;
            if estado.warp_progreso >= 1.0 {
                estado.warping = false;
                // Crear nueva cámara en la posición de destino
                primera_persona = FirstPerson::new(estado.warp_destino, Point3::origin());
                primera_persona.set_move_step(0.5);
            }
        }
    }
}