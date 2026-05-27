
use std::rc::Rc;
use crate::data::{ Op, CompiledProc, VmError, StackTrace };

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
        const FLEN : usize = 8;
        const ILEN : usize = 8;

        if entry >= self.procs.len() {
            return Err(VmError::UnknownProcId(entry, self.stack_trace()));
        }

        self.current.id = entry;
        self.current.locals = std::iter::repeat(0).take(self.procs[entry].frame_size).collect();

        let mut ret : Option<usize> = None;
        loop {
            if self.current.ip >= self.procs[self.current.id].instrs.len() {
                // TODO: with the right construction of compiled proc this might not have to be
                // something that is even checked
                return Err(VmError::InstrPointerOutOfRange(self.current.ip, self.stack_trace()));
            }

            match self.procs[self.current.id].instrs[self.current.ip] {
                Op::F64Add(dest, a, b) => { 
                    let a_addr = self.current.locals[a];
                    let b_addr = self.current.locals[b];
                    let dest_addr = self.current.locals[dest];

                    let a : [u8; 8] = self.memory[a_addr  ..= a_addr + FLEN].try_into().unwrap();
                    let b : [u8; 8] = self.memory[b_addr  ..= b_addr + FLEN].try_into().unwrap();

                    let a = f64::from_ne_bytes(a);
                    let b = f64::from_ne_bytes(b);

                    let answer = f64::to_ne_bytes( a + b );
                    self.memory[dest_addr .. dest_addr + FLEN].copy_from_slice(&answer);

                    self.current.ip += 1;
                },
                Op::F64Sub(dest, a, b) => { 

                    self.current.ip += 1;
                },
                Op::F64Mul(dest, a, b) => { 

                    self.current.ip += 1;
                },
                Op::F64Div(dest, a, b) => { 
                    self.current.ip += 1;
                },
                Op::F64Exp(dest, a, b) => { 

                    self.current.ip += 1;
                },
                Op::F64Neg(dest, x) => { 

                    self.current.ip += 1;
                },
                Op::F64Eq(dest, a, b) => { 
                    
                    self.current.ip += 1;
                },
                Op::F64Gt(dest, a, b) => {
                    self.current.ip += 1;
                },
                Op::F64Lt(dest, a, b) => { 
                    self.current.ip += 1;
                },
                Op::I64Add(dest, a, b) => { 
                    self.current.ip += 1;
                },
                Op::I64Sub(dest, a, b) => { 

                    self.current.ip += 1;
                },
                Op::I64Mul(dest, a, b) => { 
                    self.current.ip += 1;
                },
                Op::I64Div(dest, a, b) => { 
                    self.current.ip += 1;
                },
                Op::I64Mod(dest, a, b) => { 
                    self.current.ip += 1;
                },
                Op::I64Exp(dest, a, b) => { 

                    self.current.ip += 1;
                },
                Op::I64Neg(dest, x) => { 
                    self.current.ip += 1;
                },
                Op::I64Eq(dest, a, b) => {
                    self.current.ip += 1;
                },
                Op::I64Gt(dest, a, b) => { 
                    self.current.ip += 1;
                },
                Op::I64Lt(dest, a, b) => { 

                    self.current.ip += 1;
                },

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
