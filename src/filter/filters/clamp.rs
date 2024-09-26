use clap::Parser;
use num_traits::AsPrimitive;
use rayon::iter::{IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use zarrs::{
    array::{data_type::UnsupportedDataTypeError, Array, DataType},
    array_subset::ArraySubset,
    filesystem::FilesystemStore,
};

use crate::{
    filter::{
        calculate_chunk_limit, filter_error::FilterError, filter_traits::FilterTraits,
        FilterArguments, FilterCommonArguments,
    },
    progress::{Progress, ProgressCallback},
};

#[derive(Debug, Clone, Parser, Serialize, Deserialize)]
pub struct ClampArguments {
    /// Minimum.
    pub min: f64,
    /// Maximum.
    pub max: f64,
}

impl FilterArguments for ClampArguments {
    fn name(&self) -> String {
        "clamp".to_string()
    }

    fn init(
        &self,
        common_args: &FilterCommonArguments,
    ) -> Result<Box<dyn FilterTraits>, FilterError> {
        Ok(Box::new(Clamp::new(
            self.min,
            self.max,
            *common_args.chunk_limit(),
        )))
    }
}

pub struct Clamp {
    min: f64,
    max: f64,
    chunk_limit: Option<usize>,
}

impl Clamp {
    pub fn new(min: f64, max: f64, chunk_limit: Option<usize>) -> Self {
        Self {
            min,
            max,
            chunk_limit,
        }
    }

    pub fn apply_elements_inplace<T>(&self, elements: &mut [T]) -> Result<(), FilterError>
    where
        T: bytemuck::Pod + Copy + Send + Sync + PartialOrd,
        f64: AsPrimitive<T>,
    {
        let min: T = self.min.as_();
        let max: T = self.max.as_();
        elements
            .par_iter_mut()
            .for_each(|value| *value = num_traits::clamp(*value, min, max));
        Ok(())
    }
}

impl FilterTraits for Clamp {
    fn is_compatible(
        &self,
        chunk_input: &zarrs::array::ChunkRepresentation,
        chunk_output: &zarrs::array::ChunkRepresentation,
    ) -> Result<(), FilterError> {
        for data_type in [chunk_input.data_type(), chunk_output.data_type()] {
            match data_type {
                DataType::Bool
                | DataType::Int8
                | DataType::Int16
                | DataType::Int32
                | DataType::Int64
                | DataType::UInt8
                | DataType::UInt16
                | DataType::UInt32
                | DataType::UInt64
                | DataType::Float16
                | DataType::Float32
                | DataType::Float64
                | DataType::BFloat16 => {}
                _ => Err(UnsupportedDataTypeError::from(data_type.to_string()))?,
            };
        }
        Ok(())
    }

    fn memory_per_chunk(
        &self,
        chunk_input: &zarrs::array::ChunkRepresentation,
        chunk_output: &zarrs::array::ChunkRepresentation,
    ) -> usize {
        chunk_input.fixed_element_size().unwrap() + chunk_output.fixed_element_size().unwrap()
    }

    fn apply(
        &self,
        input: &Array<FilesystemStore>,
        output: &mut Array<FilesystemStore>,
        progress_callback: &ProgressCallback,
    ) -> Result<(), FilterError> {
        assert_eq!(output.shape(), input.shape());

        let chunks = ArraySubset::new_with_shape(output.chunk_grid_shape().unwrap());
        let progress = Progress::new(chunks.num_elements_usize(), progress_callback);

        let chunk_limit = if let Some(chunk_limit) = self.chunk_limit {
            chunk_limit
        } else {
            calculate_chunk_limit(self.memory_per_chunk(
                &input.chunk_array_representation(&vec![0; input.dimensionality()])?,
                &output.chunk_array_representation(&vec![0; input.dimensionality()])?,
            ))?
        };

        let indices = chunks.indices();
        rayon_iter_concurrent_limit::iter_concurrent_limit!(
            chunk_limit,
            indices,
            try_for_each,
            |chunk_indices: Vec<u64>| {
                macro_rules! apply_input {
                    ( $t_out:ty, [$( ( $data_type_in:ident, $t_in:ty ) ),* ]) => {
                        match input.data_type() {
                            $(DataType::$data_type_in => {
                                let input_output_subset = output.chunk_subset_bounded(&chunk_indices).unwrap();
                                let mut elements_in =
                                    progress.read(|| input.retrieve_array_subset_elements::<$t_in>(&input_output_subset))?;
                                progress.process(|| self.apply_elements_inplace::<$t_in>(&mut elements_in))?;

                                // macro_rules! apply_input_inner {
                                //     ($t_in, $t_in) => {{
                                //         progress.write(|| {
                                //             output.store_array_subset_elements::<$t_in>(&input_output_subset, elements_in)
                                //         })?;
                                //     }}
                                //     ($t_in, $t_out) => {{
                                        let elements_out = elements_in.iter().map(|v| v.as_()).collect::<Vec<_>>();
                                        drop(elements_in);
                                        progress.write(|| {
                                            output.store_array_subset_elements::<$t_out>(&input_output_subset, &elements_out)
                                        })?;
                                //     }}
                                // }
                                // apply_input_inner($t_in, $t_out)
                            } ,)*
                            _ => panic!()
                        }
                    };
                }
                macro_rules! apply_output {
                    ([$( ( $data_type_out:ident, $type_out:ty ) ),* ]) => {
                            match output.data_type() {
                                $(
                                    DataType::$data_type_out => {
                                        apply_input!($type_out, [
                                            (Bool, u8),
                                            (Int8, i8),
                                            (Int16, i16),
                                            (Int32, i32),
                                            (Int64, i64),
                                            (UInt8, u8),
                                            (UInt16, u16),
                                            (UInt32, u32),
                                            (UInt64, u64),
                                            (BFloat16, half::bf16),
                                            (Float16, half::f16),
                                            (Float32, f32),
                                            (Float64, f64)
                                        ]
                                    )}
                                ,)*
                                _ => panic!()
                            }
                        };
                    }
                apply_output!([
                    (Bool, u8),
                    (Int8, i8),
                    (Int16, i16),
                    (Int32, i32),
                    (Int64, i64),
                    (UInt8, u8),
                    (UInt16, u16),
                    (UInt32, u32),
                    (UInt64, u64),
                    (BFloat16, half::bf16),
                    (Float16, half::f16),
                    (Float32, f32),
                    (Float64, f64)
                ]);

                progress.next();
                Ok::<_, FilterError>(())
            }
        )?;

        Ok(())
    }
}
