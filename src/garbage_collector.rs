use std::{thread, sync::Arc};

use crossbeam::channel::{
    Sender, Receiver, unbounded
};

use crate::runtime::Runtime;


pub struct GC{
    worker_sender:Sender<Box<dyn Fn() + Send>>,
    worker_reciever:Receiver<Box<dyn Fn() + Send>>
}

impl GC{
    pub fn new(runtime:Arc<Runtime>, workers:usize) -> Self{
        let (sender, reciever) = unbounded::<Box<dyn Fn()+Send>>();

        for i in 0..workers{
            let recv = reciever.clone();
            thread::spawn(move ||{
                loop{
                    let next = match recv.recv(){
                        Ok(v) => v,
                        Err(e) => return
                    };

                    (next)();
                };
            });
        }
        return Self { 
            worker_sender: sender, 
            worker_reciever: reciever 
        }
    }
}