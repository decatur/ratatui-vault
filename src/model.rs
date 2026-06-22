//! Very simple binary format of an encrypted file.
//! The len fields are stored big endian.
//!
//! ```text
//!  0 1 2 3 4 5 6 7 8 ...
//! +-+-------+-------->
//! │V│ nonce │ nonce ->
//! │E│   len │
//! │R│       │
//! │S│       │
//! │I│       │
//! │O│       │
//! │N│       │
//!
//!  0 1 2 3 4 5 6 7 ...
//! +-------+------->
//! │  salt │ salt ->
//! │   len │
//!
//!  0 1 2 3 4 5 6 7 ...
//! +-------+------------->
//! │cipher │ ciphertext ->
//! │  text │
//! │   len │
//!```
//! See also <https://github.com/C2SP/C2SP/blob/main/age.md>
pub(crate) struct Model {
    pub(crate) version: u8,
    pub(crate) nonce: Vec<u8>,
    pub(crate) salt: Vec<u8>,
    pub(crate) ciphertext: Vec<u8>,
}

impl Model {
    pub(crate) fn serialize(&self) -> Vec<u8> {
        let mut buffer = vec![];
        buffer.push(self.version);

        let mut write_array = |bytes: &[u8]| {
            buffer.extend((bytes.len() as u32).to_be_bytes());
            buffer.extend(bytes.iter());
        };
        write_array(&self.nonce);
        write_array(&self.salt);
        write_array(&self.ciphertext);

        buffer
    }

    pub(crate) fn deserialize(mut bytes: &[u8]) -> Model {
        let version = if bytes[0] == 0 {
            // Version 0 did not have a version mark. This works because nonce length fits in 3 bytes.
            0
        } else {
            let version = bytes[0];
            bytes = &bytes[1..];
            version
        };

        let mut read_array = || {
            assert!(4 <= bytes.len(), "Input file has invalid content");
            let len = u32::from_be_bytes(bytes[0..4].try_into().unwrap()) as usize;
            bytes = &bytes[4..];
            assert!(len <= bytes.len(), "Input file has invalid content");
            let array = bytes[0..len].to_vec();
            bytes = &bytes[len..];
            array
        };

        let nonce = read_array();
        let salt = read_array();
        let payload = read_array();

        Model {
            version,
            nonce,
            salt,
            ciphertext: payload,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::Model;

    #[test]
    fn serde() {
        let a = Model {
            version: 1,
            nonce: vec![1, 2, 3],
            salt: vec![1, 2, 3],
            ciphertext: vec![1, 2, 3],
        };
        let buf = a.serialize();
        let b = Model::deserialize(&buf);
        assert_eq!(b.version, a.version);
        assert_eq!(b.nonce, a.nonce);
        assert_eq!(b.salt, a.salt);
        assert_eq!(b.ciphertext, a.ciphertext);
    }
}
