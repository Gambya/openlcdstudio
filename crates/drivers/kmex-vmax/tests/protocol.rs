#[cfg(test)]
mod tests {
    use kmex_vmax::protocol;

    #[test]
    fn encodes_control_packet() {
        assert_eq!(
            protocol::encode_control(100),
            [0xaa, 0xbb, 0x64, 0xcc, 0xdd],
        );
    }

    #[test]
    fn validates_complete_jpeg() {
        let jpeg = [0xff, 0xd8, 0x01, 0x02, 0xff, 0xd9];

        assert!(protocol::validate_jpeg(&jpeg).is_ok());
    }

    #[test]
    fn rejects_incomplete_jpeg() {
        let jpeg = [0xff, 0xd8, 0x01, 0x02];

        assert!(protocol::validate_jpeg(&jpeg).is_err());
    }
}
