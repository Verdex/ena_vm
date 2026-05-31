
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
                Op::AllocateData(x, size) => {
                    let len = self.memory.len();
                    self.memory.append(&mut vec![0; size]);
                    self.current.locals[x] = len;
                    self.current.ip += 1;
                },
                Op::SetData(x, offset, ref data) => {
                    let addr = self.current.locals[x];
                    if addr + offset > self.memory.len() {
                        return Err(VmError::MemoryAccessOutOfRange(addr + offset, self.stack_trace()));
                    }
                    if addr + offset + data.len() > self.memory.len() {
                        return Err(VmError::SetMemoryOutOfRange(addr + offset, data.len(), self.stack_trace()));
                    }
                    self.memory[addr + offset .. addr + offset + data.len()].copy_from_slice(data);
                    self.current.ip += 1;
                },
                Op::ReturnLocal(x) => { 
                    let addr = self.current.locals[x];
                    if let Some(f) = self.frames.pop() {
                        ret = Some(addr); 
                        todo!()
                    }
                    else {
                        return Ok(addr);
                    }
                },
                Op::F64Add(dest, a, b) => {  
                    self.bin_math(dest, a, b, f64::from_ne_bytes, f64::to_ne_bytes, |x, y| x + y)?;
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
                    self.bin_math(dest, a, b, i64::from_ne_bytes, i64::to_ne_bytes, |x, y| x + y)?;
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
    }

    fn bin_math<T, const S: usize>(&mut self, 
        dest: usize, 
        a: usize, 
        b: usize, 
        from: fn([u8; S]) -> T, 
        to : fn(T) -> [u8; S],
        op: fn(T, T) -> T) -> Result<(), VmError> {

        let a_addr = self.current.locals[a];
        let b_addr = self.current.locals[b];
        let dest_addr = self.current.locals[dest];

        if a_addr + S >= self.memory.len() {
            return Err(VmError::MemoryAccessOutOfRange(a_addr, self.stack_trace()));
        }
        if b_addr + S >= self.memory.len() {
            return Err(VmError::MemoryAccessOutOfRange(b_addr, self.stack_trace()));
        }
        let a : [u8; S] = self.memory[a_addr  .. a_addr + S].try_into().unwrap();
        let b : [u8; S] = self.memory[b_addr  .. b_addr + S].try_into().unwrap();

        let a = from(a);
        let b = from(b);

        let answer = to( op(a, b) );

        if dest_addr + S > self.memory.len() {
            return Err(VmError::SetMemoryOutOfRange(dest_addr, S, self.stack_trace()));
        }
        self.memory[dest_addr .. dest_addr + S].copy_from_slice(&answer);
        Ok(())
    }

    fn stack_trace(&self) -> StackTrace {
        // Note:  Previous frames will have already incremented past the current call op
        self.frames.iter().map(|x| (x.id, x.ip - 1))
                          .chain(std::iter::once( (self.current.id, self.current.ip) ) )
                          .map(|(id, ip)| (Rc::clone(&self.procs[id].name), ip))
                          .collect()
    }
}

#[cfg(test)]
mod test { 
    use super::*;

    #[test]
    fn should_handle_two_param_math_op() {
        // TODO use size of
        let procs = vec![CompiledProc { 
            name: "main".into(),
            slot_names: vec![],
            instrs: vec![
                Op::AllocateData(0, 8),
                Op::AllocateData(1, 8),
                Op::AllocateData(2, 8),
                Op::SetData(0, 0, i64::to_ne_bytes(3).to_vec()),
                Op::SetData(1, 0, i64::to_ne_bytes(7).to_vec()),
                Op::I64Add(2, 0, 1),
                Op::ReturnLocal(2),
            ],
            frame_size: 3,
        } ];
        let mut vm = Vm::new(procs);
        let addr = vm.run(0).unwrap(); 
        let x : [u8; 8] = vm.memory[addr .. addr + 8].try_into().unwrap();
        let x = i64::from_ne_bytes(x);
        assert_eq!(x, 10);
    }
}

