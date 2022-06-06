

/*
    this module relies on `#![feature(unboxed_closures)]` 
    to bind functions into js functions.
    convertion is done automatically.

    this reduce the need to varefy and convert values in bulidin functions.s
 */

use std::{sync::Arc, collections::HashMap};
use std::panic::panic_any;

use crate::utils::ToMutable;
use crate::{value::JValue, prelude, builtins::{self, JObject}, vm::VmContext};

pub fn bind_function<F, Args, T>(f:F) -> Arc<dyn Fn(&mut VmContext, JValue, &[JValue]) -> JValue + 'static>
where F:Fn<Args, Output = T> +'static, Args:Arguments, T:Returnable{

    Arc::new(move |vmctx:&mut VmContext, this:JValue, args:&[JValue]|{
        let args = Args::from_values(this, args);
        let re = f.call(args);
        re.into_value()
    })
}


pub trait Arguments{
    fn from_values(this:JValue, values:&[JValue]) -> Self;
}

macro_rules! gen_args {
    ($($typ:tt),*) => {
        impl<This $(, $typ)*, Z> Arguments for (This $(, $typ)*, Z) where This:Bindable $(, $typ:Bindable)* , Z:Last{
            fn from_values(this:JValue, values:&[JValue]) -> Self{
                let mut i = 0;
                (This::from_jvalue(this), $(
                    if let Some(v) = values.get(i){
                        i+=1;
                        $typ::from_jvalue(*v)
                    } else{
                        $typ::from_jvalue(JValue::Undefined)
                    },
                )* if i < values.len(){
                    Z::from_remain(&values[i..])
                } else{
                    Z::from_remain(&[])
                })
            }
        }
    };
}

impl<This> Arguments for (This) where This:Bindable{
    fn from_values(this:JValue, values:&[JValue]) -> Self {
        (This::from_jvalue(this))
    }
}
gen_args!();
gen_args!(A);
gen_args!(A,B);
gen_args!(A,B,C);
gen_args!(A,B,C,D);
gen_args!(A,B,C,D,E);
gen_args!(A,B,C,D,E,F);
gen_args!(A,B,C,D,E,F,G);
gen_args!(A,B,C,D,E,F,G,H);
gen_args!(A,B,C,D,E,F,G,H,I);
gen_args!(A,B,C,D,E,F,G,H,I,J);
gen_args!(A,B,C,D,E,F,G,H,I,J,K);
gen_args!(A,B,C,D,E,F,G,H,I,J,K,L);
gen_args!(A,B,C,D,E,F,G,H,I,J,K,L,M);
gen_args!(A,B,C,D,E,F,G,H,I,J,K,L,M,N);
gen_args!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O);
gen_args!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P);
gen_args!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q);
gen_args!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R);
gen_args!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S);
gen_args!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T);
gen_args!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U);
gen_args!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V);
gen_args!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W);
gen_args!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X);
gen_args!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y);

pub trait Bindable{
    fn from_jvalue(value:JValue) -> Self;
}

impl<T> Bindable for Option<T> where T:Bindable{
    fn from_jvalue(value:JValue) -> Self {
        if value.is_undefined(){
            None
        } else{
            Some(T::from_jvalue(value))
        }
    }
} 

impl Bindable for String{
    fn from_jvalue(value:JValue) -> Self {
        value.to_string()
    }
}

impl Bindable for f64{
    fn from_jvalue(value:JValue) -> Self {
        value.to_float()
    }
}

impl Bindable for i64{
    fn from_jvalue(value:JValue) -> Self {
        value.to_float() as i64
    }
}

impl Bindable for i32{
    fn from_jvalue(value:JValue) -> Self {
        value.to_i32()
    }
}

impl Bindable for i16{
    fn from_jvalue(value:JValue) -> Self {
        value.to_i32() as i16
    }
}

impl Bindable for i8{
    fn from_jvalue(value:JValue) -> Self {
        value.to_i32() as i8
    }
}

impl Bindable for bool{
    fn from_jvalue(value:JValue) -> Self {
        value.to_bool()
    }
}

impl Bindable for (){
    fn from_jvalue(value:JValue) -> Self {
        ()
    }
}

impl Bindable for prelude::JObject<'_>{
    fn from_jvalue(value:JValue) -> Self {
        match value{
            JValue::Object(o) => prelude::JObject { 
                obj: unsafe{o.as_mut().unwrap()}, 
                mark: std::marker::PhantomData
            },
            _ => panic_any(builtins::Error::newTypeError("native function argument expected object.")),
        }
    }
}

impl Bindable for prelude::JValue<'_>{
    fn from_jvalue(value:JValue) -> Self {
        unsafe{std::mem::transmute(value)}
    }
}

impl Bindable for JValue{
    fn from_jvalue(value:JValue) -> Self {
        value
    }
}

impl Bindable for Option<&mut builtins::Array>{
    fn from_jvalue(value:JValue) -> Self {
        if let Some(o) = value.object(){
            if let Some(a) = o.inner.array(){
                return Some(a.to_mut())
            }
        }
        None
    }
}

pub trait Last{
    fn from_remain(values:&[JValue]) -> Self;
}

/*
impl<T> Last for Vec<T> where T:Bindable{
    fn from_remain(values:&[JValue]) -> Self {
        values.iter().map(|v|{T::from_jvalue(*v)}).collect()
    }
}
*/

impl Last for &[prelude::JValue<'_>] {
    fn from_remain(values:&[JValue]) -> Self {
        unsafe{std::mem::transmute(values)}
    }
}

impl Last for &[JValue] {
    fn from_remain(values:&[JValue]) -> Self {
        unsafe{std::mem::transmute(values)}
    }
}

impl<T> Last for T where T:Bindable{
    fn from_remain(values:&[JValue]) -> Self {
        return T::from_jvalue(values[0])
    }
}


pub trait Returnable{
    fn into_value(self) -> JValue;
}

impl<T> Returnable for Vec<T> where T:Returnable {
    fn into_value(self) -> JValue {
        let obj = JObject::new();
        let mut args = Vec::new();
        for i in self{
            args.push(i.into_value());
        };
        builtins::Array::new(obj, &args)
    }
}

impl<T, S> Returnable for HashMap<String, T, S> where T:Returnable{
    fn into_value(self) -> JValue {
        let obj = JObject::new();
        for (key, v) in self{
            obj.set_member_str(&key, v.into_value());
        };
        JValue::Object(obj)
    }
}

impl<T, const LEN:usize> Returnable for [T;LEN] where T:Returnable{
    fn into_value(self) -> JValue {
        let obj = JObject::new();
        let mut args = Vec::new();
        for i in self{
            args.push(i.into_value());
        };
        builtins::Array::new(obj, &args)
    }
}

impl<T> Returnable for Option<T> where T:Returnable{
    fn into_value(self) -> JValue {
        if let Some(v) = self{
            v.into_value()
        } else{
            JValue::Undefined
        }
    }
}

impl Returnable for JValue{
    fn into_value(self) -> JValue {
        self
    }
}

impl Returnable for prelude::JValue<'_>{
    fn into_value(self) -> JValue {
        self.value
    }
}

impl Returnable for prelude::JObject<'_>{
    fn into_value(self) -> JValue {
        JValue::Object(self.obj)
    }
}

impl Returnable for bool{
    fn into_value(self) -> JValue {
        JValue::Boolean(self)
    }
}

impl Returnable for (){
    fn into_value(self) -> JValue {
        JValue::Undefined
    }
}

impl Returnable for String{
    fn into_value(self) -> JValue {
        JValue::from(self)
    }
}

impl Returnable for &str{
    fn into_value(self) -> JValue {
        JValue::from(self)
    }
}

impl Returnable for f64{
    fn into_value(self) -> JValue {
        JValue::Number(self)
    }
}

impl Returnable for f32{
    fn into_value(self) -> JValue {
        JValue::Number(self as f64)
    }
}

impl Returnable for i64{
    fn into_value(self) -> JValue {
        JValue::Number(self as f64)
    }
}

impl Returnable for i32{
    fn into_value(self) -> JValue {
        JValue::Number(self as f64)
    }
}

impl Returnable for i16{
    fn into_value(self) -> JValue {
        JValue::Number(self as f64)
    }
}

impl Returnable for i8{
    fn into_value(self) -> JValue {
        JValue::Number(self as f64)
    }
}