use std::{sync::Arc, panic::{catch_unwind, UnwindSafe, RefUnwindSafe}, hash::Hash, borrow::Cow};
use std::ops::{
    Add,Sub,Div,Mul,Shl,Shr,BitAnd,BitOr,BitXor, Index, Rem
};

use cranelift::prelude::types;
use swc_ecma_ast::AssignOp;

use crate::builtins::{
    object::JObject, 
    symbol::Symbol, 
    string::JString, self
};
use crate::runtime::{
    RUNTIME
};
use crate::vm::VmContext;
use crate::operator;


#[test]
fn size_assert(){
    assert!(std::mem::size_of::<JValue>() == 16)
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum JValue{    
    Undefined,
    Null,
    Number(f64),
    BigInt(i64),
    Boolean(bool),
    Symbol(Symbol),
    String(JString),
    Object(*mut JObject),
}

impl RefUnwindSafe for JValue{}
impl UnwindSafe for JValue{}
unsafe impl Send for JValue{}
unsafe impl Sync for JValue{}

impl JValue{
    pub const TYPE:types::Type = types::I128;

    pub fn null(&self) -> Option<()>{
        match self{
            Self::Null => Some(()),
            _ => None
        }
    }

    pub fn undefined(&self) -> Option<()>{
        match self{
            Self::Undefined => Some(()),
            _ => None
        }
    }

    pub fn number(&self) -> Option<f64>{
        match self{
            Self::Number(v) => Some(*v),
            _ => None
        }
    }

    pub fn bigint(&self) -> Option<i64>{
        match self{
            Self::BigInt(v) => Some(*v),
            _ => None
        }
    }

    pub fn symbol(&self) -> Option<Symbol>{
        match self{
            Self::Symbol(v) => Some(*v),
            _ => None
        }
    }

    pub fn string(&self) -> Option<JString>{
        match self{
            Self::String(v) => Some(*v),
            _ => None
        }
    }

    pub fn object(&self) ->  Option<&'static mut JObject>{
        match *self{
            Self::Object(o) => unsafe{Some(o.as_mut().unwrap())},
            _ => None
        }
    }

    pub fn is_null(&self) -> bool{
        match self{
            Self::Null => true,
            _ => false
        }
    }

    pub fn is_undefined(&self) -> bool{
        match self{
            Self::Undefined => true,
            _ => false
        }
    }

    pub fn is_number(&self) -> bool{
        match self{
            Self::Number(_) => true,
            _ => false
        }
    }

    pub fn is_bigint(&self) -> bool{
        match self{
            Self::BigInt(_) => true,
            _ => false
        }
    }

    pub fn is_symbol(&self) -> bool{
        match self{
            Self::Symbol(_) => true,
            _ => false
        }
    }

    pub fn is_string(&self) -> bool{
        match self{
            Self::String(_) => true,
            _ => false
        }
    }

    pub fn is_object(&self) -> bool{
        match self{
            Self::Object(_) => true,
            _ => false
        }
    }
    
    
    /// keep the value alive, preventing it being GC
    /// 
    /// user must call this method before 
    /// storing Objects outside the runtime.
    pub fn keep_alive(&self, b:bool){
        if let JValue::Object(o) = *self{
            todo!()
        }
    }

    pub fn to_float(self) -> f64{
        match self{
            JValue::Null => 0.0,
            JValue::Undefined => f64::NAN,
            JValue::BigInt(i) => i as f64,
            JValue::Boolean(b) => b as u8 as f64,
            JValue::Number(n) => n,
            JValue::String(s) => {
                if let Ok(f) = s.parse::<f64>(){
                    f
                } else{
                    f64::NAN
                }
            },
            JValue::Symbol(s) => f64::NAN,
            JValue::Object(o) => {
                f64::NAN
            },
        }
    }

    pub fn to_i32(self) -> i32{
        match self{
            JValue::Null => 0,
            JValue::Undefined => 0,
            JValue::Boolean(b) => b as i32,
            JValue::BigInt(b) => b as i32,
            JValue::Number(n) => n as i32,
            JValue::Object(o) => 0,
            JValue::String(n) => 0,
            JValue::Symbol(s) => 0,
        }
    }

    pub fn to_bool(self) -> bool{
        match self{
            JValue::Null => false,
            JValue::Undefined => false,
            JValue::BigInt(b) => b!=0,
            JValue::Boolean(b) => b,
            JValue::Number(n) => n!=0.0,
            JValue::Object(o) => true,
            JValue::String(s) => unsafe{*s.0 != 0},
            JValue::Symbol(s) => true,
        }
    }

    pub fn new(self) {

    }

    pub fn call(self, this:JValue, args:&[JValue]) -> Result<JValue, JValue>{
        RUNTIME.with(|runtime|{
            let (re, ok) = unsafe{
                self.call_raw(
                    &mut runtime.to_mut().context,
                    this,
                    args.as_ptr(), 
                    args.len() as i64, false)
            };
            if ok{
                return Ok(re)
            } else{
                return Err(re)
            }
        })
        
    }
    
    pub(crate) unsafe fn call_raw(self, vmctx:&mut VmContext, this:JValue, argv:*const JValue, argc:i64, spread:bool) -> (JValue, bool){
        if let Some(o) = self.object(){
            match &o.inner{
                Some(b) => {

                    let args = std::slice::from_raw_parts(argv, argc as usize);

                    // spread the last argument
                    let re = if spread{

                        let mut v = args.to_vec();
                        v.extend(operator::IteratorCollect(args[args.len()-1]));

                        let ctx = vmctx as *mut VmContext;

                        catch_unwind(||{
                            let r = self;
                            r.object().unwrap().inner_ref_uncheck().borrow_mut().call(ctx.as_mut().unwrap(), this, &v)
                        })

                    } else{
                        let ctx = vmctx as *mut VmContext;
                        
                        catch_unwind(||{
                            let r = self;
                            r.object().unwrap().inner_ref_uncheck().borrow_mut().call(ctx.as_mut().unwrap(), this, args)
                        })
                    };

                    match re{
                        Ok(v) => return (v, true),
                        Err(err) => {
                            if let Some(v) = err.downcast_ref::<JValue>(){
                                return (*v, false)
                            } else{
                                return (JValue::Undefined, false)
                            }
                        }
                    }
                },
                // todo: type Error
                None => return (JValue::Undefined, false)
            }
        } else{
            // todo: type Error
            return (JValue::Undefined, false)
        }
    }

    pub unsafe fn memberCall_raw(self, key:JValue, vmctx:&mut VmContext, argv:*const JValue, argc:i64, spread:bool) -> (JValue, bool){
        self.member(key).call_raw(vmctx, self, argv, argc, spread)
    }

    pub unsafe fn superMemberCall_raw(self, key:JValue, vmctx:&mut VmContext, argv:*const JValue, argc:i64, spread:bool) -> (JValue, bool){
        todo!()
    }

    pub fn member_str(self, name:&str) -> JValue{
        match self{
            JValue::Null => operator::throw(JValue::Undefined),
            JValue::Undefined => operator::throw(JValue::Undefined),
            JValue::Object(o) => return unsafe{&mut *o}.member_str(name),
            _ => todo!()
        }
        return JValue::Undefined
    }

    pub fn member(self, key:JValue) -> JValue{
        match key{
            JValue::String(s) => self.member_str(&s),
            JValue::Symbol(s) => self.member_str(&s),
            v => self.member_str(&v.to_string())
        }
    }

    pub fn set_member(self, key:JValue, value:JValue) {
        todo!()
    }

    
    pub fn assign_member(self, key:JValue, value:JValue, op:i8) -> JValue{
        
        match unsafe{std::mem::transmute::<_, AssignOp>(op)}{
            AssignOp::Assign => {
                self.set_member(key, value);
                return value
            }
            AssignOp::AddAssign => {
                let v = self.member(key) + value;
                self.set_member(key, v);
                return v
            },
            AssignOp::AndAssign => {
                let v = self.member(key).and(value);
                self.set_member(key, v);
                return v
            },
            AssignOp::BitAndAssign => {
                let v = self.member(key) & value;
                self.set_member(key, v);
                return v
            },
            AssignOp::BitOrAssign => {
                let v = self.member(key) | value;
                self.set_member(key, v);
                return v
            },
            AssignOp::BitXorAssign => {
                let v = self.member(key) ^ value;
                self.set_member(key, v);
                return v
            },
            AssignOp::DivAssign => {
                let v = self.member(key) / value;
                self.set_member(key, v);
                return v
            },
            AssignOp::ExpAssign => {
                let v = self.member(key).exp(value);
                self.set_member(key, v);
                return v
            },
            AssignOp::LShiftAssign => {
                let v = self.member(key) << value;
                self.set_member(key, v);
                return v
            },
            AssignOp::ModAssign => {
                let v = self.member(key) % value;
                self.set_member(key, v);
                return v
            },
            AssignOp::MulAssign => {
                let v = self.member(key) * value;
                self.set_member(key, v);
                return v
            },
            AssignOp::NullishAssign => {
                let v = self.member(key);
                if key.is_null() && key.is_undefined(){
                    self.set_member(key, value);
                    return value
                };
                return v;
            },
            AssignOp::OrAssign => {
                let v = self.member(key).or(value);
                self.set_member(key, v);
                return v
            },
            AssignOp::RShiftAssign => {
                let v = self.member(key) >> value;
                self.set_member(key, v);
                return v
            },
            AssignOp::SubAssign => {
                let v = self.member(key) - value;
                self.set_member(key, v);
                return v
            },
            AssignOp::ZeroFillRShiftAssign => {
                let v = self.member(key).unsignedRShift(value);
                self.set_member(key, v);
                return v
            },
        }
    }

    pub fn owned_keys(self) -> Vec<&'static str>{
        match self{
            JValue::Object(o) => {
                todo!()
            },
            _ => Vec::new()
        }
    }

    pub fn typeOf(self) -> JValue{
        match self{
            Self::Undefined => "undefined\0".into(),
            Self::Null => "null\0".into(),
            Self::Boolean(_) => "boolean\0".into(),
            Self::Number(_) => "number\0".into(),
            Self::BigInt(_) => "bigint\0".into(),
            Self::Symbol(_) => "symbol\0".into(),
            Self::String(_) => "string\0".into(),
            Self::Object(_) => "object\0".into()
        }
    }

    pub fn eqeqeq(self, rhs:Self) -> JValue{
        match self{
            JValue::String(s) => match rhs{
                JValue::String(s1) => JValue::Boolean(s.as_ref() == s1.as_ref()),
                _ => JValue::Boolean(false),
            },
            v => unsafe{JValue::Boolean(std::mem::transmute::<_,i128>(v) == std::mem::transmute(rhs))}
        }
    }

    pub fn and(self, rhs:Self) -> JValue{
        JValue::Boolean(self.to_bool() && rhs.to_bool())
    }

    pub fn or(self, rhs:Self) -> JValue{
        JValue::Boolean(self.to_bool() || rhs.to_bool())
    }

    pub fn exp(self, rhs:Self) -> JValue{
        match self{
            JValue::Number(n) => {
                JValue::Number(n.powf(rhs.to_float()))
            },
            JValue::BigInt(b) => {
                JValue::BigInt(b.pow(rhs.to_float() as u32))
            },
            _ => JValue::Number(self.to_float().powf(rhs.to_float()))
        }
    }

    pub fn unsignedRShift(self, rhs:Self) -> JValue{
        JValue::Number((self.to_i32() as u32 >> rhs.to_i32()) as f64)
    }
}

impl Add for JValue{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        match self{

            JValue::Null => match rhs{
                JValue::String(s) => JValue::String("null"+s),
                JValue::Null => JValue::Number(0.0),
                _ => rhs.add(self)
            },

            JValue::Symbol(_) => return JValue::Number(f64::NAN),
            JValue::Undefined => return JValue::Number(f64::NAN),

            JValue::Number(f) => match rhs{
                JValue::Undefined => JValue::Number(f64::NAN),
                JValue::Symbol(s) => match rhs{
                    _ => JValue::Number(f64::NAN)
                },

                JValue::Null => return JValue::Number(f),
                JValue::Number(f1) => JValue::Number(f+f1),
                JValue::BigInt(b) => JValue::Number(f + b as f64),
                JValue::Boolean(b) => JValue::Number(f + b as u8 as f64),
                JValue::String(s) => if let Ok(v) = s.parse::<f64>(){
                    JValue::Number(v + f)
                } else{
                    JValue::Number(f64::NAN)
                },
                JValue::Object(o) => {
                    JValue::Number(f64::NAN)
                }
            },

            JValue::BigInt(b) => match rhs{
                JValue::Undefined => JValue::Number(f64::NAN),
                JValue::Symbol(s) => match rhs{
                    _ => JValue::Number(f64::NAN)
                },
                JValue::Null => JValue::BigInt(b),
                JValue::Number(f) => JValue::BigInt(b + f as i64),
                JValue::BigInt(b1) => JValue::BigInt(b + b1),
                JValue::Boolean(b1) => JValue::BigInt(b + b1 as i64),
                JValue::String(s) => if let Ok(v) = s.parse::<i64>(){
                    JValue::BigInt(v+b)
                } else{
                    JValue::Number(f64::NAN)
                },
                JValue::Object(o) => {
                    JValue::Number(f64::NAN)
                }
            },

            JValue::Boolean(b) => match rhs{
                JValue::Null  => JValue::Number(b as u8 as f64),
                JValue::Undefined => JValue::Number(f64::NAN),
                JValue::Boolean(b1) => JValue::Number((b as u8 + b1 as u8) as f64),
                JValue::BigInt(b1) => JValue::BigInt(b1 + b as i64),
                JValue::Number(n) => JValue::Number(n + b as u8 as f64),
                _ => (b.to_string() + &rhs.to_string()).into()
            },

            JValue::String(s) => (s.to_string() + &rhs.to_string()).as_str().into(),

            JValue::Object(o) => match rhs{
                _ => JValue::Number(f64::NAN)
            }
        }
    }
}

impl Sub for JValue{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match self{
            JValue::Null => JValue::Number(0.0).sub(rhs),
            JValue::Number(f) => match rhs{
                JValue::Null => JValue::Number(f),
                JValue::Number(f1) => JValue::Number(f - f1),
                JValue::BigInt(b) => JValue::Number(f - b as f64),
                JValue::Boolean(b) => JValue::Number(f - b as u8 as f64),
                _ => JValue::Number(f64::NAN)
            },
            JValue::BigInt(b) => match rhs{
                JValue::Null => JValue::BigInt(b),
                JValue::BigInt(b1) => JValue::BigInt(b - b1),
                JValue::Number(f) => JValue::BigInt(b - f as i64),
                JValue::Boolean(b1) => JValue::BigInt(b - b1 as i64),
                _ => JValue::Number(f64::NAN)
            },
            JValue::String(s) => {
                if let Ok(v) = s.parse::<f64>(){
                    JValue::Number(v).sub(rhs)
                } else{
                    JValue::Number(f64::NAN)
                }
            },
            _ => JValue::Number(f64::NAN),
        }
    }
}

impl Div for JValue{
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        match self{
            JValue::Null => JValue::Number(0.0),
            JValue::Number(f) => match rhs{
                JValue::Null => JValue::Number(f / 0.0),
                JValue::Number(f1) => JValue::Number(f / f1),
                JValue::BigInt(b) => JValue::Number(f / b as f64),
                JValue::Boolean(b) => JValue::Number(f/ b as u8 as f64),
                _ => JValue::Number(f64::NAN)
            },
            JValue::BigInt(b) => match rhs{
                JValue::Null => JValue::BigInt(b / 0),
                JValue::BigInt(b1) => JValue::BigInt(b / b1),
                JValue::Number(f) => JValue::BigInt(b / f as i64),
                JValue::Boolean(b1) => JValue::BigInt(b/ b1 as i64),
                _ => JValue::Number(f64::NAN)
            },
            JValue::String(s) => {
                if let Ok(v) = s.parse::<f64>(){
                    JValue::Number(v).div(rhs)
                } else{
                    JValue::Number(f64::NAN)
                }
            },
            _ => JValue::Number(f64::NAN),
        }
    }
}

impl Mul for JValue{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        match self{
            JValue::Null => JValue::Number(0.0),
            JValue::Number(f) => match rhs{
                JValue::Null => JValue::Number(f * 0.0),
                JValue::Number(f1) => JValue::Number(f * f1),
                JValue::BigInt(b) => JValue::Number(f * b as f64),
                JValue::Boolean(b) => JValue::Number(f *b as u8 as f64),
                _ => JValue::Number(f64::NAN)
            },
            JValue::BigInt(b) => match rhs{
                JValue::Null => JValue::BigInt(b * 0),
                JValue::BigInt(b1) => JValue::BigInt(b * b1),
                JValue::Number(f) => JValue::BigInt(b * f as i64),
                JValue::Boolean(b1) => JValue::BigInt(b * b1 as i64),
                _ => JValue::Number(f64::NAN)
            },
            JValue::String(s) => {
                if let Ok(v) = s.parse::<f64>(){
                    JValue::Number(v).mul(rhs)
                } else{
                    JValue::Number(f64::NAN)
                }
            },
            _ => JValue::Number(f64::NAN),
        }
    }
}

impl Rem for JValue{
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        match self{
            JValue::Null => JValue::Number(0.0),
            JValue::Number(f) => match rhs{
                JValue::Null => JValue::Number(f % 0.0),
                JValue::Number(f1) => JValue::Number(f % f1),
                JValue::BigInt(b) => JValue::Number(f % b as f64),
                JValue::Boolean(b) => JValue::Number(f % b as u8 as f64),
                _ => JValue::Number(f64::NAN)
            },
            JValue::BigInt(b) => match rhs{
                JValue::Null => JValue::BigInt(b % 0),
                JValue::BigInt(b1) => JValue::BigInt(b % b1),
                JValue::Number(f) => JValue::BigInt(b % f as i64),
                JValue::Boolean(b1) => JValue::BigInt(b % b1 as i64),
                _ => JValue::Number(f64::NAN)
            },
            JValue::String(s) => {
                if let Ok(v) = s.parse::<f64>(){
                    JValue::Number(v).rem(rhs)
                } else{
                    JValue::Number(f64::NAN)
                }
            },
            _ => JValue::Number(f64::NAN),
        }
    }
}

impl BitAnd for JValue{
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        JValue::Number((self.to_i32() & rhs.to_i32()) as f64)
    }
}

impl BitOr for JValue{
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        JValue::Number((self.to_i32() | rhs.to_i32()) as f64)
    }
}

impl BitXor for JValue{
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        JValue::Number((self.to_i32() ^ rhs.to_i32()) as f64)
    }
}

impl Shl for JValue{
    type Output = Self;
    fn shl(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl Shr for JValue{
    type Output = Self;
    fn shr(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl PartialEq for JValue{
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl ToString for JValue{
    fn to_string(&self) -> String {
        match *self{
            JValue::Null => "null".to_owned(),
            JValue::Undefined => "undefined".to_owned(),
            JValue::BigInt(i) => i.to_string(),
            JValue::Number(f) => f.to_string(),
            JValue::Boolean(b) => b.to_string(),
            JValue::Object(o) => "[object Object]".to_owned(),
            JValue::String(s) => s.to_string(),
            JValue::Symbol(s) => s.to_string(),
        }
    }
}

impl Hash for JValue{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self{
            JValue::Null => {
                state.write_u8(0);
                state.write_usize(0x900);
            },
            JValue::Undefined => {
                state.write_u8(1);
                state.write_usize(0x901)
            },
            JValue::BigInt(i) => {
                state.write_u8(2);
                state.write_i64(*i)
            },
            JValue::Boolean(b) => {
                state.write_u8(3);
                state.write_u8(*b as u8)
            },
            JValue::Number(n) => {
                state.write_u8(4);
                state.write(&n.to_le_bytes())
            },
            JValue::String(s) => {
                state.write_u8(5);
                s.hash(state)
            },
            JValue::Symbol(s) => {
                state.write_u8(6);
                state.write_u32(s.id);
                s.intern.hash(state);
            },
            JValue::Object(o) => {
                state.write_u8(7);
                unsafe{(&**o).hash(state)};
            }
        }
    }
}

impl From<&'static mut JObject> for JValue{
    fn from(o: &'static mut JObject) -> Self {
        return JValue::Object(o)
    }
}

impl<F> From<F> for JValue where F:Fn(&mut VmContext, JValue, &[JValue]) -> JValue + 'static{
    fn from(func: F) -> Self {
        JObject::fromInner(builtins::function::Function::native(func)).into()
    }
}