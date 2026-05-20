
pub struct Vm {
    memory: Vec<u8>,
    frames: Vec<u8>, // everything is a reference? vec<usize>
}

impl Vm {
    fn run(&mut self, entry : usize) -> Result<> {

    }
}
