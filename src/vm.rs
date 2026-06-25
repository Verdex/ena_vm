
use std::rc::Rc;
use crate::byteable::Byteable;
use crate::data::{Op, CompiledProc, VmError, StackTrace};

struct Frame {
    id: usize,
    ip: usize,
    locals: Vec<usize>,
}

pub struct Vm {
    memory: Vec<u8>,
    memory_len: usize,
    frames: Vec<Frame>, 
    procs: Vec<CompiledProc>,
    globals: Vec<usize>,
    current: Frame,
}

impl Vm {
    pub fn new(procs : Vec<CompiledProc>) -> Vm {
        Vm { 
            procs, 
            current: Frame { id: 0, ip: 0, locals: vec![] }, 
            frames: vec![], 
            memory: vec![],
            memory_len: 0,
            globals: vec![],
        }
    }

    pub fn run(&mut self, entry : usize) -> Result<usize, VmError> {
        const CO_RUN : usize = 1;
        const CO_FINISH : usize = 2;

        if entry >= self.procs.len() {
            return Err(VmError::UnknownProcId(entry, self.stack_trace()));
        }

        self.current.id = entry;
        self.current.locals = std::iter::repeat(0).take(self.procs[entry].frame_size).collect();

        let mut ret : Option<usize> = None;
        loop {
            if self.current.ip >= self.procs[self.current.id].instrs.len() {
                // TODO: with the right construction of compiled proc this might not have to be
                // something that is even checked (will need to reject procs without returns)
                return Err(VmError::InstrPointerOutOfRange(self.current.ip, self.stack_trace()));
            }

            match self.procs[self.current.id].instrs[self.current.ip] {
                Op::Jump(label) => {
                    self.current.ip = label;
                },
                Op::BranchTrue(label, local) => {
                    let test = self.deref(local)?;
                    if test {
                        self.current.ip = label; 
                    }
                    else {
                        self.current.ip += 1;
                    }
                },
                Op::AllocateData(x, size) => {
                    let addr = self.allocate(vec![0; size]);
                    self.current.locals[x] = addr;
                    self.current.ip += 1;
                },
                Op::EndOfMemory(x) => {
                    let addr = self.memory_len; 
                    self.current.locals[x] = addr;
                    self.current.ip += 1;
                },
                Op::DropMemory(x) => {
                    let x = self.current.locals[x];
                    if x > self.memory.len() {
                        return Err(VmError::MemoryDroppedOutOfRange { available: self.memory.len(), attempt: x, st: self.stack_trace() });
                    }
                    self.memory_len = x;
                    self.current.ip += 1;
                },
                Op::Coroutine(dest, proc, ref params) => { 
                    // Status: usize
                    // running = 1
                    // finished = 2
                    // running | proc: usize | ip: usize | local_len: usize | locals (list of usize)
                    // finished | proc: usize

                    let running_size = {
                        let status_size = std::mem::size_of::<usize>();
                        let proc_size = std::mem::size_of::<usize>();
                        let ip_size = std::mem::size_of::<usize>();
                        let local_len_size = std::mem::size_of::<usize>();
                        let locals_size = std::mem::size_of::<usize>() * self.procs[proc].frame_size;
                        status_size + proc_size + ip_size + local_len_size + locals_size 
                    };

                    let mut coroutine : Vec<u8> = CO_CREATE.to().into_iter()
                        .chain(proc.to())
                        .chain(params.len().to())
                        // TODO: This should be pulling the values out of the self.current.locals
                        .chain(params.iter().flat_map(|x| x.to()))
                        .collect();

                    if coroutine.len() < running_size {
                        coroutine.append(&mut vec![0; running_size - coroutine.len()]);
                    }
                    
                    let addr = self.allocate(coroutine);
                    self.current.locals[dest] = addr;
                    self.current.ip += 1;
                },
                Op::Resume(x) => {
                    // NOTE:  The coroutines live in memory, so the program can mess them up.  This
                    // means that everything needs to be checked.  For example, the proc ids
                    // generally don't need to be checked because if they're wrong that's a compiler
                    // defect.  But here a program could change that value to something invalid.

                    // TODO push the address of the coroutine on the locals of the coroutine so that
                    // it knows where to dump updated data in a yield.
                    let addr = self.current.locals[x];
                    let status : usize = self.from_address(addr)?;
                    match status {
                        CO_CREATE => {
                            let proc_addr = addr + std::mem::size_of::<usize>();
                            let param_len_addr = proc_addr + std::mem::size_of::<usize>();
                            let params_addr = param_len_addr + std::mem::size_of::<usize>();

                            let proc : usize = self.from_address(proc_addr)?;
                            if proc >= self.procs.len() {
                                return Err(VmError::UnknownProcId(entry, self.stack_trace()));
                            }

                            let params_len : usize = self.from_address(param_len_addr)?;
                            let params = {
                                let mut xs : Vec<usize> = vec![];
                                for offset in 0..params_len {
                                    xs.push(self.from_address(params_addr + (offset * std::mem::size_of::<usize>()))?);
                                }
                                xs
                            };

                            let locals = params.into_iter()
                                               .chain(std::iter::repeat(0usize).take(self.procs[proc].frame_size - params_len))
                                               .collect();

                            self.current.ip += 1;
                            
                            let new = Frame { 
                                id: proc, 
                                ip: 0,
                                locals,
                            };

                            let old = std::mem::replace(&mut self.current, new);

                            self.frames.push(old);

                            // Note:  The address of the coroutine is a hidden last local so that
                            // Yield knows where to put the serialized coroutine
                            self.current.locals.push(addr);
                        },
                        CO_RUN => {

                            // TODO
                        },
                        CO_FINISH => {
                            // TODO
                        },
                        z => {
                            // TODO need error
                        },
                    }
                    // TODO
                },
                Op::Yield(_) => {
                    // TODO
                },
                Op::Finish(dest, coroutine) => {
                    let addr = self.current.locals[coroutine];
                    let status : usize = self.from_address(addr)?;
                    if status == CO_FINISH {
                        self.set_deref(dest, &true.to())?;
                    }
                    else {
                        self.set_deref(dest, &false.to())?;
                    }
                    self.current.ip += 1;
                },
                Op::DataToHeap(x, ref data) => {
                    let data = &data.0;
                    let addr = self.current.locals[x];
                    if addr > self.memory_len {
                        return Err(VmError::MemoryAccessOutOfRange(addr, self.stack_trace()));
                    }
                    if addr + data.len() > self.memory_len {
                        return Err(VmError::SetMemoryOutOfRange(addr, data.len(), self.stack_trace()));
                    }
                    self.memory[addr .. addr + data.len()].copy_from_slice(data);
                    self.current.ip += 1;
                },
                Op::PtrToHeap(dest, x) => {
                    let x = self.current.locals[x].to();
                    self.set_deref(dest, &x)?;
                    self.current.ip += 1;
                },
                Op::PtrFromHeap(dest, x) => {
                    let ptr : usize = self.deref(x)?;
                    self.current.locals[dest] = ptr;
                    self.current.ip += 1;
                },
                Op::CopyDataInHeap(dest, x, len) => {
                    let x = self.deref_bytes(x, len)?;
                    self.set_deref(dest, &x)?;
                    self.current.ip += 1;
                },
                Op::ReturnLocal(x) => { 
                    let addr = self.current.locals[x];
                    if let Some(f) = self.frames.pop() {
                        ret = Some(addr); 
                        self.current = f;
                    }
                    else {
                        return Ok(addr);
                    }
                },
                Op::SetLocalFromReturn(dest) => {
                    match ret {
                        Some(x) => { self.current.locals[dest] = x; },
                        None => { 
                            return Err(VmError::ReturnDoesNotExist(self.stack_trace()));
                        },
                    }
                    self.current.ip += 1;
                },
                Op::SetLocalFromLocal(dest, x) => {
                    self.current.locals[dest] = self.current.locals[x];
                    self.current.ip += 1;
                },
                Op::SetLocalFromProc(dest, proc) => {
                    self.current.locals[dest] = proc;
                    self.current.ip += 1;
                },
                Op::SetLocalFromGlobal(dest, x) => {
                    self.current.locals[dest] = self.globals[x];
                    self.current.ip += 1;
                },
                Op::SetGlobalFromLocal(dest, x) => {
                    self.globals[dest] = self.current.locals[x];
                    self.current.ip += 1;
                },
                Op::Call(fun, ref params) => {
                    let params_len = params.len();
                    let locals = params.iter()
                                       .map(|x| self.current.locals[*x])
                                       .chain(std::iter::repeat(0).take(self.procs[fun].frame_size - params_len))
                                       .collect();

                    self.current.ip += 1;
                    
                    let new = Frame { 
                        id: fun, 
                        ip: 0,
                        locals,
                    };

                    let old = std::mem::replace(&mut self.current, new);

                    self.frames.push(old);
                },
                Op::DynCall(local, ref params) => {
                    let fun : usize = self.deref(local)?;
                    let params_len = params.len();
                    let locals = params.iter()
                                       .map(|x| self.current.locals[*x])
                                       .chain(std::iter::repeat(0).take(self.procs[fun].frame_size - params_len))
                                       .collect();

                    self.current.ip += 1;
                    
                    let new = Frame { 
                        id: fun, 
                        ip: 0,
                        locals,
                    };

                    let old = std::mem::replace(&mut self.current, new);

                    self.frames.push(old);
                },
                Op::LocalPtrAdd(dest, ptr, offset) => {
                    let offset : isize = self.deref(offset)?;
                    self.current.locals[dest] = self.current.locals[ptr].checked_add_signed(offset).ok_or(VmError::BinMathOp(self.stack_trace()))?;
                    self.current.ip += 1;
                },
                Op::LocalPtrSub(dest, ptr, offset) => {
                    let offset : isize = self.deref(offset)?;
                    self.current.locals[dest] = self.current.locals[ptr].checked_sub_signed(offset).ok_or(VmError::BinMathOp(self.stack_trace()))?;
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
                    self.uni_math(dest, x, |x:isize| -x)?;
                    self.current.ip += 1;
                },
                Op::OffsetEq(dest, a, b) => {
                    self.bin_math(dest, a, b, |x:isize, y:isize| Some(x == y))?;
                    self.current.ip += 1;
                },
                Op::OffsetGt(dest, a, b) => {
                    self.bin_math(dest, a, b, |x:isize, y:isize| Some(x > y))?;
                    self.current.ip += 1;
                },
                Op::OffsetLt(dest, a, b) => {
                    self.bin_math(dest, a, b, |x:isize, y:isize| Some(x < y))?;
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
                    self.uni_math(dest, x, |x:f64| -x)?;
                    self.current.ip += 1;
                },
                Op::F64Eq(dest, a, b) => { 
                    self.bin_math(dest, a, b, |x:f64, y:f64| Some(x == y))?; 
                    self.current.ip += 1;
                },
                Op::F64Gt(dest, a, b) => {
                    self.bin_math(dest, a, b, |x:f64, y:f64| Some(x > y))?; 
                    self.current.ip += 1;
                },
                Op::F64Lt(dest, a, b) => { 
                    self.bin_math(dest, a, b, |x:f64, y:f64| Some(x < y))?; 
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
                    self.uni_math(dest, x, |x:i64| -x)?;
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
                Op::LNot(dest, x) => {
                    self.uni_math(dest, x, |x:bool| !x)?;
                    self.current.ip += 1;
                },
                Op::LAnd(dest, a, b) => {
                    self.bin_math(dest, a, b, |x:bool, y:bool| Some(x && y))?;
                    self.current.ip += 1;
                },
                Op::LOr(dest, a, b) => {
                    self.bin_math(dest, a, b, |x:bool, y:bool| Some(x || y))?;
                    self.current.ip += 1;
                },
                Op::LXor(dest, a, b) => {
                    self.bin_math(dest, a, b, |x:bool, y:bool| Some(x ^ y))?;
                    self.current.ip += 1;
                },
                Op::LEq(dest, a, b) => {
                    self.bin_math(dest, a, b, |x:bool, y:bool| Some(x == y))?;
                    self.current.ip += 1;
                },
                Op::Nop => { self.current.ip += 1; },
            }
        }
    }

    fn uni_math<T: Byteable<S>, const S: usize>(&mut self, 
        dest: usize, 
        x: usize, 
        op: fn(T) -> T) -> Result<(), VmError> {

        let x = self.deref(x)?;
        let answer = op(x).to();
        self.set_deref(dest, &answer)?;
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
        if dest_addr + value.len() > self.memory_len {
            return Err(VmError::SetMemoryOutOfRange(dest_addr, value.len(), self.stack_trace()));
        }
        self.memory[dest_addr .. dest_addr + value.len()].copy_from_slice(&value);
        Ok(())
    }

    fn deref_bytes(&self, local: usize, len : usize) -> Result<Vec<u8>, VmError> {
        let addr = self.current.locals[local];
        if addr + len > self.memory_len {
            return Err(VmError::MemoryAccessOutOfRange(addr, self.stack_trace()));
        }
        let value : Vec<u8> = self.memory[addr  .. addr + len].try_into().unwrap();
        Ok(value)
    }

    fn deref<T: Byteable<S>, const S: usize>(&self, local: usize) -> Result<T, VmError> {
        let addr = self.current.locals[local];
        if addr + S > self.memory_len {
            return Err(VmError::MemoryAccessOutOfRange(addr, self.stack_trace()));
        }
        let value : [u8; S] = self.memory[addr  .. addr + S].try_into().unwrap();
        let value = Byteable::<S>::from(value);
        Ok(value)
    }

    fn from_address<T: Byteable<S>, const S: usize>(&self, addr: usize) -> Result<T, VmError> {
        if addr + S > self.memory_len {
            return Err(VmError::MemoryAccessOutOfRange(addr, self.stack_trace()));
        }
        let value : [u8; S] = self.memory[addr  .. addr + S].try_into().unwrap();
        let value = Byteable::<S>::from(value);
        Ok(value)
    }

    fn allocate(&mut self, mut data : Vec<u8>) -> usize {
        let len = self.memory_len;
        self.memory_len += data.len();
        self.memory.append(&mut data);
        return len;
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

