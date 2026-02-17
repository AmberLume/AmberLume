use crate::resources::dynamic::resource_backend::ResourceKey;

pub struct Hasher {
    handle: blake3::Hasher,
}

impl Hasher {
    pub fn new() -> Self {
        Self {
            handle: blake3::Hasher::new(),
        }
    }

    pub fn finalize(&mut self) -> ResourceKey {
        let digest = self.handle.finalize();
        let mut out = [0u8; 16];
        out.copy_from_slice(&digest.as_bytes()[..16]);
        out
    }

    #[inline]
    pub fn hash_u32(&mut self, value: u32) {
        self.handle.update(&value.to_le_bytes());
    }

    #[inline]
    pub fn hash_i32(&mut self, value: i32) {
        self.handle.update(&value.to_le_bytes());
    }

    #[inline]
    pub fn hash_bool(&mut self, value: bool) {
        self.handle.update(&[value as u8]);
    }

    #[inline]
    pub fn hash_string(&mut self, value: &String) {
        self.hash_u32(value.len() as u32);

        self.handle.update(value.as_bytes());
    }
}
