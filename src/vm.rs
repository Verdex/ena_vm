
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

    pub fn run(&mut self, entry : usize) -> Result<usize, VmError> {
        if entry >= self.procs.len() {
            return Err(VmError::UnknownProcId(entry, self.stack_trace()));
        }

        self.current.id = entry;
        self.current.locals = std::iter::repeat(0).take(self.procs[entry].frame_size).collect();

        let mut ret : Option<usize> = None;
        loop {
            if self.current.ip >= self.procs[self.current.id].instrs.len() {
                return Err(VmError::InstrPointerOutOfRange(self.current.ip, self.stack_trace()));
            }

            match self.procs[self.current.id].instrs[self.current.ip] {
                Op::FAdd(ID, ID, ID) => { },
                Op::FSub(ID, ID, ID) => { },
                Op::FMul(ID, ID, ID) => { },
                Op::FDiv(ID, ID, ID) => { },
                Op::FExp(ID, ID, ID) => { },
                Op::FNeg(ID, ID) => { },

                Op::FEq(ID, ID, ID) => { },
                Op::FGt(ID, ID, ID) => { },
                Op::FLt(ID, ID, ID) => { },

                Op::IAdd(ID, ID, ID) => { },
                Op::ISub(ID, ID, ID) => { },
                Op::IMul(ID, ID, ID) => { },
                Op::IDiv(ID, ID, ID) => { },
                Op::IMod(ID, ID, ID) => { },
                Op::IExp(ID, ID, ID) => { },
                Op::INeg(ID, ID) => { },

                Op::IEq(ID, ID, ID) => { },
                Op::IGt(ID, ID, ID) => { },
                Op::ILt(ID, ID, ID) => { },

                _ => todo!(),
            }
        }
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
