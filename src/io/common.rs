use std::fmt;

pub enum Endianness {
    LitteEndian,
    BigEndian,
}

#[derive(Debug, PartialEq)]
pub enum VolumeDataFileType {
    RAW3D,
}

// use like "Foo::from_str(input).unwrap()"
impl std::str::FromStr for VolumeDataFileType {
    type Err = ();
    fn from_str(input: &str) -> Result<VolumeDataFileType, Self::Err> {
        match input.to_lowercase().as_str() {
            "raw"  => Ok(VolumeDataFileType::RAW3D),
            "raw3d" => Ok(VolumeDataFileType::RAW3D),
            _      => Err(()),
        }
    }
}

// use like "let s: String = Foo::Quux.to_string();"
impl fmt::Display for VolumeDataFileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
