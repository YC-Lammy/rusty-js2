use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;

use futures::FutureExt;
use tokio::task::{JoinHandle, JoinError};

use crate::value::JValue;
use crate::operator;

use super::object::JObjectInner;

pub enum Promise{
    AsyncNative(JoinHandle<JValue>),
    Resolved(JValue),
}

impl Promise{
    pub fn native<F>(future:F) -> Arc<dyn JObjectInner> where F:Future<Output = JValue> + Send + 'static{
        let handle = tokio::spawn(future);
        return Arc::new(Promise::AsyncNative(handle))
    }
}

impl JObjectInner for Promise{
    fn call(&mut self, vmctx:&mut crate::vm::VmContext, this:JValue, args:&[JValue]) -> JValue {
        operator::throw(JValue::Null)
    }
}

impl Future for Promise{
    type Output = JValue;
    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        match self.get_mut(){
            Promise::AsyncNative(h) => { 
                let re = h.poll_unpin(cx);
                match re{
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(v) => {
                        match v{
                            Ok(v) => return Poll::Ready(v),
                            Err(e) => {
                                if e.is_panic(){
                                    let err = e.into_panic();

                                    if let Some(v) = err.downcast_ref::<JValue>(){
                                        operator::throw(*v)
                                    } else{
                                        println!("async error: {:#?}", err);
                                        operator::throw(JValue::Undefined)
                                    }
                                } else{
                                    println!("tokio join cancel error");
                                    operator::throw(JValue::Undefined)
                                }
                            }
                        }
                    }
                }
            },

            Promise::Resolved(v) => return Poll::Ready(*v),
        }
    }
}