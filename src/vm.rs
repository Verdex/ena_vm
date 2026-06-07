
use std::rc::Rc;
use crate::byteable::Byteable;
use crate::data::{Op, CompiledProc, VmError, StackTrace };


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
                Op::DataToHeap(x, ref data) => {
                    let data = &data.0;
                    let addr = self.current.locals[x];
                    if addr > self.memory.len() {
                        return Err(VmError::MemoryAccessOutOfRange(addr, self.stack_trace()));
                    }
                    if addr + data.len() > self.memory.len() {
                        return Err(VmError::SetMemoryOutOfRange(addr, data.len(), self.stack_trace()));
                    }
                    self.memory[addr .. addr + data.len()].copy_from_slice(data);
                    self.current.ip += 1;
                },
                Op::ReturnLocal(x) => { 
                    let addr = self.current.locals[x];
                    if let Some(f) = self.frames.pop() {
                        ret = Some(addr); 
                        todo!() // TODO
                    }
                    else {
                        return Ok(addr);
                    }
                },
                Op::LocalPtrAdd(dest, ptr, offset) => {
                    self.current.ip += 1;
                },
                Op::LocalPtrSub(dest, ptr, offset) => {
                    self.current.ip += 1;
                },
                Op::PtrAdd(dest, ptr, offset) => {
                    self.bin_math(dest, ptr, offset, |x:usize, y:isize| x.checked_add_signed(y))?;
                    self.current.ip += 1;
                },
                Op::PtrSub(dest, ptr, offset) => {
                    self.bin_math(dest, ptr, offset, |x:usize, y:isize| x.checked_sub_signed(y))?;
                    self.current.ip += 1;
                },
                Op::OffsetAdd(dest, a, b) => {
                    self.bin_math(dest, a, b, |x:isize, y:isize| Some(x + y))?;
                    self.current.ip += 1;
                },
                Op::OffsetSub(dest, a, b) => {
                    self.bin_math(dest, a, b, |x:isize, y:isize| Some(x - y))?;
                    self.current.ip += 1;
                },
                Op::OffsetMul(dest, a, b) => {
                    self.bin_math(dest, a, b, |x:isize, y:isize| Some(x * y))?;
                    self.current.ip += 1;
                },
                Op::OffsetDiv(dest, a, b) => {
                    self.bin_math(dest, a, b, |x:isize, y:isize| Some(x / y))?;
                    self.current.ip += 1;
                },
                Op::OffsetNeg(dest, x) => {
                    self.uni_math(dest, x, isize::from_ne_bytes, isize::to_ne_bytes, |x| -x)?;
                    self.current.ip += 1;
                },
                Op::F64Add(dest, a, b) => {  
                    self.bin_math(dest, a, b, |x:f64, y:f64| Some(x + y))?;
                    self.current.ip += 1;
                },
                Op::F64Sub(dest, a, b) => { 
                    self.bin_math(dest, a, b, |x:f64, y:f64| Some(x - y))?;
                    self.current.ip += 1;
                },
                Op::F64Mul(dest, a, b) => { 
                    self.bin_math(dest, a, b, |x:f64, y:f64| Some(x * y))?;
                    self.current.ip += 1;
                },
                Op::F64Div(dest, a, b) => { 
                    self.bin_math(dest, a, b, |x:f64, y:f64| Some(x / y))?;
                    self.current.ip += 1;
                },
                Op::F64Exp(dest, a, b) => { 
                    self.bin_math(dest, a, b, |x:f64, y:f64| Some(x.powf(y)))?;
                    self.current.ip += 1;
                },
                Op::F64Neg(dest, x) => { 
                    self.uni_math(dest, x, f64::from_ne_bytes, f64::to_ne_bytes, |x| -x)?;
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
                    self.bin_math(dest, a, b, |x:i64, y:i64| Some(x + y))?;
                    self.current.ip += 1;
                },
                Op::I64Sub(dest, a, b) => { 
                    self.bin_math(dest, a, b, |x:i64, y:i64| Some(x - y))?;
                    self.current.ip += 1;
                },
                Op::I64Mul(dest, a, b) => { 
                    self.bin_math(dest, a, b, |x:i64, y:i64| Some(x * y))?;
                    self.current.ip += 1;
                },
                Op::I64Div(dest, a, b) => { 
                    self.bin_math(dest, a, b, |x:i64, y:i64| Some(x / y))?;
                    self.current.ip += 1;
                },
                Op::I64Mod(dest, a, b) => { 
                    self.bin_math(dest, a, b, |x:i64, y:i64| Some(x % y))?;
                    self.current.ip += 1;
                },
                Op::I64Neg(dest, x) => { 
                    self.uni_math(dest, x, i64::from_ne_bytes, i64::to_ne_bytes, |x| -x)?;
                    self.current.ip += 1;
                },
                Op::I64Eq(dest, a, b) => {
                    self.bin_math(dest, a, b, |x:i64, y:i64| Some(x == y))?;
                    self.current.ip += 1;
                },
                Op::I64Gt(dest, a, b) => { 
                    self.bin_math(dest, a, b, |x:i64, y:i64| Some(x > y))?;
                    self.current.ip += 1;
                },
                Op::I64Lt(dest, a, b) => { 
                    self.bin_math(dest, a, b, |x:i64, y:i64| Some(x < y))?;
                    self.current.ip += 1;
                },

                _ => todo!(),
            }
        }
    }

    fn uni_math<T, const S: usize>(&mut self, 
        dest: usize, 
        x: usize, 
        from: fn([u8; S]) -> T, 
        to : fn(T) -> [u8; S],
        op: fn(T) -> T) -> Result<(), VmError> {

        let x_addr = self.current.locals[x];
        let dest_addr = self.current.locals[dest];

        if x_addr + S >= self.memory.len() {
            return Err(VmError::MemoryAccessOutOfRange(x_addr, self.stack_trace()));
        }
        let x : [u8; S] = self.memory[x_addr  .. x_addr + S].try_into().unwrap();

        let x = from(x);

        let answer = to( op(x) );

        if dest_addr + S > self.memory.len() {
            return Err(VmError::SetMemoryOutOfRange(dest_addr, S, self.stack_trace()));
        }
        self.memory[dest_addr .. dest_addr + S].copy_from_slice(&answer);
        Ok(())
    }

    fn bin_math<T1: Byteable<S1>, T2: Byteable<S2>, T3: Byteable<S3>, 
                F: Fn(T1, T2) -> Option<T3>, 
                const S1: usize, const S2: usize, const S3: usize>(
        &mut self, dest: usize, a: usize, b: usize, op: F) -> Result<(), VmError> {

        let a = self.deref(a)?;
        let b = self.deref(b)?;

        let answer = op(a, b).ok_or(VmError::BinMathOp(self.stack_trace()))?.to();

        self.set_deref(dest, &answer)?;
        Ok(())
    }

    fn set_deref(&mut self, dest: usize, value: &[u8]) -> Result<(), VmError> {
        let dest_addr = self.current.locals[dest];
        if dest_addr + value.len() > self.memory.len() {
            return Err(VmError::SetMemoryOutOfRange(dest_addr, value.len(), self.stack_trace()));
        }
        self.memory[dest_addr .. dest_addr + value.len()].copy_from_slice(&value);
        Ok(())
    }

    fn deref<T: Byteable<S>, const S: usize>(&self, local: usize) -> Result<T, VmError> {
        let addr = self.current.locals[local];
        if addr + S > self.memory.len() {
            return Err(VmError::MemoryAccessOutOfRange(addr, self.stack_trace()));
        }
        let value : [u8; S] = self.memory[addr  .. addr + S].try_into().unwrap();
        let value = Byteable::<S>::from(value);
        Ok(value)
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
    use crate::data;
    use super::*;

    #[test]
    fn should_handle_single_local_actions() {
        // Note:  Make sure that an item at the beginning and end of memory can be
        // set and retrieved
        const X : usize = data::I64_SIZE;
        let procs = vec![CompiledProc { 
            name: "main".into(),
            slot_names: vec![],
            instrs: vec![
                Op::AllocateData(0, X),
                Op::DataToHeap(0, data::int64(3)),
                Op::I64Add(0, 0, 0),
                Op::ReturnLocal(0),
            ],
            frame_size: 3,
        } ];
        let mut vm = Vm::new(procs);
        let addr = vm.run(0).unwrap(); 
        let x : [u8; X] = vm.memory[addr .. addr + X].try_into().unwrap();
        let x = i64::from_ne_bytes(x);
        assert_eq!(x, 6);
    }

    #[test]
    fn should_handle_two_param_math_op() {
        const X : usize = data::I64_SIZE;
        let procs = vec![CompiledProc { 
            name: "main".into(),
            slot_names: vec![],
            instrs: vec![
                Op::AllocateData(0, X),
                Op::AllocateData(1, X),
                Op::AllocateData(2, X),
                Op::DataToHeap(0, data::int64(3)),
                Op::DataToHeap(1, data::int64(7)),
                Op::I64Add(2, 0, 1),
                Op::ReturnLocal(2),
            ],
            frame_size: 3,
        } ];
        let mut vm = Vm::new(procs);
        let addr = vm.run(0).unwrap(); 
        let x : [u8; X] = vm.memory[addr .. addr + X].try_into().unwrap();
        let x = i64::from_ne_bytes(x);
        assert_eq!(x, 10);
    }
}

