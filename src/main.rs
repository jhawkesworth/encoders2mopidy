//use std::error::Error;
use std::thread;
use std::time::{Duration, Instant};

//use rust_gpiozero::*;

use rppal::gpio::{Gpio, Level};
use rppal::gpio::Trigger::FallingEdge;

//use std::io;
//use std::io::prelude::*;
//use std::sync::atomic::{AtomicBool, Ordering};
use serde::{Serialize, Deserialize};
//use rppal::gpio::Gpio;

use std::sync::mpsc;
use ureq::{Agent, Response, Error};
use Level::{High, Low};


/*
TODOs
* get rid of rust_gpiozero code and move it to using rppal
* figure out how to debounce the button presses.
* improve the structs so that I don't need 3 to handle the
result being string/integer or boolean.
* move all of the the message sending to a separate thread
* refactor out lots of duplicated code
* get the rgb leds to do anything useful
* move next button to next/prev on rotation

 */

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
const ENCODER_WAIT_MILLIS: u64 = 3;
const MOPIDY_RECOVERY_TIME_WAIT_MILLIS: u64 = 333;

const SWITCH_DEBOUNCE_MILLIS: u64 = 333;


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


fn mopidy_volume_message_handler(volume_receiver: std::sync::mpsc::Receiver<i32>) {


    loop {
        // only want to pop the last value off the receiver (use it like a stack)
        // iterating the values has exactly the desired effect of 'consuming'
        // all the values it seems.
        let content_iter = volume_receiver.try_iter();
        let mut new_volume = -1;
        //let mut debug_channel_content_size = 0;
        for value in content_iter {
            //debug_channel_content_size = debug_channel_content_size +1;
            new_volume = value;
        }
        //println!("THE CHANNEL CONTENT SIZE IS {}", debug_channel_content_size);

        // below zero makes mopidy go bang and conveniently also tells us
        // there's no change to process, so the thread can go back to sleep again.
        if new_volume > -1 {
            println!("setting volume to {}", new_volume);
            let agent: Agent = ureq::AgentBuilder::new()
                .timeout_read(Duration::from_secs(5))
                .timeout_write(Duration::from_secs(5))
                .build();
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

        // a nice big sleep here to stop flooding mopidy with changes
        thread::sleep(Duration::from_millis(MOPIDY_RECOVERY_TIME_WAIT_MILLIS));
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
    return match resp {
        Ok(response) => {
            let json: JsonRpcResponse = response.into_json().expect("duff json");
            println!("Current volume is {:?}", json.result);
            json.result
        }
        Err(Error::Status(code, _response)) => {
            println!("bad response {} ", code);
            -1  // ugh TODO FIX
        }
        Err(_) => {
            println!("really bad response");
            -1 // ugh TODO FIX
        }
    }
    // TODO consider return Option<i32> as this means 'dunno' right now.

}


fn next(last_processed: std::time::Instant) {
    if last_processed.elapsed() >= Duration::from_millis(SWITCH_DEBOUNCE_MILLIS) {
        let agent: Agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .build();
        println!("Advance to next track");
        let method: String = "core.playback.next".to_string();

        let change_response: Result<Response, Error> = agent.post(MOPIDY_RPC_ENDPOINT)
            //.send_json(SerdeValue::)  // todo get this to work with JsonRpcRequest
            .send_json(ureq::json!({
           "jsonrpc": "2.0",
           "id": 1,
           "method": method,
       }));
        match change_response {
            Ok(_response2) => {
                println!("Success response for method: {}", &method);
            }
            Err(Error::Status(code, _response)) => { panic!("bad response {} ", code); }
            Err(_) => { panic!("really bad response"); }
        }
    }
}

fn toggle_play_pause(last_processed: std::time::Instant) {
    if last_processed.elapsed() >= Duration::from_millis(SWITCH_DEBOUNCE_MILLIS) {
        //let new_now = Instant::now();
        //println!("It has been {:?}", new_now.saturating_duration_since(last_processed));
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
            Ok(_response2) => {
                println!("changed to state: {}", &method);
            }
            Err(Error::Status(code, _response)) => {panic!("bad response {} ", code);}
            Err(_) => { panic!("really bad response");}
        }
    } //else {
      //  println!("TOO SOON, ignoring");
    //}

}


fn main() -> Result<(), Box<dyn std::error::Error>>  {
    println!("encoders2mopidy starting");
    let (volume_transmitter, volume_sender) = mpsc::channel();

    // start the thread that sends volume change messages to mopidy.
    std::thread::spawn(move || mopidy_volume_message_handler(volume_sender));

    let rotary2_dt = Gpio::new()?.get(ROTARY2_DT)?.into_input_pullup(); // rot 1: 4
    let rotary2_clk = Gpio::new()?.get(ROTARY2_CLK)?.into_input_pullup(); // rot 1: 14
    //let rotary2_dt = DigitalInputDevice::new_with_pullup(12); //rot 1: 4
    //let rotary2_clk = DigitalInputDevice::new_with_pullup(6); //rot 1: 14
    let mut rotary2_button = Gpio::new()?.get(ROTARY2_BUTTON)?.into_input_pulldown(); // GPIO 16, phys 36
    let mut rotary1_button = Gpio::new()?.get(ROTARY1_BUTTON)?.into_input_pulldown();

    // listen for button presses and toggle play/pause state, with some simple
    // time-based switch debouncing.
    let mut rotary2_last_processed = Instant::now();
    rotary2_button.set_async_interrupt(FallingEdge, move |_level| {
        toggle_play_pause(rotary2_last_processed);
        rotary2_last_processed = Instant::now();
    }).expect("bad interrupt setup rot2");

    let mut rotary1_last_processed = Instant::now();
    rotary1_button.set_async_interrupt(FallingEdge, move |_level| {
        next(rotary1_last_processed);
        rotary1_last_processed = Instant::now();
    }).expect("bad interrupt setup rot1");

    let agent: Agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build();
    let mut volume = get_volume(&agent);
    println!("On startup, volume is {}", &volume);

    // loop forever reading the rotary2 encoder and sending new volume settings
    // to the mopidy_volume_message_handler thread.
    loop
    {
        thread::sleep(Duration::from_millis(ENCODER_WAIT_MILLIS));
        let mut dt_val = rotary2_dt.read();
        let mut clk_val = rotary2_clk.read();
        //let mut dt_val = rotary2_dt.value();
        //let mut clk_val = rotary2_clk.value();

        //if rotary2_dt.is_high() && rotary2_clk.is_low() {}
        if dt_val.eq(&High) && clk_val.eq(&Low) {
        //if dt_val.eq(&true) && clk_val.eq(&false) {
            volume = volume +1;
            println!("-> {}", volume);
            volume_transmitter.send(volume).unwrap();
            while clk_val.eq(&Low) {
                thread::sleep(Duration::from_millis(ENCODER_WAIT_MILLIS));
                clk_val = rotary2_clk.read();
            }
            while clk_val.eq(&High) {
                thread::sleep(Duration::from_millis(ENCODER_WAIT_MILLIS));
                clk_val = rotary2_clk.read();
            }
        } else if dt_val.eq(&High) && clk_val.eq(&High) {
            volume = volume -1;
            if volume < 0 { volume = 0;}
            println!("<- {}", volume);
            volume_transmitter.send(volume).unwrap();
            while dt_val.eq(&High) {
                thread::sleep(Duration::from_millis(ENCODER_WAIT_MILLIS));
                dt_val = rotary2_dt.read();
            }
        }
        else { // both false, wait a bit
           // thread::sleep(Duration::from_millis(ENCODER_WAIT_MILLIS));
        }
    }
    //Ok(())
}
