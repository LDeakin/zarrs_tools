use ndarray::{ArrayD, IxDyn, SliceInfo, SliceInfoElem};
use zarrs::array_subset::ArraySubset;

#[derive(Debug)]
pub struct ArraySubsetOverlap {
    subset_src_overlap: ArraySubset,
    subset_dst_in_src: ArraySubset,
}

impl ArraySubsetOverlap {
    pub fn new(shape_src: &[u64], subset_src: &ArraySubset, overlap: &[u64]) -> Self {
        let subset_overlap_start = subset_src
            .start()
            .iter()
            .zip(overlap)
            .map(|(f, o)| f.saturating_sub(*o))
            .collect::<Vec<_>>();
        let subset_overlap_end = itertools::izip!(&subset_src.end_exc(), shape_src, overlap)
            .map(|(e, &s, o)| std::cmp::min(*e + o, s))
            .collect();
        let offset_dst = subset_src
            .start()
            .iter()
            .zip(subset_overlap_start.iter())
            .map(|(start, overlap_start)| start - overlap_start)
            .collect::<Vec<u64>>();
        let subset_src_overlap =
            ArraySubset::new_with_start_end_exc(subset_overlap_start, subset_overlap_end).unwrap();
        let subset_dst_in_src = unsafe {
            ArraySubset::new_with_start_shape_unchecked(offset_dst, subset_src.shape().to_vec())
        };

        ArraySubsetOverlap {
            subset_src_overlap,
            subset_dst_in_src,
        }
    }

    pub fn subset_input(&self) -> &ArraySubset {
        &self.subset_src_overlap
    }

    pub fn extract_subset<T: Clone>(&self, array: &ArrayD<T>) -> ArrayD<T> {
        let slices: Vec<SliceInfoElem> = std::iter::zip(
            self.subset_dst_in_src.start(),
            self.subset_dst_in_src.end_exc(),
        )
        .map(|(&s, e)| SliceInfoElem::from(s as usize..e as usize))
        .collect::<Vec<_>>();
        array
            .slice(SliceInfo::<_, IxDyn, IxDyn>::try_from(slices).unwrap())
            .to_owned()
    }
}
