use pyo3::prelude::*;
use pyo3::types::PyTuple;
use actix::prelude::*;

pub struct Pump {
    pub pypump: Py<PyAny>,
}

impl Actor for Pump {
    type Context = Context<Self>;
}

/// Pump: Basis of the whole system
/// it pumps request to python classes, and returns it on (hopefully) JSON
/// a String -> String pump should be fine
/// Stuff will be serialised and deserialised in JSON anyway
#[derive(Message)]
#[rtype(result = "String")]
pub struct Fuel(pub String);

impl Handler<Fuel> for Pump {
    type Result = String;

    fn handle(&mut self, msg: Fuel, _ctx: &mut Self::Context) -> Self::Result {
        // push the raw kargs to python, let python extract this stuff...
        Python::with_gil(|py| {
            let args = PyTuple::new_bound(py, &[msg.0]);
            let result: String = self.pypump.call_method_bound(py, "pump", args, None).unwrap().extract(py).unwrap();
            result
        })
    }
}
