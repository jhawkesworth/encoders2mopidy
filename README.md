# encoders2mopidy

A service to allow Mopidy to be controlled using Pimoroni COM1006 RGB Rotary Encoders.

Written in rough-and-ready Rust, cross compiled for Arm v6 (Raspberry Pi Zero W).

I am not recommending anybody does this.
Having realised rotary encoders really need a microcontroller, I decided to plough on and see if I could use rust to get me out of the hole I had dug myself into by buying bits before doing the reading.
This is the result. 

---
If anyone happens to have the necessary hardware lying around, here's how it needs to be setup.

BOM

Pirate audio 3w stereo board: https://shop.pimoroni.com/products/pirate-audio-3w-stereo-amp
Raspberry Pi Zero W https://thepihut.com/products/raspberry-pi-zero-w

Veroboard

2 * RGB Rotary encoders with push switch  https://shop.pimoroni.com/products/rotary-encoder-illuminated-rgb 
common anode
Any old momentary contact push button
8 * 220 Ohm resistors

MicroSD Card at least 8Gb
Wire and solder

TODO fritzing diagram.



The veroboard is essentially just a way of inserting the correct resistors between the gpio pins and the encoders.

Since they are common anode LEDS, they are powered from 3v3 bus and switched by the gpio pins.

```
                       |-------- >| R ------------------ 220Ohm --- GPIO
           +3v3  ------+-------- >| G ----------------- 220Ohm --- GPIO
                       |-------- >| B ------------------ 220Ohm --- GPIO
                       |-------- / SW ---------------- 220Ohm --- GPIO

GPIO ----- ENC DT ---
GND  ------------------<
GPIO ----- ENC CLK ---
```

Setup

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

Copy the encoders2modpidy.service file to /etc/systemd/system (you'll need to sudo).

The exe should then be manageable as a service using normal systemd operations.

Remember to enable using systemctl

Software power on off via gpio button.

If you have a pushbutton lying around you can connect it up between a ground pin and
gpio 3 (only gpio 3 apparent - tip from stderr.nl/Blog/ )
and it will  enables soft shutdown/bootup via the button press, if you add the following
 line to /boot/config.txt
```
dtoverlay=gpio-shutdown,gpio_pin=3
```
The board still gets powered and you'd do well to wait until all the LED flashing
has stopped before disconnecting the power cable. 

