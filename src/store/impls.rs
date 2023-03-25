use super::*;
use crate::util::serde;

#[derive(Clone, Debug)]
pub struct CollapsingLowestDenseStore {
    counts: Vec<f64>,
    offset: i32,
    min_index: i32,
    max_index: i32,
    is_collapsed: bool,
    array_length_overhead: i32,
    array_length_growth_increment: i32,
    max_num_bins: i32,
}

#[derive(Clone, Debug)]
pub struct CollapsingHighestDenseStore {
    counts: Vec<f64>,
    offset: i32,
    min_index: i32,
    max_index: i32,
    is_collapsed: bool,
    array_length_overhead: i32,
    array_length_growth_increment: i32,
    max_num_bins: i32,
}

impl CollapsingLowestDenseStore {
    pub fn new(max_num_bins: i32) -> CollapsingLowestDenseStore {
        CollapsingLowestDenseStore {
            max_num_bins,
            is_collapsed: false,
            counts: Vec::new(),
            offset: 0,
            min_index: i32::MAX,
            max_index: i32::MIN,
            array_length_growth_increment: 64,
            array_length_overhead: 6,
        }
    }

    fn normalize(&mut self, index: i32) -> i32 {
        if index < self.min_index {
            if self.is_collapsed {
                return 0;
            } else {
                self.extend_range(index, index);
                if self.is_collapsed {
                    return 0;
                }
            }
        } else if index > self.max_index {
            self.extend_range(index, index);
        }
        index - self.offset
    }

    fn get_length(&self) -> i32 {
        self.counts.len() as i32
    }

    fn extend_range(&mut self, new_min_index: i32, new_max_index: i32) {
        let new_min_index = new_min_index.min(self.min_index);
        let new_max_index = new_max_index.max(self.max_index);

        if self.is_empty() {
            let initial_length = self.get_new_length(new_min_index, new_max_index);
            if initial_length >= self.get_length() {
                self.counts.resize(initial_length as usize, 0.0);
            }
            self.offset = new_min_index;
            self.min_index = new_min_index;
            self.max_index = new_max_index;
            self.adjust(new_min_index, new_max_index);
        } else if new_min_index >= self.offset && new_max_index < self.offset + self.get_length() {
            self.min_index = new_min_index;
            self.max_index = new_max_index;
        } else {
            // To avoid shifting too often when nearing the capacity of the array, we may grow it before
            // we actually reach the capacity.
            let new_length = self.get_new_length(new_min_index, new_max_index);
            if new_length > self.get_length() {
                self.counts.resize(new_length as usize, 0.0);
            }
            self.adjust(new_min_index, new_max_index);
        }
    }

    fn adjust(&mut self, new_min_index: i32, new_max_index: i32) {
        if new_max_index - new_min_index + 1 > self.get_length() {
            // The range of indices is too wide, buckets of lowest indices need to be collapsed.

            let new_min_index = new_max_index - self.get_length() + 1;

            if new_min_index >= self.max_index {
                // There will be only one non-empty bucket.

                let total_count = self.get_total_count();
                self.reset_counts(self.min_index, self.max_index);
                self.offset = new_min_index;
                self.min_index = new_min_index;
                self.counts[0] = total_count;
            } else {
                let shift = self.offset - new_min_index;

                if shift < 0 {
                    // Collapse the buckets.
                    let collapsed_count =
                        self.get_total_count_with_range(self.min_index, new_min_index - 1);
                    self.reset_counts(self.min_index, new_min_index - 1);
                    self.counts[(new_min_index - self.offset) as usize] += collapsed_count;
                    self.min_index = new_min_index;
                    // Shift the buckets to make room for new_max_index.
                    self.shift_counts(shift);
                } else {
                    // Shift the buckets to make room for new_min_index.
                    self.shift_counts(shift);
                    self.min_index = new_min_index;
                }
            }

            self.max_index = new_max_index;

            self.is_collapsed = true;
        } else {
            self.center_counts(new_min_index, new_max_index);
        }
    }

    fn get_new_length(&self, new_min_index: i32, new_max_index: i32) -> i32 {
        let desired_length = (new_max_index as i64 - new_min_index as i64) as i32 + 1;
        i32::min(
            self.max_num_bins,
            ((desired_length + self.array_length_overhead - 1)
                / self.array_length_growth_increment
                + 1)
                * self.array_length_growth_increment,
        )
    }

    fn center_counts(&mut self, new_min_index: i32, new_max_index: i32) {
        let middle_index = new_min_index + (new_max_index - new_min_index + 1) / 2;
        let shift = self.offset + self.get_length() / 2 - middle_index;
        self.shift_counts(shift);
        self.min_index = new_min_index;
        self.max_index = new_max_index;
    }

    fn shift_counts(&mut self, shift: i32) {
        let min_array_index = self.min_index - self.offset;
        let max_array_index = self.max_index - self.offset;

        // System.arraycopy(counts, min_array_index, counts, min_array_index + shift, max_array_index - min_array_index + 1);
        self.array_copy(
            min_array_index,
            min_array_index + shift,
            max_array_index - min_array_index + 1,
        );

        if shift > 0 {
            // Arrays.fill(counts, min_array_index, min_array_index + shift, 0);
            let from = min_array_index;
            let to = min_array_index + shift;
            for index in from..to {
                self.counts[index as usize] = 0.0;
            }
        } else {
            // Arrays.fill(counts, max_array_index + 1 + shift, max_array_index + 1, 0);
            let from = max_array_index + 1 + shift;
            let to = max_array_index + 1;
            for index in from..to {
                self.counts[index as usize] = 0.0;
            }
        }

        self.offset -= shift;
    }

    fn array_copy(&mut self, src_pos: i32, dest_pos: i32, length: i32) {
        if src_pos < dest_pos {
            let mut offset = length - 1;
            while offset >= 0 {
                self.counts[(dest_pos + offset) as usize] =
                    self.counts[(src_pos + offset) as usize];
                offset -= 1;
            }
        } else if src_pos > dest_pos {
            let mut offset = 0;
            while offset < length {
                self.counts[(dest_pos + offset) as usize] =
                    self.counts[(src_pos + offset) as usize];
                offset += 1;
            }
        }
    }

    fn get_total_count_with_range(&mut self, from_index: i32, to_index: i32) -> f64 {
        if self.is_empty() {
            return 0.0;
        }

        let from_array_index = i32::max(from_index - self.offset, 0);
        let to_array_index = i32::min(to_index - self.offset, self.get_length() - 1) + 1;

        let mut total_count: f64 = 0.0;
        for array_index in from_array_index..to_array_index {
            total_count += self.counts[array_index as usize];
        }
        total_count
    }

    fn reset_counts(&mut self, from_index: i32, to_index: i32) {
        // Arrays.fill(counts, from_index - self.offset, to_index - self.offset + 1, 0);
        for index in (from_index - self.offset)..(to_index - self.offset + 1) {
            self.counts[index as usize] = 0.0;
        }
    }
}

impl Store for CollapsingLowestDenseStore {
    fn add(&mut self, index: i32, count: f64) {
        if count <= 0.0 {
            return;
        }

        let array_index = serde::i32_to_usize_exact(self.normalize(index));
        match array_index {
            Ok(index) => {
                self.counts[index] += count;
            }
            _ => {}
        }
    }

    fn add_bin(&mut self, bin: (i32, f64)) {
        if bin.1 == 0.0 {
            return;
        }
        let array_index = serde::i32_to_usize_exact(self.normalize(bin.0));
        match array_index {
            Ok(index) => {
                self.counts[index] += bin.1;
            }
            _ => {}
        }
    }

    fn clear(&mut self) {
        self.counts.fill(0.0);
        self.max_index = i32::MIN;
        self.min_index = i32::MAX;
        self.offset = 0;
        self.is_collapsed = false;
    }

    fn is_empty(&self) -> bool {
        self.max_index < self.min_index
    }

    fn get_total_count(&mut self) -> f64 {
        self.get_total_count_with_range(self.min_index, self.max_index)
    }

    fn get_min_index(&self) -> i32 {
        self.min_index
    }

    fn get_max_index(&self) -> i32 {
        self.max_index
    }

    fn get_descending_stream(&mut self) -> Vec<(i32, f64)> {
        let mut bins = Vec::new();
        let mut index = self.max_index;
        while index >= self.min_index {
            let value = self.counts[(index - self.offset) as usize];
            if value > 0.0 {
                let bin = (index, value);
                bins.push(bin);
            }
            index -= 1;
        }
        bins
    }

    fn get_ascending_stream(&mut self) -> Vec<(i32, f64)> {
        let mut bins = Vec::new();
        let mut index = self.min_index;
        while index <= self.max_index {
            let value = self.counts[(index - self.offset) as usize];
            if value > 0.0 {
                let bin = (index, value);
                bins.push(bin);
            }
            index -= 1;
        }
        bins
    }

    fn get_descending_iter(&mut self) -> StoreIter {
        StoreIter::new(
            self.min_index,
            self.max_index,
            self.offset,
            true,
            self.counts.as_slice(),
        )
    }

    fn get_ascending_iter(&mut self) -> StoreIter {
        StoreIter::new(
            self.min_index,
            self.max_index,
            self.offset,
            false,
            self.counts.as_slice(),
        )
    }

    fn foreach<F>(&mut self, mut acceptor: F)
    where
        F: FnMut(i32, f64),
    {
        if self.is_empty() {
            return;
        }

        for i in self.min_index..self.max_index {
            let value = self.counts[(i - self.offset) as usize];
            if value != 0.0 {
                acceptor(i, value);
            }
        }

        let last_count = self.counts[(self.max_index - self.offset) as usize];
        if last_count != 0.0 {
            acceptor(self.max_index, last_count);
        }
    }
}

impl CollapsingHighestDenseStore {
    pub fn new(max_num_bins: i32) -> CollapsingHighestDenseStore {
        CollapsingHighestDenseStore {
            max_num_bins,
            is_collapsed: false,
            counts: Vec::new(),
            offset: 0,
            min_index: i32::MAX,
            max_index: i32::MIN,
            array_length_growth_increment: 64,
            array_length_overhead: 6,
        }
    }

    fn normalize(&mut self, index: i32) -> i32 {
        if index > self.max_index {
            if self.is_collapsed {
                return self.get_length() - 1;
            } else {
                self.extend_range(index, index);
                if self.is_collapsed {
                    return self.get_length() - 1;
                }
            }
        } else if index < self.min_index {
            self.extend_range(index, index);
        }
        index - self.offset
    }

    fn get_length(&self) -> i32 {
        self.counts.len() as i32
    }

    fn extend_range(&mut self, new_min_index: i32, new_max_index: i32) {
        let new_min_index = new_min_index.min(self.min_index);
        let new_max_index = new_max_index.max(self.max_index);

        if self.is_empty() {
            let initial_length = self.get_new_length(new_min_index, new_max_index);
            if initial_length >= self.get_length() {
                self.counts.resize(initial_length as usize, 0.0);
            }
            self.offset = new_min_index;
            self.min_index = new_min_index;
            self.max_index = new_max_index;
            self.adjust(new_min_index, new_max_index);
        } else if new_min_index >= self.offset && new_max_index < self.offset + self.get_length() {
            self.min_index = new_min_index;
            self.max_index = new_max_index;
        } else {
            // To avoid shifting too often when nearing the capacity of the array, we may grow it before
            // we actually reach the capacity.
            let new_length = self.get_new_length(new_min_index, new_max_index);
            if new_length > self.get_length() {
                self.counts.resize(new_length as usize, 0.0);
            }
            self.adjust(new_min_index, new_max_index);
        }
    }

    fn adjust(&mut self, new_min_index: i32, new_max_index: i32) {
        if new_max_index - new_min_index + 1 > self.get_length() {
            // The range of indices is too wide, buckets of lowest indices need to be collapsed.

            let new_max_index = new_min_index + self.get_length() - 1;

            if new_max_index <= self.min_index {
                // There will be only one non-empty bucket.

                let total_count = self.get_total_count();
                self.reset_counts(self.min_index, self.max_index);
                self.offset = new_min_index;
                self.max_index = new_max_index;
                let move_index = (self.get_length() - 1) as usize;
                self.counts[move_index] = total_count;
            } else {
                let shift = self.offset - new_min_index;

                if shift > 0 {
                    // Collapse the buckets.
                    let collapsed_count =
                        self.get_total_count_with_range(new_max_index + 1, self.max_index);
                    self.reset_counts(new_max_index + 1, self.max_index);
                    self.counts[(new_max_index - self.offset) as usize] += collapsed_count;
                    self.max_index = new_max_index;
                    // Shift the buckets to make room for new_max_index.
                    self.shift_counts(shift);
                } else {
                    // Shift the buckets to make room for new_min_index.
                    self.shift_counts(shift);
                    self.max_index = new_max_index;
                }
            }

            self.min_index = new_min_index;

            self.is_collapsed = true;
        } else {
            self.center_counts(new_min_index, new_max_index);
        }
    }

    fn get_new_length(&self, new_min_index: i32, new_max_index: i32) -> i32 {
        let desired_length = (new_max_index as i64 - new_min_index as i64) as i32 + 1;
        i32::min(
            self.max_num_bins,
            ((desired_length + self.array_length_overhead - 1)
                / self.array_length_growth_increment
                + 1)
                * self.array_length_growth_increment,
        )
    }

    fn center_counts(&mut self, new_min_index: i32, new_max_index: i32) {
        let middle_index = new_min_index + (new_max_index - new_min_index + 1) / 2;
        let shift = self.offset + self.get_length() / 2 - middle_index;
        self.shift_counts(shift);
        self.min_index = new_min_index;
        self.max_index = new_max_index;
    }

    fn shift_counts(&mut self, shift: i32) {
        let min_array_index = self.min_index - self.offset;
        let max_array_index = self.max_index - self.offset;

        // System.arraycopy(counts, min_array_index, counts, min_array_index + shift, max_array_index - min_array_index + 1);
        self.array_copy(
            min_array_index,
            min_array_index + shift,
            max_array_index - min_array_index + 1,
        );

        if shift > 0 {
            // Arrays.fill(counts, min_array_index, min_array_index + shift, 0);
            let from = min_array_index;
            let to = min_array_index + shift;
            for index in from..to {
                self.counts[index as usize] = 0.0;
            }
        } else {
            // Arrays.fill(counts, max_array_index + 1 + shift, max_array_index + 1, 0);
            let from = max_array_index + 1 + shift;
            let to = max_array_index + 1;
            for index in from..to {
                self.counts[index as usize] = 0.0;
            }
        }

        self.offset -= shift;
    }

    fn array_copy(&mut self, src_pos: i32, dest_pos: i32, length: i32) {
        if src_pos < dest_pos {
            let mut offset = length - 1;
            while offset >= 0 {
                self.counts[(dest_pos + offset) as usize] =
                    self.counts[(src_pos + offset) as usize];
                offset -= 1;
            }
        } else if src_pos > dest_pos {
            let mut offset = 0;
            while offset < length {
                self.counts[(dest_pos + offset) as usize] =
                    self.counts[(src_pos + offset) as usize];
                offset += 1;
            }
        }
    }

    fn get_total_count_with_range(&mut self, from_index: i32, to_index: i32) -> f64 {
        if self.is_empty() {
            return 0.0;
        }

        let from_array_index = i32::max(from_index - self.offset, 0);
        let to_array_index = i32::min(to_index - self.offset, self.get_length() - 1) + 1;

        let mut total_count: f64 = 0.0;
        for array_index in from_array_index..to_array_index {
            total_count += self.counts[array_index as usize];
        }
        total_count
    }

    fn reset_counts(&mut self, from_index: i32, to_index: i32) {
        // Arrays.fill(counts, from_index - self.offset, to_index - self.offset + 1, 0);
        for index in (from_index - self.offset)..(to_index - self.offset + 1) {
            self.counts[index as usize] = 0.0;
        }
    }
}

impl Store for CollapsingHighestDenseStore {
    fn add(&mut self, index: i32, count: f64) {
        if count <= 0.0 {
            return;
        }

        let array_index = serde::i32_to_usize_exact(self.normalize(index));
        match array_index {
            Ok(index) => {
                self.counts[index] += count;
            }
            _ => {}
        }
    }

    fn add_bin(&mut self, bin: (i32, f64)) {
        if bin.1 == 0.0 {
            return;
        }
        let array_index = serde::i32_to_usize_exact(self.normalize(bin.0));
        match array_index {
            Ok(index) => {
                self.counts[index] += bin.1;
            }
            _ => {}
        }
    }

    fn clear(&mut self) {
        self.counts.fill(0.0);
        self.max_index = i32::MIN;
        self.min_index = i32::MAX;
        self.offset = 0;
        self.is_collapsed = false;
    }

    fn is_empty(&self) -> bool {
        self.max_index < self.min_index
    }

    fn get_total_count(&mut self) -> f64 {
        self.get_total_count_with_range(self.min_index, self.max_index)
    }

    fn get_min_index(&self) -> i32 {
        self.min_index
    }

    fn get_max_index(&self) -> i32 {
        self.max_index
    }

    fn get_descending_stream(&mut self) -> Vec<(i32, f64)> {
        let mut bins = Vec::new();
        let mut index = self.max_index;
        while index >= self.min_index {
            let value = self.counts[(index - self.offset) as usize];
            if value > 0.0 {
                let bin = (index, value);
                bins.push(bin);
            }
            index -= 1;
        }
        bins
    }

    fn get_ascending_stream(&mut self) -> Vec<(i32, f64)> {
        let mut bins = Vec::new();
        let mut index = self.min_index;
        while index <= self.max_index {
            let value = self.counts[(index - self.offset) as usize];
            if value > 0.0 {
                let bin = (index, value);
                bins.push(bin);
            }
            index -= 1;
        }
        bins
    }

    fn get_descending_iter(&mut self) -> StoreIter {
        StoreIter::new(
            self.min_index,
            self.max_index,
            self.offset,
            true,
            self.counts.as_slice(),
        )
    }

    fn get_ascending_iter(&mut self) -> StoreIter {
        StoreIter::new(
            self.min_index,
            self.max_index,
            self.offset,
            false,
            self.counts.as_slice(),
        )
    }

    fn foreach<F>(&mut self, mut acceptor: F)
    where
        F: FnMut(i32, f64),
    {
        if self.is_empty() {
            return;
        }

        for i in self.min_index..self.max_index {
            let value = self.counts[(i - self.offset) as usize];
            if value != 0.0 {
                acceptor(i, value);
            }
        }

        let last_count = self.counts[(self.max_index - self.offset) as usize];
        if last_count != 0.0 {
            acceptor(self.max_index, last_count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_add() {
        let mut store = CollapsingLowestDenseStore::new(5);
        store.add(0, 1.0);
        store.add(1, 2.0);
        store.add(2, 1.0);
        store.add(11, 1.0);
        store.add(12, 1.0);
        store.add(3, 1.0);
        store.add(4, 1.0);
        assert_eq!(8, store.get_min_index());
        assert_eq!(12, store.get_max_index());
        assert_eq!(8.0, store.get_total_count());
    }
}
