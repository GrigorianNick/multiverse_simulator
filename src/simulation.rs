use std::{default, iter, ops, vec};

use uuid::Uuid;
use physical_constants::{self, NEWTONIAN_CONSTANT_OF_GRAVITATION};

#[derive(Default, Debug, Clone, Copy)]
pub struct Pos
{
    pub x: f64,
    pub y: f64,
    pub z: f64
}

impl Pos {

    fn dist(&self, other: Pos) -> f64 {
        self.dist_sq(other).sqrt()
    }

    fn dist_sq(&self, other: Pos) -> f64 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2)
    }
}

impl ops::AddAssign<Pos> for Pos {
    fn add_assign(&mut self, rhs: Pos) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl ops::Mul<f64> for Pos {
    type Output = Pos;

    fn mul(self, rhs: f64) -> Self::Output {
        Pos{
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl ops::Sub<Pos> for Pos {
    type Output = Pos;

    fn sub(self, rhs: Pos) -> Self::Output {
        Pos{
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z
        }
    }
}

impl iter::Sum for Pos {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut total = Pos::default();
        for pos in iter {
            total.x += pos.x;
            total.y += pos.y;
            total.z += pos.z;
        }
        total
    }
}

#[derive(Default)]
pub struct Body
{
    pub id: uuid::Uuid,
    pub velocity: Pos,
    pub position: Pos,
    pub mass: f64,
}

impl Body {

    pub fn new() -> Body {
        let mut b = Body::default();
        b.id = Uuid::new_v4();
        b
    }

    pub fn get_force(&self, other: &Body) -> f64 {
        let d = self.position.dist_sq(other.position);
        if d == 0.0 {
            return  0.0;
        }
        let x = (NEWTONIAN_CONSTANT_OF_GRAVITATION * self.mass + other.mass) / d;
        x
    }

    pub fn get_pull(&self, other: &Body) -> Pos {
        (other.position - self.position) * self.get_force(other)
    }

    pub fn update_velocity(&mut self, other_bodies: Vec<&Body>) {
        //let other_bodies: Vec<&Body> = bodies.iter().filter(|&b| b.id != self.id).collect();
        let grav_sum: Pos = other_bodies.iter().map(|b| self.get_pull(b)).sum();
        self.velocity += grav_sum;
    }

    pub fn tick(&mut self) {
        self.position += self.velocity;
    }
}

pub struct Universe
{
    pub bodies: Vec<Body>,
}

impl Universe {
    pub fn new() -> Universe {
        Universe{
            bodies: vec![]
        }
    }

    pub fn add_body(&mut self, body: Body) {
        self.bodies.push(body);
    }

    pub fn tick(&mut self) {
        let mut grav_sums: Vec<Pos> = vec![];
        for body in &self.bodies {
            let grav_sum: Pos = self.bodies.iter().map(|b| body.get_pull(b)).sum();
            grav_sums.push(grav_sum);
        }

        let deltas = self.bodies.iter_mut().zip(grav_sums);
        for delta in deltas {
            delta.0.velocity += delta.1
        }
        for body in &mut self.bodies {
            body.tick();
        }
    }

    pub fn tick_for(&mut self, count: i32) {
        for _ in 1..=count {
            self.tick();
        }
    }
}