use crate::{Base64, Error, OutRef, STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};

use rand::RngCore;

fn rand_bytes(n: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; n];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes
}

#[test]
fn test_forgiving() {
    let inputs = ["ab", "abc", "abcd"];
    let outputs: &[&[u8]] = &[&[105], &[105, 183], &[105, 183, 29]];

    for i in 0..inputs.len() {
        let (src, expected) = (inputs[i], outputs[i]);
        let mut buf = src.to_owned().into_bytes();
        let ans = crate::forgiving_decode_inplace(&mut buf).unwrap();
        assert_eq!(ans, expected, "src = {:?}, expected = {:?}", src, expected);
    }
}

#[cfg(miri)]
use std::io::Write as _;

macro_rules! dbgmsg {
    ($($fmt:tt)*) => {
        println!($($fmt)*);
        #[cfg(miri)]
        std::io::stdout().flush().unwrap();
    };
}

#[allow(clippy::type_complexity)]
fn safety_unit_test(
    check: for<'s> fn(&'_ Base64, &'s [u8]) -> Result<(), Error>,
    encode: for<'s, 'd> fn(&'_ Base64, &'s [u8], OutRef<'d, [u8]>) -> &'d mut [u8],
    decode: for<'s, 'd> fn(&'_ Base64, &'s [u8], OutRef<'d, [u8]>) -> Result<&'d mut [u8], Error>,
    decode_inplace: for<'b> fn(&'_ Base64, &'b mut [u8]) -> Result<&'b mut [u8], Error>,
) {
    // canonicity tests
    // <https://eprint.iacr.org/2022/361>
    {
        let test_vectors = [
            ("SGVsbG8=", Some("Hello")),
            ("SGVsbG9=", None),
            ("SGVsbG9", None),
            ("SGVsbA==", Some("Hell")),
            ("SGVsbA=", None),
            ("SGVsbA", None),
            ("SGVsbA====", None),
        ];

        for (encoded, expected) in test_vectors {
            let base64 = STANDARD;

            let is_valid = check(&base64, encoded.as_bytes()).is_ok();
            let result: _ = base64.decode_type::<Vec<u8>>(encoded.as_bytes());

            assert_eq!(is_valid, result.is_ok());
            match expected {
                Some(expected) => assert_eq!(&*result.unwrap(), expected.as_bytes()),
                None => assert!(result.is_err(), "expected = {expected:?}"),
            }
        }
    }

    println!();
    for n in 0..128 {
        dbgmsg!("n = {}", n);
        let bytes = rand_bytes(n);

        let test_config = [
            STANDARD,        //
            URL_SAFE,        //
            STANDARD_NO_PAD, //
            URL_SAFE_NO_PAD, //
        ];
        let base_config = [
            base64::STANDARD,
            base64::URL_SAFE,
            base64::STANDARD_NO_PAD,
            base64::URL_SAFE_NO_PAD,
        ];

        for (base64, config) in test_config.into_iter().zip(base_config.into_iter()) {
            dbgmsg!("base64 = {:?}", base64);

            let encoded = base64::encode_config(&bytes, config);
            let encoded = encoded.as_bytes();
            assert!(check(&base64, encoded).is_ok());

            {
                let mut buf = vec![0u8; base64.encoded_length(n)];
                let buf = OutRef::from_slice(&mut buf);
                let ans = encode(&base64, &bytes, buf);
                assert_eq!(ans, encoded);
                assert!(check(&base64, ans).is_ok());
                dbgmsg!("encoding ... ok");
            }

            {
                let mut buf = encoded.to_owned();
                let ans = decode_inplace(&base64, &mut buf).unwrap();
                assert_eq!(ans, bytes);
                dbgmsg!("decoding inplace ... ok");
            }

            {
                let mut buf = vec![0u8; n];
                let buf = OutRef::from_slice(&mut buf);
                let ans = decode(&base64, encoded, buf).unwrap();
                assert_eq!(ans, bytes);
                dbgmsg!("decoding ... ok");
            }
        }
    }
}

#[test]
fn test_safety() {
    safety_unit_test(Base64::check, Base64::encode, Base64::decode, Base64::decode_inplace);
}

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
#[wasm_bindgen_test::wasm_bindgen_test]
fn wasm() {
    safety_unit_test(Base64::check, Base64::encode, Base64::decode, Base64::decode_inplace);
}
