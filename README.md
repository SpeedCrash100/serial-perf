# serial perf

A library targeting embedded and linux devices with UART interfaces to conduct performance and stress tests on them. 

## Overview

The library is uses `embedded-hal-nb` and `embedded-timers` for easier portability between different platforms. 
There are 2 main test implemented in the library:

  - `Loopback` - a simple test that receive bytes and send them back to the sender.
  - `Counter` - a test for measuring loss of data over time. It sends an a special increasing number and can receive and verify it on the other side. You can use this test only as client, server or both depending on your needs

In addition to test there are utils for helping with testing.
  - `ByteRateSerialLimiter` - A wrapper around serial port that limits the bytes per second sent over it. This limits only TX, RX side is not limited


## Examples

Examples can be found in the `examples` folder.


## License

This project is licensed under the [MIT license](./LICENSE)

