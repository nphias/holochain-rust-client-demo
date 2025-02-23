use std::env;

use holochain_types::{dna::ActionHash, prelude::ExternIO};
use zome_client::*;

const APP_ID: &str = "hello-world"; // can be obtained from the happ.yaml


/// This is a simple example of how to use the rust zome-client library to interact with a Holochain conductor.
/// This example assumes that you have a running Holochain conductor with the hello-world DNA installed.
/// The conductor must be listening on the specified admin port passed in as a command line argument.
/// The demo code connects to the conductor, gets the cell id of the first cell, creates a message, and then gets the messages.

#[tokio::main]  // allows an async main function
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <admin port>", args[0]);
        return;
    }
    let admin_port: u16 = args[1].parse().expect("Invalid port number");

    println!("Connecting to Conductor...");
    let app: AppSessionData = AppSessionData::init(APP_ID.to_string(),admin_port).await.unwrap();
    println!("{} Connected",APP_ID);
    let cell_id = app.get_cell_id_by_role(None).unwrap();
    println!("Create a message");
    let payload = ExternIO::encode("hello holochain message".to_string()).unwrap();
    let response = app.clone().zomecall(cell_id.clone(),"hello_world","hello_world",Some(payload)).await.unwrap();
    println!("Response: {:?}",ExternIO::decode::<ActionHash>(&response));
    println!("get messages");
    let response = &app.zomecall(cell_id,"hello_world","get_hellos",None).await.unwrap();
    println!("Response: {:?}",ExternIO::decode::<Vec<HelloOutput>>(&response));
}