use crate::{octree::tables::SubCellIndex, ADDRESS_SIZE};

/// Stores the address of a cell in an parent cell children array.
#[derive(Debug)]
pub struct Address {
    /// @brief Storing the actuall address
    raw_address: Vec<Option<SubCellIndex>>,
}

impl Address {
    pub fn new() -> Self {
        Self {
            raw_address: vec![None; ADDRESS_SIZE as usize],
        }
    }

    pub fn set(
        &mut self,
        parent_address_ptr: &Vec<Option<SubCellIndex>>,
        pos_in_parent: Option<SubCellIndex>,
    ) {
        for i in 0..ADDRESS_SIZE {
            // Copy the parent's address
            match parent_address_ptr[i] {
                Some(_) => {
                    // Add the new position in parent to the address
                    // Avoid any further assignments
                    self.raw_address[i] = parent_address_ptr[i];
                }
                None => {
                    self.raw_address[i] = pos_in_parent;
                    break;
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.raw_address.clear();
        self.raw_address.resize(ADDRESS_SIZE, None);
    }

    pub fn populate_address(&mut self, raw_address: &Vec<Option<SubCellIndex>>) {
        self.raw_address = raw_address.to_vec();
    }

    /// @brief Retrieves the address as a single uint
    /// We use an unsigned integers (uint) which is 32-bit on
    /// all* platforms and can store a range of 4 billion
    /// which is 10 digits. Thus can safely be used for up to
    /// depth 9 (2^9 == 512 samples). Thus max address == 7777_7777
    pub fn get_formatted(&self) -> usize {
        self.format_address()
    }

    pub fn get_raw(&self) -> &Vec<Option<SubCellIndex>> {
        &self.raw_address
    }

    fn format_address(&self) -> usize {
        let mut formatted_address: usize = 0;
        for i in (0..ADDRESS_SIZE).rev() {
            match self.raw_address[i] {
                Some(value) => {
                    formatted_address +=
                        (value as usize as f32 * 10f32.powf(i as f32)).ceil() as usize;
                }
                None => {
                    formatted_address += (8 as f32 * 10f32.powf(i as f32)).ceil() as usize;
                }
            }
        }

        formatted_address
    }
}
