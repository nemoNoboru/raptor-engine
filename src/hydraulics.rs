use std::sync::Arc;
use pyo3::{prelude::*, types::IntoPyDict};
use actix::prelude::*;
use pyo3::types::PyTuple;

pub struct Hydraulics;

impl Actor for Hydraulics {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
       println!("Actor is alive");
    }

    fn stopped(&mut self, ctx: &mut Context<Self>) {
       println!("Actor is stopped");
    }
}

#[derive(Message)]
#[rtype(result="Arc<Py<PyAny>>")]
pub struct PySlug(pub Vec<u8>);

/// Highly unlikely that this stuff will work any time soon
/// PLaying with raw binary data of a python class in rust
/// 
/// If this works any time soon then it should return a ARC with a python class inside.
/// To be sent to a Pump so it can start pumping requests
impl Handler<PySlug> for Hydraulics {
    type Result = Arc<Py<PyAny>>;

    fn handle(&mut self, msg: PySlug, _ctx: &mut Self::Context) -> Self::Result {
        println!("hydraulics starting");
        Python::with_gil(|py| {
            println!("Gil adquired, now preparing for python");
            let args = PyTuple::new_bound(py, &[msg.0]);
            let locals = [("pickle", py.import_bound("pickle").unwrap())].into_py_dict_bound(py);
            let _ = locals.set_item("slug", args);
            let code 
            = "print(slug[0])";

            println!("running python");
            let loaded_slug: Py<PyAny> = py.eval_bound(code, None, Some(&locals)).unwrap().extract().unwrap(); 
            println!("python ran");
            Arc::new(loaded_slug)
        })
    }
}