use actix::{Actor, Addr, Handler, SyncArbiter, System};
use pyo3::prelude::*;
use rocket::{data::ToByteUnit, tokio::{io::AsyncReadExt, sync::RwLock}, Data, Error, State};
use std::{collections::HashMap, sync::Arc};
use rocket::form::Form;

#[macro_use]
extern crate rocket;

mod hydraulics;
mod pump;

static HYDRAULICS_WORKERS: usize = 2;

/// Get a Hydraulics Actor system and spawn a Pump
/// 
/// note:
/// Just stream the bytes to a file
/// then read it using python
/// don't make your life harder than it meant to be
#[post("/load/<pump_name>", data = "<data>")]
async fn load_pump(
    pump_name: String,
    data: Vec<u8>,
    shared_hydraulics: &State<Addr<hydraulics::Hydraulics>>,
    shared_pumps: &State<Arc<RwLock<HashMap<String, actix::Addr<pump::Pump>>>>>,
) -> String {
    // Read the bytes from python
    // let mut buffer: Vec<u8> = Vec::new();
    // data.open(512.kibibytes()).read_to_end(&mut buffer).await.unwrap();

    println!("{:?}", String::from_utf8(data.clone()));

    let pyslug = hydraulics::PySlug { 0: data.clone() };

    // Send and Extract the pyObject back
    let arc_py_pump = 
    match shared_hydraulics.send(pyslug).await {
        Ok(T) => T,
        Err(E) => panic!("Problem sending to hydraulics: {:?}", E)
    };
    let py_pump: Py<PyAny> = Arc::into_inner(arc_py_pump).unwrap();

    let pump_addr = pump::Pump { pypump: py_pump }.start();

    let mut mut_shared_pumps = shared_pumps.write().await;

    mut_shared_pumps.insert(pump_name, pump_addr);

    "OK".to_string()
}

#[post("/invoke/<pump_name>", data = "<inputs>")]
async fn run_pump(pump_name: String, inputs: String, shared_pumps: &State<Arc<RwLock<HashMap<String, actix::Addr<pump::Pump>>>>>) -> String {
    // Get that Pump Actor and send data to it, then return
    let read_shared_pumps = shared_pumps.read().await;

    let pump_handler = read_shared_pumps.get(&pump_name).unwrap();

    let msg = pump::Fuel {0: inputs};
    let result = pump_handler.send(msg).await.unwrap();

    result

}

// Add a Thread-Safe shared hashmap with the list of actors and addrs shared to rockets handlers.
#[actix_rt::main]
async fn main() {
    // let _ = System::new();
    let shared_pumps = Arc::new(RwLock::new(HashMap::<String, actix::Addr<pump::Pump>>::new()));
    let shared_hydraulics = hydraulics::Hydraulics.start();

    rocket::build()
        .manage(shared_hydraulics)
        .manage(shared_pumps)
        .mount("/", routes![load_pump, run_pump])
        .launch()
        .await.unwrap();
}
