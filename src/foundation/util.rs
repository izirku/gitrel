/// Given a vector of string vectors, return a vector of max lengths of strings
/// in these sub-vectors.
pub fn svec2_row_maj_max_lens(vecs: &[Vec<String>]) -> Vec<usize> {
    // let mut lens = Vec::with_capacity(vecs.len());
    // for sub_vec in vecs {
    //     let sub_len = sub_vec.iter().map(|val| val.len()).max().unwrap_or_default();
    //     lens.push(sub_len);
    // }
    // lens
    vecs.iter()
        .map(|sub_vec| {
            sub_vec
                .iter()
                .map(|val| val.len())
                .max()
                .unwrap_or_default()
        })
        .collect()
}

/// Returns a vector of max lengths of strings of sub-vectors over a transpose of`vecs`.
pub fn svec2_col_maj_max_lens_unchecked(vecs: &[Vec<String>]) -> Vec<usize> {
    let mut transposed = vec![Vec::with_capacity(vecs.len()); vecs[0].len()];
    for row in vecs {
        for (idx, v) in row.iter().enumerate() {
            transposed[idx].push(v.len());
        }
    }

    transposed
        .into_iter()
        .map(|sub_vec| sub_vec.into_iter().max().unwrap())
        .collect()
}
