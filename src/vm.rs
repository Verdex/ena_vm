
use crate::data::CompiledProc;

pub struct Vm {
    memory: Vec<u8>,
    frames: Vec<usize>, 
    code: Vec<CompiledProc>,
}

impl Vm {
    pub fn new(code : Vec<CompiledProc>) -> Vm {
        Vm { code, frames: vec![], memory: vec![] }
    }

    pub fn run(&mut self, entry : usize) -> Result<usize, String> {
        Ok(0)
    }
}
