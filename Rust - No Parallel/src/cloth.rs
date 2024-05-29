use rand::Rng;

#[derive(Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub ax: f32,
    pub ay: f32,
    pub fixed: bool,
    pub ext_m: f32,
}

#[derive(Clone, Copy)]
pub struct Spring {
    pub p1: (usize, usize),
    pub p2: (usize, usize),
    pub rest_length: f32,
    pub spring_coeff: f32,
    pub damp_coeff: f32,
}

pub struct Cloth {
    pub points: Vec<Vec<Point>>,
    pub springs: Vec<Spring>,
    pub g: f32,
    pub m: f32,
    pub g_on: bool,
}

impl Cloth {
    pub fn new(n: usize, m: usize) -> Self {
        let mut points =
            vec![vec![Point {x:0.0, y:0.0, vx:0.0, vy:0.0, ax:0.0, ay:0.0, fixed: false, ext_m:0.0}; m]; n];
        for i in 0..n {
            for j in 0..m {
                points[i][j] = Point {
                    x: j as f32,
                    y: i as f32,
                    vx: 0.0,
                    vy: 0.0,
                    ax: 0.0,
                    ay: 0.0,
                    fixed: false,
                    ext_m: 0.0,
                };
            }
        }

        // make the top left and top right points fixed
        points[n - 1][0].fixed = true;
        points[n - 1][m - 1].fixed = true;

        let mut springs = Vec::new();
        for i in 0..n {
            for j in 0..m {
                if i < n - 1 {
                    springs.push(Spring {
                        p1: (i, j),
                        p2: (i + 1, j),
                        spring_coeff: 10.0,
                        damp_coeff: 0.03,
                        rest_length: 1.0,
                    });
                }

                if j < m - 1 {
                    springs.push(Spring {
                        p1: (i, j),
                        p2: (i, j + 1),
                        spring_coeff: 10.0,
                        damp_coeff: 0.03,
                        rest_length: 1.0,
                    });
                }
            }
        }

        Cloth {
            points,
            springs,
            g: 9.81,
            m: 0.01,
            g_on: true,
        }
    }
    pub fn simulate(&mut self, dt: f32) {
        let mut forces = vec![vec![(0.0, 0.0); self.points[0].len()]; self.points.len()];

        for (i, row) in self.points.iter().enumerate() {
            for (j, point) in row.iter().enumerate() {
                let mut total_force_x = 0.0;
                let mut total_force_y = 0.0;

                //spring and damper
                for spring in &self.springs {
                    let point1 = &self.points[spring.p1.0][spring.p1.1];
                    let point2 = &self.points[spring.p2.0][spring.p2.1];

                    let dx = point2.x - point1.x;
                    let dy = point2.y - point1.y;

                    let distance = (dx * dx + dy * dy).sqrt();
                    let magnitude = spring.spring_coeff * (distance - spring.rest_length);

                    let spring_force_x = (magnitude * dx) / distance;
                    let spring_force_y = (magnitude * dy) / distance;

                    let damping_force_x = -point.vx * spring.damp_coeff;
                    let damping_force_y = -point.vy * spring.damp_coeff;

                    if point1.x == point.x && point1.y == point.y {
                        total_force_x += spring_force_x + damping_force_x;
                        total_force_y += spring_force_y + damping_force_y;
                    } else if point2.x == point.x && point2.y == point.y {
                        total_force_x -= spring_force_x - damping_force_x;
                        total_force_y -= spring_force_y - damping_force_y;
                    }
                }

                //gravity
                let gravity_force_x = 0.0;
                let gravity_force_y = if self.g_on { -self.g * self.m } else { 0.0 };

                //external forces
                let mut rng = rand::thread_rng();
                let ext_force_x = rng.gen_range(-1.0..1.0) * point.ext_m;
                let ext_force_y = rng.gen_range(-1.0..1.0) * point.ext_m;

                //total
                total_force_x += gravity_force_x + ext_force_x;
                total_force_y += gravity_force_y + ext_force_y;

                forces[i][j] = (total_force_x, total_force_y);
            }
        }

        for (i, row) in self.points.iter_mut().enumerate() {
            for (j, point) in row.iter_mut().enumerate() {
                if point.fixed {
                    continue;
                }

                let (fx, fy) = forces[i][j];

                //accelaration
                point.ax = fx / self.m;
                point.ay = fy / self.m;

                let prev_x = point.x;
                let prev_y = point.y;
                point.x += point.vx * dt + 0.5 * point.ax * dt * dt;
                point.y += point.vy * dt + 0.5 * point.ay * dt * dt;

                //floor colision
                if point.y < -32.0 {
                    point.y = -32.0;
                    point.vy = 0.0;
                }

                //velocity
                let new_vx = (point.x - prev_x) / dt;
                let new_vy = (point.y - prev_y) / dt;
                point.vx = if point.y == -32.0 { -new_vy } else { new_vx };
                point.vy = if point.y == -32.0 { -new_vy } else { new_vy };
            }
        }
    }
}
