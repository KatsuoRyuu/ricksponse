pub trait PayloadControl {
    const MAX_PAYLOAD_SIZE: Option<usize>;
    const BUFFER_CAPACITY: Option<usize>;
}
