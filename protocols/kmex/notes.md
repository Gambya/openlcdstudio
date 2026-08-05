## Confirmed transport behavior

The HL VMAX display accepts complete JPEG/JFIF files written directly to
its CDC ACM data port.

Confirmed properties:

- Device: `33c3:f101`
- Linux port: `/dev/serial/by-id/usb-HL_VMAX_HL-VMAX-USB-Device-if00`
- Image format: baseline JPEG/JFIF
- Native frame resolution observed: 462 × 1920
- No proprietary prefix is required before a JPEG
- No short control packet is required before image transmission
- Repeated JPEG transmission keeps the display active and stable
- Stable tests were completed at:
  - 2 FPS
  - 1 FPS
  - 0.5 FPS
- No flickering was observed
- After frame transmission stops, the display briefly presents its
  firmware default image and then turns black

This confirms that static themes must still be refreshed periodically.
The render loop belongs to `openlcd-runtime`, not to the hardware driver.

## Unknown short packets

Two short packets were observed:

- `AA BB 00 CC DD`
- `AA BB 64 CC DD`

Sending `AA BB 64 CC DD` before a JPEG produced no visible brightness
change and was not necessary for image transmission.

The meaning of these packets remains unknown. They must not yet be
exposed as brightness controls.