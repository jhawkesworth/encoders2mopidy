# encoders2mopidy

A service to allow Mopidy to be controlled using Pimoroni COM1006 RGB Rotary Encoders.

Written in rough-and-ready Rust, cross compiled for Arm v6 (Raspberry Pi Zero W).

Having realised rotary encoders really need a microcontroller, I decided to plough on and see 
if I could use rust to get me out of the hole I had dug myself into by buying bits before doing the reading.
Python was too slow to handle the rotary encoders and handle playing music on the little Pi Zero W, 
so I wondered, would rust be fast enough?

Initially I thought I'd write a mopidy plugin, but I found it has a rest api so it was much more
straightforward to call the api from rust then try to bridge rust to python via py03.

This is the result. 

It was fun, I learned a bunch, but I am not recommending anybody does this.

TLDR Rust is _plenty_ fast enough.  Even though there's a tight loop watching the encoder, it only uses about 2% cpu
time according to 'top'.

That said, the results aren't great - the volume encoder is pretty twitchy as there's 
only really a narrow range (about 20 values) in my particular 
setup as the amp is more powerful than the original 1960s speaker can take.
The buttons are still overly sensitive too.

I have since read that I should have used more ground pins rather than sharing a common one 
for everything connected to the veroboard, so that might improve things, as would better
soldering skills, using proper resistors rather than the variable ones I had lying around.

The code is not great either, using 2 different libraries to talk to gpio.  While I got the LEDs
to light, I haven't integrated the code for that either, so this is just using the rotary and built-in buttons.
One point about the code worth recalling is that I switched to ureq from reqwest as I am calling the rest api
locally on the same machine where mopidy is running and ureq lets you turn off https support, which I didn't
need, allowing for a much smaller binary and quicker compilation and remote sync, in this case.

I don't intend updating this code further but wanted to go through the process of doing it and documenting it on 
github as a learning experience, and a building block for further things.

---
If anyone happens to have the necessary hardware lying around, here's how it needs to be setup.

## Bill of Materials

* Pirate audio 3w stereo board: https://shop.pimoroni.com/products/pirate-audio-3w-stereo-amp
* Raspberry Pi Zero W https://thepihut.com/products/raspberry-pi-zero-w
* Veroboard
* 2 * RGB Rotary encoders with push switch  https://shop.pimoroni.com/products/rotary-encoder-illuminated-rgb 
common anode
* Any old momentary contact push button
* 8 * 220 Ohm resistors
* MicroSD Card at least 8Gb
* Some way of breaking out the unused pins from the Pirate audio board.
(I split a 20 cm 40-way IDC cable which I wouldn't recommend it was fiddly and took a long time)
* Hookup wire and solder
* Some old radio or case to put it in.
* A 4 Ohm speaker cable of being driven by a 3 W amp.  I used original radio speaker because it has character.

Here's how I laid out the veroboard.

![Veroboard diagram](veroboard-layout.jpg)

The veroboard is pretty much just a way of inserting the correct resistors between the gpio pins and the encoders.
Each square represents a hole in the veroboard and an X in the square indicates a track break.

About the RGB Encoders.
Since they are common anode LEDS, they are powered from 3v3 bus and switched by the gpio pins.

```
RGB Led part of encoder
                       |-------- >| R ----------------- 220 Ohm --- GPIO
           +3v3  ------+-------- >| G ----------------- 220 Ohm --- GPIO
                       |-------- >| B ----------------- 220 Ohm --- GPIO
                       |-------- / SW ----------------- 220 Ohm --- GPIO
Encoder connections

GPIO ----- ENC DT ---
GND  ------------------<
GPIO ----- ENC CLK ---
```

Despite what it says on pinout.xyz the Pirate Audio 3W board seems to use the following physical pins.
To be fair, there have clearly been a couple of revisions of the board so it 

2,12,14,17,19,21,22,23,26,33,35,40

I used the following physical pins
```
1 3v3
5 Ri Pi POWER Button ( short to earth to soft boot up down )
7 ENC1 ROTARY DT
8 ENC1 ROTARY CLK
9 GND
10 ENC1 SWITCH
11 ENC1 RED
13 ENC1 GREEN
15 ENC1 BLUE
27 ENC2 RED
28 ENC2 GREEN
29 ENC2 BLUE
31 ENC2 ROTARY DT
32 ENC2 ROTARY CLK
36 ENC2 SWITCH
```
It would have been better to make up some headers as it would have made experimenting and troubleshooting easier. 


# Setup

## Compilation

To cross-compile from sources on ubuntu
```
sudo apt install gcc-arm-linux-gnueabihf
```
See https://disconnected.systems/blog/rust-powered-rover/#setting-up-rust-for-cross-compiling
Note the extra fiddle for v6 (pi zero) as per
https://github.com/BurntSushi/ripgrep/issues/676#issuecomment-374058198
```
BUILDDIR="${HOME}/build-rpi"
mkdir -p "${BUILDDIR}"
test -d "${BUILDDIR}/tools" || git -C "${BUILDDIR}" clone --depth=1 https://github.com/raspberrypi/tools.git
```
For armv7 pi 2/3/4 full size boards (which I have not tested):
https://medium.com/swlh/compiling-rust-for-raspberry-pi-arm-922b55dbb050

Once you have the code cross-compiling, you can use the 'deploy' script to 
build and copy to your Raspberry Pi.  Run it like 
```
.\deploy <hostname>
```

Once you are happy the code is doing what you want,

## Set up a systemd unit to run the program as a service.

Copy the encoders2modpidy.service file to /etc/systemd/system (you'll need to sudo).

The exe should then be manageable as a service using normal systemd operations (systemctl/journalctl).

Remember to enable using systemctl so it comes up on reboot.

## Set up software power on off via gpio button.

If you have a pushbutton lying around you can connect it up between a ground pin and
gpio 3 (only gpio 3 apparent - tip from stderr.nl/Blog/ )
and it will  enables soft shutdown/bootup via the button press, if you add the following
 line to /boot/config.txt
```
dtoverlay=gpio-shutdown,gpio_pin=3
```
The board still gets powered and you'd do well to wait until all the LED flashing
has stopped before disconnecting the power cable. 

# Changelog
```
24 Jan 2021.  1st commit.
1 Feb 2020.  Fixed up the systemd unit
13 March 2021.  Added veroboard picture to readme.  
                Added failed experiment to get rid of rust_gpiozero to separate branch.
                Close down project.
```