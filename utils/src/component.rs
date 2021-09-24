use wgpu::{Device, Queue, SwapChainDescriptor, VertexBufferLayout};

pub struct Necessary<'a>{
    device:&'a Device,
    queue:&'a Queue,
    swap_chain_desc:&'a SwapChainDescriptor
}

pub trait Component{


    fn priority(&self) -> i32 { 0 }
    fn init(&mut self,device:Necessary<'_>);
    fn on_ret_vertex_buf_layout(&self) -> Vec<VertexBufferLayout>;

}