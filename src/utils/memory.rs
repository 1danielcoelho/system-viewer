pub unsafe fn any_slice_to_u8_slice<T>(input: &[T]) -> &[u8] {
    core::slice::from_raw_parts(
        input.as_ptr() as *const u8,
        input.len() * core::mem::size_of::<T>(),
    )
}
