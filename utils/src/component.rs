use wgpu::{Device, Queue};

pub trait Component{
    fn priority() -> i32 { 0 }
    fn init(&mut self,device:&Device,queue:&Queue);
}