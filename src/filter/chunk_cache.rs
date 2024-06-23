use std::sync::{Arc, Mutex};

use zarrs::{array::{Array, ArrayError}, array_subset::ArraySubset, storage::store::FilesystemStore};

pub type ChunkCache = lru::LruCache<Vec<u64>, Arc<Vec<u8>>>;

pub fn retrieve_array_subset_ndarray_cached<T: bytemuck::Pod + Default + std::fmt::Debug>(
    array: &Array<FilesystemStore>,
    cache: Arc<Mutex<ChunkCache>>,
    subset: &ArraySubset,
) -> Result<ndarray::ArrayD<T>, ArrayError> {
    // Find all chunks in range
    let chunks = array.chunks_in_array_subset(subset)?;
    let mut output = ndarray::ArrayD::<T>::default(
        subset
            .shape()
            .iter()
            .map(|f| *f as usize)
            .collect::<Vec<_>>(),
    );
    if let Some(chunks) = chunks {
        for chunk_indices in chunks
            .indices()
            // .into_par_iter()
            .into_iter()
        {
            let chunk_subset = array.chunk_subset(&chunk_indices)?;
            let overlap = unsafe { subset.overlap_unchecked(&chunk_subset) };
            let chunk_subset_in_array_subset =
                unsafe { overlap.relative_to_unchecked(subset.start()) };
            let array_subset_in_chunk_subset =
                unsafe { overlap.relative_to_unchecked(chunk_subset.start()) };

            // Try and get a chunk from cache
            let mut cache = cache.lock().unwrap();
            let bytes = if let Some(bytes) = cache.get(&chunk_indices) {
                // println!("{chunk_indices:?} CACHED");
                unsafe {
                    array_subset_in_chunk_subset.extract_bytes_unchecked(
                        bytes,
                        chunk_subset.shape(),
                        array.data_type().size(),
                    )
                }
            } else {
                // println!("{chunk_indices:?} LOAD");
                let bytes = array.retrieve_chunk(&chunk_indices)?;
                let elements_subset = unsafe {
                    array_subset_in_chunk_subset.extract_bytes_unchecked(
                        &bytes,
                        chunk_subset.shape(),
                        array.data_type().size(),
                    )
                };
                cache.put(chunk_indices, Arc::new(bytes));
                elements_subset
            };
            let elements = zarrs::bytemuck::try_cast_slice(&bytes).unwrap();

            let output_slice = output.as_slice_mut().unwrap();
            let mut decoded_offset = 0;
            for (array_subset_element_index, num_elements) in unsafe {
                chunk_subset_in_array_subset
                    .contiguous_linearised_indices_unchecked(subset.shape())
                    .into_iter()
            } {
                let output_offset = usize::try_from(array_subset_element_index).unwrap();
                let length = usize::try_from(num_elements).unwrap();
                debug_assert!((output_offset + length) <= output_slice.len());
                debug_assert!((decoded_offset + length) <= elements.len());
                output_slice[output_offset..output_offset + length]
                    .copy_from_slice(&elements[decoded_offset..decoded_offset + length]);
                decoded_offset += length;
            }
            // cache.lock().unwrap().insert(chunk, elements);
        }
    }
    Ok(output)
}
