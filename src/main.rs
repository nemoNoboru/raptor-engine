use std::{collections::HashMap, sync::Arc};

use actix::Handler;

#[macro_use] extern crate rocket;

mod pump;
mod hydraulics;

#[post("/load/<pump_name>")]
fn load_pump(pump_name: &str) {
    // Get a Hydraulics Actor system and spawn a Pump 
}

#[post("/invoke/<pump_name>")]
fn run_pump(pump_name: &str) {
    // Get that Pump Actor and send data to it, then return
}

// Add a Thread-Safe shared hashmap with the list of actors and addrs shared to rockets handlers.
#[launch]
fn rocket() -> _ {
    let shared_map = Arc::new(HashMap<String,Handler>::new());
    rocket::build().mount("/", routes![load_pump, run_pump])
}