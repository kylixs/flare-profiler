use super::native::jvmti_native::*;

///
/// Represents a Java API version structure.
///
#[derive(Eq, PartialEq, Debug)]
pub struct VersionNumber {
    pub major_version: u16,
    pub minor_version: u8,
    pub micro_version: u8
}

impl VersionNumber {

    ///
    /// Parse an unsigned 32-bit integer as a Java API version number
    ///
    pub fn from_u32(version: &u32) -> VersionNumber {
        let major_version = ((version & JVMTI_VERSION_MASK_MAJOR) >> JVMTI_VERSION_SHIFT_MAJOR) as u16;
        let minor_version = ((version & JVMTI_VERSION_MASK_MINOR) >> JVMTI_VERSION_SHIFT_MINOR) as u8;
        let micro_version = ((version & JVMTI_VERSION_MASK_MICRO) >> JVMTI_VERSION_SHIFT_MICRO) as u8;

        VersionNumber {
            major_version: major_version,
            minor_version: minor_version,
            micro_version: micro_version
        }
    }

    /// Return a magic version number that is most likely not used anywhere else to indicate an unknown
    /// version number (for cases where the version number cannot be precisely determined)
    pub fn unknown() -> VersionNumber {
        VersionNumber { major_version: 0x7FFF, minor_version: 0x8F, micro_version: 0x9F }
    }
}
