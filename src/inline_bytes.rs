#[derive(Clone, Copy)]
pub struct InlineBytes<const N: usize> {
    data: [u8; N],
    len: u8,
}

impl<const N: usize> InlineBytes<N> {
    #[inline]
    pub fn new(data: [u8; N], len: u8) -> Self {
        Self { data, len }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.len()]
    }
}

impl<const N: usize> AsRef<[u8]> for InlineBytes<N> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}
