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
}

#[derive(Clone, Copy)]
pub struct Spring {
    pub p1: Point,
    pub p2: Point,
    pub rest_length: f32,
    pub spring_coeff: f32,
    pub damp_coeff: f32,
}

pub struct Cloth {
    pub points: Vec<Vec<Point>>,
    pub springs: Vec<Spring>,
    pub g: f32,
    pub m: f32,
    pub ext_m: f32,
}

impl Cloth {
    pub fn new(n: usize, m: usize) -> Self {
        let mut points =
            vec![vec![Point {x:0.0, y:0.0, vx:0.0, vy:0.0, ax:0.0, ay:0.0, fixed: false}; m]; n];
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
                        p1: points[i][j],
                        p2: points[i + 1][j],
                        spring_coeff: 10.0,
                        damp_coeff: 0.03,
                        rest_length: 1.0,
                    });
                }

                if j < m - 1 {
                    springs.push(Spring {
                        p1: points[i][j],
                        p2: points[i][j + 1],
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
            ext_m: 0.0,
        }
    }
    pub fn simulate(&mut self, dt: f32) {
        for row in &mut self.points {
            for point in row {
                let mut total_force_x = 0.0;
                let mut total_force_y = 0.0;
                //spring
                for spring in &self.springs {
                    if
                        (spring.p1.x == point.x && spring.p1.y == point.y) ||
                        (spring.p2.x == point.x && spring.p2.y == point.y)
                    {
                        let dx = spring.p2.x - spring.p1.x;
                        let dy = spring.p2.y - spring.p1.y;

                        let distance = (dx * dx + dy * dy).sqrt();
                        let magnitude = spring.spring_coeff * (distance - spring.rest_length);

                        let spring_force_x = (magnitude * dx) / distance;
                        let spring_force_y = (magnitude * dy) / distance;

                        // damping
                        let damping_force_x = -point.vx * spring.damp_coeff;
                        let damping_force_y = -point.vy * spring.damp_coeff;

                        total_force_x += spring_force_x + damping_force_x;
                        total_force_y += spring_force_y + damping_force_y;
                    }
                }

                //gravity
                let gravity_force_x = 0.0;
                let gravity_force_y = -self.g * self.m;

                //external force
                let mut rng = rand::thread_rng();
                let external_force_x = rng.gen_range(-1.0..1.0) * self.ext_m;
                let external_force_y = rng.gen_range(-1.0..1.0) * self.ext_m;

                //total force
                total_force_x += gravity_force_x + external_force_x;
                total_force_y += gravity_force_y + external_force_y;

                //acceleration
                point.ax = total_force_x / self.m;
                point.ay = total_force_y / self.m;
            }
        }
        for row in &mut self.points {
            for point in row {
                if point.fixed {
                    continue;
                }

                let prev_x = point.x;
                let prev_y = point.y;
                point.x += point.vx * dt + 0.5 * point.ax * dt * dt;
                point.y += point.vy * dt + 0.5 * point.ay * dt * dt;

                // Collision with floor
                if point.y + point.vy * dt < -16.0 {
                    point.y = -16.0;
                    point.vy = -point.vy;
                }

                point.vx = (point.x - prev_x) / dt;
                point.vy = (point.y - prev_y) / dt;
            }
        }
    }
}
