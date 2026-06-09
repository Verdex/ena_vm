

use std::rc::Rc;

// locals and globals are all references
// dest, src, [src]

pub const F64_SIZE : usize = std::mem::size_of::<f64>();
pub const I64_SIZE : usize = std::mem::size_of::<i64>();
pub const PTR_SIZE : usize = std::mem::size_of::<usize>();
pub const OFFSET_SIZE : usize = std::mem::size_of::<isize>();
pub const BOOL_SIZE : usize = std::mem::size_of::<bool>();

#[derive(Debug)]
pub struct Data(pub (crate) Vec<u8>);

#[derive(Debug)]
pub enum Op<ID> {

    Jump(ID),
    BranchTrue(ID, ID),

    AllocateData(ID, usize),
    // TODO whether or not we need to use ret probably depends on the execution strat here
    Coroutine(ID, Vec<ID>),
    Resume(ID),
    Yield(ID),

    DataToHeap(ID, Data),
    PtrToHeap(ID, ID),
    PtrFromHeap(ID, ID),
    CopyData(ID, ID, usize),

    ReturnLocal(ID), 
    SetLocalFromReturn(ID),
    SetLocalFromLocal(ID, ID),
    SetLocalFromGlobal(ID, ID),
    SetLocalFromProc(ID, ID),
    SetGlobalFromLocal(ID, ID),

    Call(ID, Vec<ID>),
    DynCall(ID, Vec<ID>),

    // Dest pointer, source pointer, source offset
    LocalPtrAdd(ID, ID, ID), 
    LocalPtrSub(ID, ID, ID),
    PtrAdd(ID, ID, ID), 
    PtrSub(ID, ID, ID),

    OffsetAdd(ID, ID, ID),
    OffsetSub(ID, ID, ID),
    OffsetMul(ID, ID, ID),
    OffsetDiv(ID, ID, ID),
    OffsetNeg(ID, ID),

    OffsetEq(ID, ID, ID),
    OffsetGt(ID, ID, ID),
    OffsetLt(ID, ID, ID),

    F64Add(ID, ID, ID),
    F64Sub(ID, ID, ID),
    F64Mul(ID, ID, ID),
    F64Div(ID, ID, ID),
    F64Exp(ID, ID, ID),
    F64Neg(ID, ID),

    F64Eq(ID, ID, ID),
    F64Gt(ID, ID, ID),
    F64Lt(ID, ID, ID),

    I64Add(ID, ID, ID),
    I64Sub(ID, ID, ID),
    I64Mul(ID, ID, ID),
    I64Div(ID, ID, ID),
    I64Mod(ID, ID, ID),
    I64Neg(ID, ID),

    I64Eq(ID, ID, ID),
    I64Gt(ID, ID, ID),
    I64Lt(ID, ID, ID),

    LNot(ID, ID),
    LAnd(ID, ID, ID),
    LOr(ID, ID, ID),
    LXor(ID, ID, ID),
    LEq(ID, ID, ID),

    Nop,
}

pub fn int64(x: i64) -> Data {
    Data(i64::to_ne_bytes(x).to_vec())
}

pub fn float64(x: f64) -> Data {
    Data(f64::to_ne_bytes(x).to_vec())
}

pub fn bool(x: bool) -> Data {
    Data(vec![x as u8])
}

pub fn offset(x: isize) -> Data {
    Data(isize::to_ne_bytes(x).to_vec())
}

#[derive(Debug)]
pub struct Proc { 
    pub name : Rc<str>,
    pub instrs : Vec<Op<Rc<str>>>,
}

#[derive(Debug)]
pub struct CompiledProc { 
    pub name : Rc<str>,
    pub (crate) instrs : Vec<Op<usize>>,
    pub (crate) slot_names : Vec<Rc<str>>,
    pub (crate) frame_size : usize,
}


pub type StackTrace = Vec<(Rc<str>, usize)>;

#[derive(Debug)]
pub enum VmError {
    UnknownProcId(usize, StackTrace),
    InstrPointerOutOfRange(usize, StackTrace),
    MemoryAccessOutOfRange(usize, StackTrace),
    SetMemoryOutOfRange(usize, usize, StackTrace),
    BinMathOp(StackTrace),
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        fn d(x : &StackTrace) -> String {
            x.into_iter().map(|(n, i)| format!("    {} at index {}\n", n, i)).collect()
        }
        match self { 
            VmError::UnknownProcId(id, st) => write!(f, "encountered unknown proc id: {}\n{}", id, d(st)),
            VmError::InstrPointerOutOfRange(ip, st) => write!(f, "encountered instruction pointer past proc length: {}\n{}", ip, d(st)),
            VmError::MemoryAccessOutOfRange(addr, st) => write!(f, "memory access out of range: {}\n{}", addr, d(st)),
            VmError::SetMemoryOutOfRange(addr, len, st) => write!(f, "set memory out of range: {} of length: {}\n{}", addr, len, d(st)),
            VmError::BinMathOp(st) => write!(f, "error with binary operator\n{}", d(st)),
        }
    }
}

impl std::error::Error for VmError { }


