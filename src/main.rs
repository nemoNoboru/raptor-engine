use actix::{Actor, Addr, SyncArbiter};
use percent_encoding::percent_decode_str;
use rocket::{tokio::sync::RwLock,  State};
use std::{collections::HashMap, fs, sync::{atomic::{AtomicBool, Ordering}, Arc}};

#[macro_use]
extern crate rocket;
use base64::prelude::*;

mod hydraulics;
mod pump;

static HYDRAULICS_WORKERS: usize = 4;

/// Get a Hydraulics Actor system and spawn a Pump
/// 
/// note:
/// Just stream the bytes to a file
/// then read it using python
/// don't make your life harder than it meant to be
#[post("/load/<pump_name>", data = "<file>")]
async fn load_pump(
    pump_name: String,
    file: String,
    shared_hydraulics: &State<Addr<hydraulics::Hydraulics>>,
    shared_pumps: &State<Arc<RwLock<HashMap<String, actix::Addr<pump::Pump>>>>>,
) -> String {
    let file_decoded : String = percent_decode_str(&file).decode_utf8_lossy().to_string();

    let path = "slugs/test.pkl";

    let bin_slug = BASE64_STANDARD.decode(file_decoded).unwrap();

    fs::write(path, bin_slug).unwrap();

    let pyslug = hydraulics::PySlug { 0: path.to_string() };

    let pump_addr = 
    match shared_hydraulics.send(pyslug).await {
        Ok(t) => t,
        Err(e) => panic!("Problem sending to hydraulics: {:?}", e)
    };

    let mut mut_shared_pumps = shared_pumps.write().await;

    mut_shared_pumps.insert(pump_name, pump_addr);

    "OK".to_string()
}

#[post("/invoke/<pump_name>", data = "<inputs>")]
async fn run_pump(pump_name: String, inputs: String, shared_pumps: &State<Arc<RwLock<HashMap<String, actix::Addr<pump::Pump>>>>>, py_status_ready: &State<AtomicBool>) -> String {
    // Get that Pump Actor and send data to it, then return
    let read_shared_pumps = shared_pumps.read().await;

    let pump_handler = read_shared_pumps.get(&pump_name).unwrap();

    let msg = pump::Fuel {0: inputs};

    println!("current status is  {}", py_status_ready.load(Ordering::SeqCst));

    // block the status as not ready, run python and then set it to ready again.
    py_status_ready.store(false, Ordering::SeqCst);    
    let result = pump_handler.send(msg).await.unwrap();
    py_status_ready.store(true, Ordering::SeqCst); 

    result
}

#[get("/status")]
async fn check_status(py_status_ready: &State<AtomicBool>) -> String {

    let status: bool = py_status_ready.load(Ordering::SeqCst);
    match status {
        true => "ready".to_string(),
        false => "processing".to_string()
    }
}

// Add a Thread-Safe shared hashmap with the list of actors and addrs shared to rockets handlers.
#[actix_rt::main]
async fn main() {
    // maybe move the rwlock-hashmap etc shanenigans to a dedicated actor.
    let py_status_ready = AtomicBool::new(true);
    let shared_pumps = Arc::new(RwLock::new(HashMap::<String, actix::Addr<pump::Pump>>::new()));
    let shared_hydraulics = hydraulics::Hydraulics.start();

    rocket::build()
        .manage(py_status_ready)
        .manage(shared_hydraulics)
        .manage(shared_pumps)
        .mount("/", routes![load_pump, run_pump, check_status])
        .launch()
        .await.unwrap();
}
