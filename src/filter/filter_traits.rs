use zarrs::{
    array::{Array, ArrayBuilder, ArrayShape, ChunkRepresentation, DataType, FillValue},
    filesystem::FilesystemStore,
};

use crate::{
    convert_fill_value, get_array_builder_reencode, progress::ProgressCallback, ZarrReencodingArgs,
};

use super::filter_error::FilterError;

pub trait FilterTraits {
    /// Checks if the input and output are compatible.
    fn is_compatible(
        &self,
        chunk_input: &ChunkRepresentation,
        chunk_output: &ChunkRepresentation,
    ) -> Result<(), FilterError>;

    /// Returns the memory overhead per chunk.
    ///
    /// This can be used to automatically constrain the number of concurrent chunks based on the amount of available memory.
    fn memory_per_chunk(
        &self,
        chunk_input: &ChunkRepresentation,
        chunk_output: &ChunkRepresentation,
    ) -> usize;

    /// Returns an [`ArrayShape`] if the filter changes the array shape.
    #[allow(unused_variables)]
    fn output_shape(&self, array_input: &Array<FilesystemStore>) -> Option<ArrayShape> {
        None
    }

    /// Returns a [`DataType`] and [`FillValue`] if the filter changes the data type.
    #[allow(unused_variables)]
    fn output_data_type(
        &self,
        array_input: &Array<FilesystemStore>,
    ) -> Option<(DataType, FillValue)> {
        None
    }

    fn output_array_builder(
        &self,
        array_input: &Array<FilesystemStore>,
        reencoding_args: &ZarrReencodingArgs,
    ) -> ArrayBuilder {
        let mut reencoding_args = reencoding_args.clone();

        if let Some(data_type) = &reencoding_args.data_type {
            // Use explicitly set data type
            let data_type = DataType::from_metadata(data_type).unwrap();
            if reencoding_args.fill_value.is_none() {
                // Convert fill value to new data type if no explicit fill value set
                reencoding_args.fill_value =
                    Some(data_type.metadata_fill_value(&convert_fill_value(
                        array_input.data_type(),
                        array_input.fill_value(),
                        &data_type,
                    )));
            }
            reencoding_args.data_type = Some(data_type.metadata());
        } else if let Some((data_type, fill_value)) = self.output_data_type(array_input) {
            // Use auto data type/fill value from filter, if defined
            reencoding_args.data_type = Some(data_type.metadata());
            reencoding_args.fill_value = Some(data_type.metadata_fill_value(&fill_value));
        }

        get_array_builder_reencode(
            &reencoding_args,
            array_input,
            self.output_shape(array_input),
        )
    }

    fn apply(
        &self,
        input: &Array<FilesystemStore>,
        output: &mut Array<FilesystemStore>,
        progress_callback: &ProgressCallback,
    ) -> Result<(), FilterError>;
}

impl<T: FilterTraits + ?Sized> FilterTraits for Box<T> {
    #[inline]
    fn apply(
        &self,
        input: &Array<FilesystemStore>,
        output: &mut Array<FilesystemStore>,
        progress_callback: &ProgressCallback,
        // progress_callback: CB,
    ) -> Result<(), FilterError> {
        (**self).apply(input, output, progress_callback)
    }

    #[inline]
    fn is_compatible(
        &self,
        chunk_input: &ChunkRepresentation,
        chunk_output: &ChunkRepresentation,
    ) -> Result<(), FilterError> {
        (**self).is_compatible(chunk_input, chunk_output)
    }

    #[inline]
    fn memory_per_chunk(
        &self,
        chunk_input: &ChunkRepresentation,
        chunk_output: &ChunkRepresentation,
    ) -> usize {
        (**self).memory_per_chunk(chunk_input, chunk_output)
    }

    #[inline]
    fn output_array_builder(
        &self,
        array_input: &Array<FilesystemStore>,
        reencoding_args: &ZarrReencodingArgs,
    ) -> ArrayBuilder {
        (**self).output_array_builder(array_input, reencoding_args)
    }

    #[inline]
    fn output_data_type(
        &self,
        array_input: &Array<FilesystemStore>,
    ) -> Option<(DataType, FillValue)> {
        (**self).output_data_type(array_input)
    }

    #[inline]
    fn output_shape(&self, array_input: &Array<FilesystemStore>) -> Option<ArrayShape> {
        (**self).output_shape(array_input)
    }
}
