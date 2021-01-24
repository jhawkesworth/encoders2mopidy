//use std::error::Error;
use std::thread;
use std::time::Duration;


use rust_gpiozero::*;

use rppal::gpio::Gpio;
use rppal::gpio::Trigger::FallingEdge;

use std::io;
use std::io::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use serde::{Serialize, Deserialize};
//use rppal::gpio::Gpio;

use ureq::{Agent, Response, Error};

//use reqwest::Url;

//const GPIO_R: u8 = 17;
//const GPIO_G: u8 = 27;
//const GPIO_B: u8 = 22;

//const GPIO_R: u8 = 0;
//const GPIO_G: u8 = 1;
//const GPIO_B: u8 = 5;

const ROTARY1_DT: u8 = 4; // GPIO 4 phys 7
const ROTARY1_CLK: u8 = 14; // GPIO 14 phy 8
const ROTARY1_BUTTON: u8 = 15; // GPIO 15 phys 10

const ROTARY2_DT: u8 = 12; // GPIO 12 phys 32
const ROTARY2_CLK: u8 = 6; // GPIO 6 phys 31
const ROTARY2_BUTTON: u8 = 16; // GPIO 16, phys 36

const MOPIDY_RPC_ENDPOINT: &str = "http://localhost:6680/mopidy/rpc";
const ENCODER_WAIT_MILLIS: u64 = 2;
static RUNNING: AtomicBool = AtomicBool::new(true);

fn watch_stdin() {
    // wait for key press to exit
    let _ = io::stdin().read(&mut [0u8]).unwrap();
    RUNNING.store(false, Ordering::Relaxed);
}

// {"jsonrpc": "2.0", "id": 1, "result": 20}
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
  //jsonrpc: String,
 // id: u64,
  result: i32,
}
#[derive(Debug, Deserialize)]
struct JsonRpcResponseBool {
    //jsonrpc: String,
    // id: u64,
    result: bool,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponseString {
    //jsonrpc: String,
    // id: u64,
    result: String,
}


#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: i32,
    method: String,
    params: Option<Vec<i32>>
}

fn set_volume(agent: &ureq::Agent, new_volume: i32) {
    println!("Setting volume");

    let resp: Result<Response, Error> = agent.post(MOPIDY_RPC_ENDPOINT)
        //.send_json(SerdeValue::)  // todo get this to work with JsonRpcRequest
         .send_json(ureq::json!({
           "jsonrpc": "2.0",
           "id": 1,
           "method": "core.mixer.set_volume",
           "params": vec![new_volume],
       }));
    match resp {
        Ok(response) => {
            let json: JsonRpcResponseBool = response.into_json().expect("duff json");
            println!("set volume result was {:?}", json.result);
        }
        Err(Error::Status(code, _response)) => {panic!("bad response {} ", code);}
        Err(_) => { panic!("really bad response");}
    }
}


fn get_volume(agent: &ureq::Agent) -> i32 {
    println!("Getting volume");
    // curl -d '{"jsonrpc": "2.0", "id": 1, "method": "core.mixer.get_volume"}' -H 'Content-Type: application/json' http://localhost:6680/mopidy/rpc

    let resp: Result<Response, Error> = agent.post(MOPIDY_RPC_ENDPOINT)
        .send_json(ureq::json!({
          "jsonrpc": "2.0",
          "id": 1,
          "method": "core.mixer.get_volume"
      }));
    match resp {
        Ok(response) => {
            let json: JsonRpcResponse = response.into_json().expect("duff json");
            println!("Current volume is {:?}", json.result);
            return json.result;
        }
        Err(Error::Status(code, _response)) => {
            println!("bad response {} ", code);
            return -1;  // ugh
        }
        Err(_) => {
            println!("really bad response");
            return -1; // ugh TODO FIX
        }
    }
    // TODO consider return Option<i32> as this means 'dunno' right now.

}


fn next() {
    let agent: Agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build();
    println!("Advance to next track");
    let method: String= "core.playback.next".to_string();

    let change_response: Result<Response, Error> = agent.post(MOPIDY_RPC_ENDPOINT)
        //.send_json(SerdeValue::)  // todo get this to work with JsonRpcRequest
        .send_json(ureq::json!({
           "jsonrpc": "2.0",
           "id": 1,
           "method": method,
       }));
    match change_response {
        Ok(response2) => {
            println!("Success response for method: {}", &method);
        }
        Err(Error::Status(code, _response)) => {panic!("bad response {} ", code);}
        Err(_) => { panic!("really bad response");}
    }

}

fn toggle_play_pause() {
    let agent: Agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build();
    println!("Toggle Play/pause volume");

    let mut method: String= "core.playback.pause".to_string();
    // get play state {"jsonrpc": "2.0", "id": 1, "result": "paused"}
    // may be paused, playing or stopped
    let resp: Result<Response, Error> = agent.post(MOPIDY_RPC_ENDPOINT)
        //.send_json(SerdeValue::)  // todo get this to work with JsonRpcRequest
        .send_json(ureq::json!({
           "jsonrpc": "2.0",
           "id": 1,
           "method": "core.playback.get_state",
       }));
    match resp {
        Ok(response) => {
            let json: JsonRpcResponseString = response.into_json().expect("duff json");
            println!("get_state result was {:?}", json.result);
            if json.result.ne("playing") {  // may be paused, playing or stopped
                method = "core.playback.play".to_string();
            }
        }
        Err(Error::Status(code, _response)) => {panic!("bad response {} ", code);}
        Err(_) => { panic!("really bad response");}
    }

    // actually try to change the state
    let change_response: Result<Response, Error> = agent.post(MOPIDY_RPC_ENDPOINT)
        //.send_json(SerdeValue::)  // todo get this to work with JsonRpcRequest
        .send_json(ureq::json!({
           "jsonrpc": "2.0",
           "id": 1,
           "method": method,
       }));
    match change_response {
        Ok(response2) => {
            println!("changed to state: {}", &method);
        }
        Err(Error::Status(code, _response)) => {panic!("bad response {} ", code);}
        Err(_) => { panic!("really bad response");}
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>>  {
    println!("encoders2mopidy starting");
    std::thread::spawn(watch_stdin);
    let rotary2_dt = DigitalInputDevice::new_with_pullup(12); //rot 1: 4
    let rotary2_clk = DigitalInputDevice::new_with_pullup(6); //rot 1: 14

    let mut rotary2_button = Gpio::new()?.get(ROTARY2_BUTTON)?.into_input_pulldown(); // GPIO 16, phys 36

    let mut rotary1_button = Gpio::new()?.get(ROTARY1_BUTTON)?.into_input_pulldown();

    let agent: Agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build();
    rotary2_button.set_async_interrupt(FallingEdge, |level| { toggle_play_pause() }).expect("bad interrupt setup rot2");

    rotary1_button.set_async_interrupt(FallingEdge, |level| {next()}).expect("bad interrupt setup rot1");

    let mut volume = get_volume(&agent);
    println!("volume is {}", &volume);

    while RUNNING.load(Ordering::Relaxed) {
        //println!("in while running.load");
        thread::sleep(Duration::from_millis(ENCODER_WAIT_MILLIS));
        let mut dt_val = rotary2_dt.value();
        let mut clk_val = rotary2_clk.value();

        if dt_val.eq(&true) && clk_val.eq(&false) {
            volume = volume +1;
            println!("-> {}", volume);
            set_volume(&agent, volume);
            while clk_val.eq(&false) {
                thread::sleep(Duration::from_millis(ENCODER_WAIT_MILLIS));
                clk_val = rotary2_clk.value();
            }
            while clk_val.eq(&true) {
                thread::sleep(Duration::from_millis(ENCODER_WAIT_MILLIS));
                clk_val = rotary2_clk.value();
            }
        } else if dt_val.eq(&true) && clk_val.eq(&true) {
            volume = volume -1;
            println!("<- {}", volume);
            set_volume(&agent, volume);
            while dt_val.eq(&true) {
                thread::sleep(Duration::from_millis(ENCODER_WAIT_MILLIS));
                dt_val = rotary2_dt.value();
            }
        }
        else { // both false, wait a bit
            thread::sleep(Duration::from_millis(ENCODER_WAIT_MILLIS));
        }
    }
    Ok(())
}
