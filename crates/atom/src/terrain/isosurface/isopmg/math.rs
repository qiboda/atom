struct QEFNormal<T, N: i32> {
    pub data: [T; (N + 1) * (N + 2) / 2],
}

impl<T, N> QEFNormal {
    pub fn combine_self(&self, eqn: &QEFNormal) {
        let mut index = 0;
        for i in 0..N + 1 {
            for j in i..N + 1 {
                self.data[index] += eqn.data[i] * eqn.data[j];
                index += 1;
            }
        }
    }
}
