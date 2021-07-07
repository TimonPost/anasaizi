use std::fmt;

#[derive(PartialOrd, PartialEq, Eq)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Version {
        Version {
            major,
            minor,
            patch,
        }
    }

    pub fn encode(&self) -> u32 {
        ((self.major) << 22) | ((self.minor) << 12) | (self.patch)
    }

    pub fn decode(encoded_version: u32) -> Version {
        let major = ((encoded_version) >> 22) & 0x7F;
        let minor = ((encoded_version) >> 12) & 0x3FF;
        let patch = (encoded_version) & 0xFFF;

        Version {
            major,
            minor,
            patch,
        }
    }

    pub fn major(&self) -> u32 {
        self.major
    }

    pub fn minor(&self) -> u32 {
        self.minor
    }

    pub fn patch(&self) -> u32 {
        self.patch
    }
}

impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "({},{},{})", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use crate::Version;

    #[test]
    fn encode_decode_version_test() {
        let version = Version::new(1, 2, 3);
        let encoded = version.encode();
        let decoded = Version::decode(encoded);

        assert_eq!(version, decoded);
    }
}
