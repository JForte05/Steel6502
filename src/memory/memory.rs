pub enum AccessError{
    OutOfRange(usize),
}

pub trait Indexed{
    fn len(&self) -> usize;
}
pub trait ReadableBuffer: Indexed{
    fn peek(&self, idx: usize) -> Result<u8, AccessError>;
    fn read(&mut self, idx: usize) -> Result<u8, AccessError>;
}
pub trait WritableBuffer: Indexed{
    fn write(&mut self, idx: usize, val: u8) -> Result<(), AccessError>;
}


/// A chunk of memory of a fixed size, 256 bytes.
pub struct MemoryPage{
    buffer: [u8; 256]
}
impl MemoryPage{
    pub const SIZE: usize = 256;

    pub fn new() -> Self{
        Self { buffer: [0u8; Self::SIZE] }
    }
    #[inline]
    fn check_idx(idx: usize) -> Result<usize, AccessError>{
        if idx >= Self::SIZE{
            return Err(AccessError::OutOfRange(idx));
        }

        Ok(idx)
    }

    #[inline]
    pub fn read_unchecked(&mut self, idx: u8) -> u8{
        self.buffer[idx as usize]
    }
    #[inline]
    pub fn peek_unchecked(&self, idx: u8) -> u8{
        self.buffer[idx as usize]
    }

    #[inline]
    pub fn write_unchecked(&mut self, idx: u8, val: u8){
        self.buffer[idx as usize] = val;
    }
}
impl Indexed for MemoryPage{
    fn len(&self) -> usize {
        Self::SIZE
    }
}
impl ReadableBuffer for MemoryPage{
    #[inline]
    fn peek(&self, idx: usize) -> Result<u8, AccessError> {
        Ok(self.buffer[Self::check_idx(idx)?])
    }

    #[inline]
    fn read(&mut self, idx: usize) -> Result<u8, AccessError> {
        if idx >= Self::SIZE {
            return Err(AccessError::OutOfRange(idx));
        }

        Ok(self.buffer[idx])
    }
}
impl WritableBuffer for MemoryPage{
    fn write(&mut self, idx: usize, val: u8) -> Result<(), AccessError> {
        if idx >= Self::SIZE {
            return Err(AccessError::OutOfRange(idx));
        }

        self.buffer[idx] = val;
        Ok(())
    }
}

pub struct RAMSegment{
    pages: Vec<MemoryPage>,
    size_bytes: usize
}
impl RAMSegment{
    fn new(num_pages: usize) -> Self{
        Self { 
            pages: (0..num_pages).map(|_| MemoryPage::new()).collect(), 
            size_bytes: MemoryPage::SIZE * num_pages 
        }
    }

    fn idx_split(global_idx: usize) -> (usize, u8){
        let page_index: usize = global_idx >> 8;
        let offset: u8 = (global_idx & 0xff) as u8;

        (page_index, offset)
    }
    fn check_idx(&self, idx: usize) -> Result<(usize, u8), AccessError>{
        let idx_result = Self::idx_split(idx);
        if idx_result.0 >= self.pages.len(){
            return Err(AccessError::OutOfRange(idx));
        }

        Ok(idx_result)
    }
}
impl Indexed for RAMSegment{
    fn len(&self) -> usize {
        self.size_bytes
    }
}
impl ReadableBuffer for RAMSegment{
    fn peek(&self, idx: usize) -> Result<u8, AccessError> {
        let (page, offset) = self.check_idx(idx)?;

        Ok(self.pages[page].peek_unchecked(offset))
    }
    fn read(&mut self, idx: usize) -> Result<u8, AccessError> {
        let (page, offset) = self.check_idx(idx)?;

        Ok(self.pages[page].read_unchecked(offset))
    }
}
impl WritableBuffer for RAMSegment{
    fn write(&mut self, idx: usize, val: u8) -> Result<(), AccessError> {
        let (page, offset) = self.check_idx(idx)?;

        self.pages[page].write_unchecked(offset, val);
        Ok(())
    }
}

pub struct ROMSegment{
    pages: Vec<MemoryPage>,
    size_bytes: usize
}
impl ROMSegment{
    fn new(num_pages: usize) -> Self{
        Self { 
            pages: (0..num_pages).map(|_| MemoryPage::new()).collect(), 
            size_bytes: MemoryPage::SIZE * num_pages 
        }
    }

    fn idx_split(global_idx: usize) -> (usize, u8){
        let page_index: usize = global_idx >> 8;
        let offset: u8 = (global_idx & 0xff) as u8;

        (page_index, offset)
    }
    fn check_idx(&self, idx: usize) -> Result<(usize, u8), AccessError>{
        let idx_result = Self::idx_split(idx);
        if idx_result.0 >= self.pages.len(){
            return Err(AccessError::OutOfRange(idx));
        }

        Ok(idx_result)
    }
}
impl Indexed for ROMSegment{
    fn len(&self) -> usize {
        self.size_bytes
    }
}
impl ReadableBuffer for ROMSegment{
    fn peek(&self, idx: usize) -> Result<u8, AccessError> {
        let (page, offset) = self.check_idx(idx)?;

        Ok(self.pages[page].peek_unchecked(offset))
    }
    fn read(&mut self, idx: usize) -> Result<u8, AccessError> {
        let (page, offset) = self.check_idx(idx)?;

        Ok(self.pages[page].read_unchecked(offset))
    }
}