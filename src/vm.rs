
use std::rc::Rc;
use crate::data::{ CompiledProc, VmError, StackTrace };

struct Frame {
    id: usize,
    ip: usize,
    locals: Vec<usize>,
}

pub struct Vm {
    memory: Vec<u8>,
    frames: Vec<Frame>, 
    procs: Vec<CompiledProc>,
    current: Frame,
}

impl Vm {
    pub fn new(procs : Vec<CompiledProc>) -> Vm {
        Vm { 
            procs, 
            current: Frame { id: 0, ip: 0, locals: vec![] }, 
            frames: vec![], 
            memory: vec![] 
        }
    }

    pub fn run(&mut self, entry : usize) -> Result<usize, String> {
        Ok(0)
    }

    fn stack_trace(&self) -> StackTrace {
        struct RetAddr { proc: usize, instr : usize }

        let mut stack = self.frames.iter().map(|x| RetAddr { proc: x.id, instr: x.ip }).collect::<Vec<_>>();
        stack.push(RetAddr { proc: self.current.id, instr: self.current.ip + 1});

        let mut trace = vec![];
        for addr in stack {
            // Note:  if the procedure was already pushed into the stack, then
            // that means that it already resolved to a known procedure. Don't
            // have to check again that the proc map has it.
            let name = Rc::clone(&self.procs[addr.proc].name);
            trace.push((name, addr.instr - 1));
        }
        trace
    }
}
