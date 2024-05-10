use pyo3::prelude::*;
use actix::prelude::*;
use pyo3::types::PyTuple;

use crate::pump;

pub struct Hydraulics;

impl Actor for Hydraulics {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
       println!("Actor is alive");
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
       println!("Actor is stopped");
    }
}

#[derive(Message)]
#[rtype(result="Addr<pump::Pump>")]
pub struct PySlug(pub String);

/// Highly unlikely that this stuff will work any time soon
/// PLaying with raw binary data of a python class in rust
/// 
/// If this works any time soon then it should return a ARC with a python class inside.
/// To be sent to a Pump so it can start pumping requests
impl Handler<PySlug> for Hydraulics {
    type Result = Addr<pump::Pump>;

    fn handle(&mut self, msg: PySlug, _ctx: &mut Self::Context) -> Self::Result {
        Python::with_gil(|py| {
           let py_loader: Py<PyAny> = PyModule::from_code_bound(
            py,
            "import pickle
def load(arg):
    print(arg)
    with open(arg,'rb') as input_file:
        return pickle.load(input_file)"
            , "","").unwrap().getattr("load").unwrap().into();

            let args = PyTuple::new_bound(py, [msg.0]);

            let result = py_loader.call_bound(py, args, None).unwrap();

            pump::Pump{pypump: result}.start()
        })
    }
}