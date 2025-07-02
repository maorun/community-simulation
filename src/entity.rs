use serde::{Deserialize, Serialize};

pub type EntityId = usize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
        pub state: EntityState,
            pub active: bool,
            }

            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub struct EntityState {
                pub position: Vector3,
                    pub velocity: Vector3,
                        pub mass: f64,
                            pub energy: f64,
                            }

                            #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
                            pub struct Vector3 {
                                pub x: f64,
                                    pub y: f64,
                                        pub z: f64,
                                        }

                                        impl Vector3 {
                                            pub fn new(x: f64, y: f64, z: f64) -> Self {
                                                    Self { x, y, z }
                                                        }
                                                            
                                                                pub fn zero() -> Self {
                                                                        Self::new(0.0, 0.0, 0.0)
                                                                            }
                                                                                
                                                                                    pub fn magnitude(&self) -> f64 {
                                                                                            (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
                                                                                                }
                                                                                                    
                                                                                                        pub fn normalize(&self) -> Self {
                                                                                                                let mag = self.magnitude();
                                                                                                                        if mag > 0.0 {
                                                                                                                                    Self::new(self.x / mag, self.y / mag, self.z / mag)
                                                                                                                                            } else {
                                                                                                                                                        Self::zero()
                                                                                                                                                                }
                                                                                                                                                                    }
                                                                                                                                                                    }

                                                                                                                                                                    impl std::ops::Add for Vector3 {
                                                                                                                                                                        type Output = Self;
                                                                                                                                                                            
                                                                                                                                                                                fn add(self, other: Self) -> Self {
                                                                                                                                                                                        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
                                                                                                                                                                                            }
                                                                                                                                                                                            }

                                                                                                                                                                                            impl std::ops::Mul<f64> for Vector3 {
                                                                                                                                                                                                type Output = Self;
                                                                                                                                                                                                    
                                                                                                                                                                                                        fn mul(self, scalar: f64) -> Self {
                                                                                                                                                                                                                Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
                                                                                                                                                                                                                    }
                                                                                                                                                                                                                    }

                                                                                                                                                                                                                    impl Entity {
                                                                                                                                                                                                                        pub fn new(id: EntityId) -> Self {
                                                                                                                                                                                                                                Self {
                                                                                                                                                                                                                                            id,
                                                                                                                                                                                                                                                        state: EntityState {
                                                                                                                                                                                                                                                                        position: Vector3::zero(),
                                                                                                                                                                                                                                                                                        velocity: Vector3::zero(),
                                                                                                                                                                                                                                                                                                        mass: 1.0,
                                                                                                                                                                                                                                                                                                                        energy: 100.0,
                                                                                                                                                                                                                                                                                                                                    },
                                                                                                                                                                                                                                                                                                                                                active: true,
                                                                                                                                                                                                                                                                                                                                                        }
                                                                                                                                                                                                                                                                                                                                                            }
                                                                                                                                                                                                                                                                                                                                                                
                                                                                                                                                                                                                                                                                                                                                                    pub fn update(&mut self, dt: f64, forces: Vector3) {
                                                                                                                                                                                                                                                                                                                                                                            if !self.active {
                                                                                                                                                                                                                                                                                                                                                                                        return;
                                                                                                                                                                                                                                                                                                                                                                                                }
                                                                                                                                                                                                                                                                                                                                                                                                        
                                                                                                                                                                                                                                                                                                                                                                                                                let acceleration = forces * (1.0 / self.state.mass);
                                                                                                                                                                                                                                                                                                                                                                                                                        self.state.velocity = self.state.velocity + acceleration * dt;
                                                                                                                                                                                                                                                                                                                                                                                                                                self.state.position = self.state.position + self.state.velocity * dt;
                                                                                                                                                                                                                                                                                                                                                                                                                                        
                                                                                                                                                                                                                                                                                                                                                                                                                                                self.state.energy -= 0.01 * dt;
                                                                                                                                                                                                                                                                                                                                                                                                                                                        if self.state.energy <= 0.0 {
                                                                                                                                                                                                                                                                                                                                                                                                                                                                    self.active = false;
                                                                                                                                                                                                                                                                                                                                                                                                                                                                            }
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                }
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                }