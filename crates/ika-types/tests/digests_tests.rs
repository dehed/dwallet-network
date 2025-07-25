// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: BSD-3-Clause-Clear

use std::str::FromStr;

use ika_types::digests::{
    DWalletCheckpointContentsDigest, DWalletCheckpointMessageDigest, MessageDigest,
};

macro_rules! define_digest_test {
    ($name:ident, $ty:ty) => {
        #[test]
        fn $name() {
            let invalid_b58 = "$$%";
            let short_b58 = "AAAA";
            let good_b58 = "DMBdBZnpYR4EeTXzXL8A6BtVafqGjAWGsFZhP2zJYmXU";
            let good_digest_arr = [
                0xb7u8, 0x77, 0xdf, 0x27, 0xcc, 0x44, 0xdc, 0x04, 0x7e, 0xea, 0xe8, 0x92, 0x6a,
                0xf9, 0x62, 0x0c, 0xaa, 0xd1, 0x62, 0xcb, 0xf3, 0x4d, 0x9a, 0xe1, 0xb1, 0xd8, 0xa9,
                0x65, 0x33, 0x74, 0x4f, 0xdf,
            ];

            let invalid_digest = <$ty>::from_str(invalid_b58);
            let short_digest = <$ty>::from_str(short_b58);
            let good_digest = <$ty>::from_str(good_b58);

            assert!(invalid_digest.is_err());
            assert!(short_digest.is_err());
            assert_eq!(good_digest.unwrap(), <$ty>::new(good_digest_arr));
        }
    };
}

define_digest_test!(
    test_dwallet_checkpoint_contents_digest_from_str,
    DWalletCheckpointContentsDigest
);

define_digest_test!(
    test_dwallet_checkpoint_digest_from_str,
    DWalletCheckpointMessageDigest
);

define_digest_test!(test_transaction_digest_from_str, MessageDigest);
