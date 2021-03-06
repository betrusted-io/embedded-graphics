use nom::*;

#[derive(Debug, PartialEq)]
pub struct RawPacket<'a> {
    /// Number of pixels of this packet
    pub num_pixels: u8,

    /// Pixel data in this packet, up to 32 bits (4 bytes) per pixel
    pub pixel_data: &'a [u8],
}

impl<'a> RawPacket<'a> {
    /// Get the number of bytes in this packet
    pub fn len(&self) -> usize {
        self.pixel_data.len()
    }
}

named_args!(pub raw_packet(bytes_per_pixel: u8)<&[u8], RawPacket>,
    do_parse!(
        num_pixels: bits!(
            preceded!(
                // 0x00 = raw packet, 0x01 = RLE packet
                tag_bits!(u8, 1, 0x00),
                // Run length is encoded as 0 = 1 pixel, 1 = 2 pixels, etc, hence this offset
                map!(take_bits!(u8, 7), |len| len + 1)
            )
        ) >>
        pixel_data: take!(num_pixels * bytes_per_pixel) >>
        (
            RawPacket {
                num_pixels,
                pixel_data
            }
        )
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let input = [
            // 2 pixels worth of RAW data
            0b0000_0001,
            // 32BPP pixel
            0xAA,
            0xBB,
            0xCC,
            0xDD,
            // 32BPP pixel
            0x11,
            0x22,
            0x33,
            0x44,
        ];

        let (remaining, packet) = raw_packet(&input, 4).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(
            packet,
            RawPacket {
                num_pixels: 2,
                pixel_data: &[
                    0xAA, 0xBB, 0xCC, 0xDD, //
                    0x11, 0x22, 0x33, 0x44, //
                ]
            }
        );
    }

    #[test]
    fn ignore_rle_packet() {
        let input = [
            // 2 pixels worth of RLE data
            0b1000_0001,
            // 32BPP pixel
            0xAA,
            0xBB,
            0xCC,
            0xDD,
        ];

        let result = raw_packet(&input, 4);

        assert!(result.is_err());
    }

    #[test]
    fn stop_at_packet_end() {
        let input = [
            // 2 pixels worth of non-RLE data
            0b0000_0001,
            // 32BPP pixel
            0xAA,
            0xBB,
            0xCC,
            0xDD,
            // 32BPP pixel
            0x11,
            0x22,
            0x33,
            0x44,
            // 32BPP pixel (extra, invalid)
            0x55,
            0x66,
            0x77,
            0x88,
        ];

        let (remaining, packet) = raw_packet(&input, 4).unwrap();

        assert_eq!(remaining, &[0x55, 0x66, 0x77, 0x88]);
        assert_eq!(
            packet,
            RawPacket {
                num_pixels: 2,
                pixel_data: &[
                    0xAA, 0xBB, 0xCC, 0xDD, //
                    0x11, 0x22, 0x33, 0x44, //
                ]
            }
        );
    }
}
