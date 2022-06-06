use std::marker;
use std::ops::Deref;
use std::borrow::Borrow;
use std::ops::{
    Add,Sub,Div,Mul,Shl,Shr,BitAnd,BitOr,BitXor, Index, Rem
};

use crate::bindgen;

use super::value;
use super::builtins;

pub enum StringOrNumber{
    String(String),
    Number(f64),
    BigInt(i64)
}

/// This is a value borrowed from the runtime, 
/// this value does not guarantee to stay alive,
/// it may be recycled during execution.
/// 
/// use ToOwned() to own the value and prevent it from being recycled.
#[repr(transparent)]
pub struct JValue<'a>{
    pub(crate) value:value::JValue,
    pub(crate) marker:marker::PhantomData<&'a ()>
}

impl<'a> JValue<'a>{
    pub fn object(&self) -> Option<JObject<'a>>{
        if let Some(o) = self.value.object(){
            Some(JObject{
                obj:o,
                mark:marker::PhantomData
            })
        } else{
            None
        }
    }

    pub fn number(&self) -> Option<f64>{
        self.value.number()
    }

    pub fn undefined(&self) -> Option<()>{
        self.value.undefined()
    }

    pub fn null(&self) -> Option<()>{
        self.value.null()
    }

    pub fn bigint(&self) -> Option<i64>{
        self.value.bigint()
    }

    pub fn string(&self) -> Option<String>{
        if let Some(v) = self.value.string(){
            Some(v.to_string())
        } else{
            None
        }
    }

    pub fn bool(&self) -> Option<bool>{
        self.value.bool()
    }
}

impl<'a> Deref for JValue<'a>{
    type Target = value::JValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a> AsRef<value::JValue> for JValue<'a>{
    fn as_ref(&self) -> &value::JValue {
        &self.value
    }
}

impl<'a> Add for JValue<'a>{
    type Output = StringOrNumber;
    fn add(self, rhs: Self) -> Self::Output {
        match (self.value + rhs.value){
            value::JValue::BigInt(i) => StringOrNumber::BigInt(i),
            value::JValue::Number(n) => StringOrNumber::Number(n),
            value::JValue::String(s) => StringOrNumber::String(s.to_string()),
            _ => unreachable!()
        }
    }
}

impl<'a> Sub for JValue<'a>{
    type Output = StringOrNumber;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self.value - rhs.value){
            value::JValue::BigInt(i) => StringOrNumber::BigInt(i),
            value::JValue::Number(n) => StringOrNumber::Number(n),
            value::JValue::String(s) => StringOrNumber::String(s.to_string()),
            _ => unreachable!()
        }
    }
}

impl<'a> Div for JValue<'a>{
    type Output = StringOrNumber;
    fn div(self, rhs: Self) -> Self::Output {
        match (self.value / rhs.value){
            value::JValue::BigInt(i) => StringOrNumber::BigInt(i),
            value::JValue::Number(n) => StringOrNumber::Number(n),
            value::JValue::String(s) => StringOrNumber::String(s.to_string()),
            _ => unreachable!()
        }
    }
}

impl<'a> Mul for JValue<'a>{
    type Output = StringOrNumber;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self.value * rhs.value){
            value::JValue::BigInt(i) => StringOrNumber::BigInt(i),
            value::JValue::Number(n) => StringOrNumber::Number(n),
            value::JValue::String(s) => StringOrNumber::String(s.to_string()),
            _ => unreachable!()
        }
    }
}

impl<'a> Shl for JValue<'a>{
    type Output = StringOrNumber;
    fn shl(self, rhs: Self) -> Self::Output {
        match (self.value << rhs.value){
            value::JValue::BigInt(i) => StringOrNumber::BigInt(i),
            value::JValue::Number(n) => StringOrNumber::Number(n),
            value::JValue::String(s) => StringOrNumber::String(s.to_string()),
            _ => unreachable!()
        }
    }
}

impl<'a> Shr for JValue<'a>{
    type Output = StringOrNumber;
    fn shr(self, rhs: Self) -> Self::Output {
        match (self.value >> rhs.value){
            value::JValue::BigInt(i) => StringOrNumber::BigInt(i),
            value::JValue::Number(n) => StringOrNumber::Number(n),
            value::JValue::String(s) => StringOrNumber::String(s.to_string()),
            _ => unreachable!()
        }
    }
}

impl<'a> Rem for JValue<'a>{
    type Output = StringOrNumber;
    fn rem(self, rhs: Self) -> Self::Output {
        match (self.value % rhs.value){
            value::JValue::BigInt(i) => StringOrNumber::BigInt(i),
            value::JValue::Number(n) => StringOrNumber::Number(n),
            value::JValue::String(s) => StringOrNumber::String(s.to_string()),
            _ => unreachable!()
        }
    }
}

impl<'a> BitAnd for JValue<'a>{
    type Output = StringOrNumber;
    fn bitand(self, rhs: Self) -> Self::Output {
        match (self.value & rhs.value){
            value::JValue::BigInt(i) => StringOrNumber::BigInt(i),
            value::JValue::Number(n) => StringOrNumber::Number(n),
            value::JValue::String(s) => StringOrNumber::String(s.to_string()),
            _ => unreachable!()
        }
    }
}

impl<'a> BitOr for JValue<'a>{
    type Output = StringOrNumber;
    fn bitor(self, rhs: Self) -> Self::Output {
        match (self.value | rhs.value){
            value::JValue::BigInt(i) => StringOrNumber::BigInt(i),
            value::JValue::Number(n) => StringOrNumber::Number(n),
            value::JValue::String(s) => StringOrNumber::String(s.to_string()),
            _ => unreachable!()
        }
    }
}

impl<'a> BitXor for JValue<'a>{
    type Output = StringOrNumber;
    fn bitxor(self, rhs: Self) -> Self::Output {
        match (self.value ^ rhs.value){
            value::JValue::BigInt(i) => StringOrNumber::BigInt(i),
            value::JValue::Number(n) => StringOrNumber::Number(n),
            value::JValue::String(s) => StringOrNumber::String(s.to_string()),
            _ => unreachable!()
        }
    }
}

/// a user owned value that will not be recycled until drop
pub struct OwnedValue{
    pub(crate) value:JValue<'static>
}

impl<'a> ToOwned for JValue<'a>{
    type Owned = OwnedValue;
    fn to_owned(&self) -> Self::Owned {
        self.value.keep_alive(true);
        OwnedValue{
            value:unsafe{std::mem::transmute_copy(self)}
        }
    }
}

impl Drop for OwnedValue{
    fn drop(&mut self) {
        self.value.keep_alive(false)
    }
}

impl<'a> Borrow<JValue<'a>> for OwnedValue{
    fn borrow(&self) -> &JValue<'a> {
        &self.value
    }
}

impl<'a> AsRef<JValue<'a>> for OwnedValue{
    fn as_ref(&self) -> &JValue<'a> {
        &self.value
    }
}

pub struct JObject<'a>{
    pub(crate) obj:&'static mut builtins::JObject,
    pub(crate) mark:marker::PhantomData<&'a ()>
}

impl<'a> JObject<'a>{
    pub fn set<T>(&self, key:&str, value:T) where T:bindgen::Bindable{

    }
}

impl<'a> ToOwned for JObject<'a>{
    type Owned = OwnedJObject;
    fn to_owned(&self) -> Self::Owned {
        self.obj.keep_alive(true);
        OwnedJObject{
            obj:unsafe{std::mem::transmute_copy::<_, JObject<'static>>(self)}
        }
    }
}

pub struct OwnedJObject{
    pub(crate) obj:JObject<'static>,
}

impl<'a> AsRef<JObject<'a>> for OwnedJObject{
    fn as_ref(&self) -> &JObject<'a> {
        &self.obj
    }
}

impl<'a> Borrow<JObject<'a>> for OwnedJObject{
    fn borrow(&self) -> &JObject<'a>{
        &self.obj
    }
}

impl Drop for OwnedJObject{
    fn drop(&mut self) {
        self.obj.obj.keep_alive(false);
    }
}