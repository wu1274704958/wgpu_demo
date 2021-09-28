use wgpu::{Device, Queue, SwapChainDescriptor, VertexBufferLayout};
use std::any::{TypeId, Any};
use crate::object::Object;

pub struct InitNecessary<'a>{
    device:&'a Device,
    queue:&'a Queue,
    swap_chain_desc:&'a SwapChainDescriptor
}
pub struct RenderNecessary<'a>{
    swap_chain_desc:&'a SwapChainDescriptor
}
pub struct UpdateNecessary<'a>{
    device:&'a Device,
    queue:&'a Queue,
    swap_chain_desc:&'a SwapChainDescriptor
}

pub trait Component{

    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
    fn type_id(&self) -> TypeId { self.as_any().type_id() }

    fn on_reg(&mut self,obj:*const Object);
    fn on_unreg(&mut self);
    fn on_add(&mut self);
    fn on_remove(&mut self);

    fn object(&self) -> &Object;
    fn mut_object(&self) -> &mut Object;

    fn priority(&self) -> i32 { 0 }
    fn init(&mut self,nec:InitNecessary<'_>);
    fn render(&mut self,nec:RenderNecessary<'_>);
    fn start(&mut self);
    fn update(&mut self,delta:f32);
    fn destroy(&mut self);

}